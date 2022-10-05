use std::process::exit;

use runner::container::{get_container_settings, ContainerActions};
use tonic::transport::Server;
use runner_common::runner::runner_server::{RunnerServer};
use service::PodmanRunnerService;

mod configuration;
mod runner;
mod service;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Before anything, pull the latest container image for running rust code
    // We will use RustBot's runner image for this
    // https://github.com/TheConner/RustBot/pkgs/container/rustbot-runner
    if let Err(e) = get_container_settings().pull_image() {
        println!("Error pulling image: {:?}", e);

        // Fail & bail
        exit(-1);
    };

    let addr = "[::1]:50051".parse()?;
    let runner_service = PodmanRunnerService::default();

    Server::builder()
        .add_service(RunnerServer::new(runner_service))
        .serve(addr)
        .await?;

    Ok(())
}