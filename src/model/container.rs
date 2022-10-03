use crate::configuration;
use crate::model::configurable::ConfigurableValue;
use std::io;
use std::io::Error;
use std::process::{Command, Stdio};

/// Settings for our container
#[derive(Clone)]
pub struct ContainerSettings {
    pub cpu: String,
    pub memory: String,
    pub swap: String,
    pub image: String,
    pub max_runtime: u64,
    pub network: String,
    pub pid_limit: u64,
}

pub trait ContainerActions {
    fn container_command(&self) -> String;
    fn generate_runtime_flags(&self) -> String;
    fn pull_image(&self) -> Result<(), Error>;
    fn invoke_command(&self, command: String) -> io::Result<std::process::Child>;
}

impl ContainerActions for ContainerSettings {
    /// Gets the container command to use
    /// Either podman-remote or podman
    fn container_command(&self) -> String {
        // Are we running this in a container?
        if configuration::IS_RUNNING_IN_CONTAINER.value() {
            // Invoke podman with remote socket that should be passed to the container
            String::from("podman-remote")
        } else {
            // Invoke podman the default way
            String::from("podman")
        }
    }

    /// Turns a ContainerSettings instance into a string of CLI args for Podman or Docker
    fn generate_runtime_flags(&self) -> String {
        format!(
            "--cap-drop=ALL --security-opt=no-new-privileges --cpus={} --memory={} --network={} --pids-limit={}",
            self.cpu, self.memory, self.network, self.pid_limit
        )
    }

    /// Pulls a container image from a registry
    fn pull_image(&self) -> Result<(), Error> {
        let output = Command::new(self.container_command())
            .arg("pull")
            .arg(&self.image)
            .status()
            .expect("failed to execute process");

        let status = output.code().expect("No output code");

        if status == 0 {
            Ok(())
        } else {
            Result::Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Could not pull container image, got error code {}", status),
            ))
        }
    }

    fn invoke_command(&self, command: String) -> io::Result<std::process::Child> {
        let container_command = format!(
            "{} run --rm {} {} {}",
            self.container_command(),
            self.generate_runtime_flags(),
            self.image,
            command
        );

        println!("{}", container_command);

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
        Command::new("sh")
            .args(["-c", container_command.as_str()])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

/// Gets the default container settings
pub fn get_container_settings() -> ContainerSettings {
    ContainerSettings {
        cpu: (*configuration::CONTAINER_CPU).value(),
        image: (*configuration::CONTAINER_IMAGE).value(),
        memory: (*configuration::CONTAINER_MEMORY).value(),
        swap: (*configuration::CONTAINER_SWAP).value(),
        max_runtime: (*configuration::CONTAINER_MAX_RUNTIME).value(),
        network: (*configuration::CONTAINER_NETWORK).value(),
        pid_limit: (*configuration::CONTAINER_PIDS).value(),
    }
}
