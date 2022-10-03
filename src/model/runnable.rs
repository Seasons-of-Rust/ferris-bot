use async_trait::async_trait;
use process_control::{ChildExt, Control, Output};
use std::io;
use std::io::Error;
use std::time::Duration;

use crate::model::container::{get_container_settings, ContainerActions, ContainerSettings};

#[async_trait]
pub trait Runnable {
    async fn run(&self) -> Result<Output, Error>;
    async fn run_with_settings(
        &self,
        container_settings: ContainerSettings,
    ) -> Result<Output, Error>;
}

#[async_trait]
impl Runnable for String {
    async fn run(&self) -> Result<Output, Error> {
        let settings = get_container_settings();
        self.run_with_settings(settings).await
    }

    async fn run_with_settings(
        &self,
        container_settings: ContainerSettings,
    ) -> Result<Output, Error> {
        // TODO: the original rustbot had support for running with arguments, may be worth adding this in the future
        // https://github.com/TheConner/RustBot/blob/main/src/commands/run.rs#L37-L41

        // In order to run an arbitrary string with the current design, we have to first base64 the content
        // and then run the command with the base64'd content as stdin.
        let encoded_program = base64::encode(&self);

        // Next, we have to build the command that invokes the trampoline inside the container
        let container_command = format!("trampoline {}", encoded_program);

        //println!("{}", container_command);

        // Because std::command does not give me the ability to override / modify
        // how arguments are escaped I have to do some stupid hack to make this
        // work. For example, if I wanted to run
        // podman run rustbot:latest ls -al
        // this would be impossible if I did
        //
        //  std::process::Command::new("podman")
        //    .args(["run", "rustbot:latest", "ls -al"])
        //    .output()
        //    .expect("failed to invoke container");
        //
        // As the ls -al would be quoted, and the container would try to execute
        // `ls -al` which would fail. The alternative is to seperate "ls", "-al"
        // which would also fail as the container would run `ls` then `-al`
        // ... what a stupid design
        // So instead of embracing the safety this API gives you, i'm just invoking
        // a shell with a payload I deem as safe
        let process = container_settings.invoke_command(container_command);

        process?
            .controlled_with_output()
            .time_limit(Duration::from_millis(container_settings.max_runtime))
            .terminate_for_timeout()
            .wait()?
            .ok_or_else(|| Error::new(io::ErrorKind::TimedOut, "Process timed out"))
    }
}
