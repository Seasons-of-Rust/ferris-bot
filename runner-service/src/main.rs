use std::process::exit;
use std::thread::sleep;
use std::time::{Instant, Duration};

use dotenv::dotenv;
use futures::future::join_all;
use runner::configurable::ConfigurableValue;
use runner::container::{get_container_settings, ContainerActions};
use runner_common::controller::{CONTROLLER_PORT, RegisterRequest};
use runner_common::controller::controller_client::ControllerClient;
use runner_common::runner::RUNNER_PORT;
use runner_common::tonic;
use runner_common::tonic::transport::Server;
use runner_common::runner::runner_server::{RunnerServer};
use service::PodmanRunnerService;

mod configuration;
mod runner;
mod service;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    // Before anything, pull the latest container image for running rust code
    // We will use RustBot's runner image for this
    // https://github.com/TheConner/RustBot/pkgs/container/rustbot-runner
    //if let Err(e) = get_container_settings().pull_image() {
    //    println!("Error pulling image: {:?}", e);
//
    //    // Fail & bail
    //    exit(-1);
    //};

    let addr = "[::1]:50051".parse()?;
    let runner_service = PodmanRunnerService::default();

    // Spawn in one thread the service
    let runner_service_thread =  tokio::spawn(async move {
        println!("Start up runner service");
        Server::builder()
            .add_service(RunnerServer::new(runner_service))
            .serve(addr).await;
    });
    
    // In another thread spawn the discovery / controller client
    let controller_client_thread = tokio::spawn(async move {
        // Delay a bit to ensure the runner service is stood up
        sleep(Duration::from_millis(2500));
        
        println!("Start up controller");
        let controller_host = configuration::CONTROLLER_HOST.value();
        let mut controller_client = ControllerClient::connect(format!("http://{}:{}", controller_host, CONTROLLER_PORT)).await.unwrap();
        println!("connected to controller, do registration request");
        let register_request = tonic::Request::new(RegisterRequest {
            host: "localhost".into(), // TODO get hostname dynamically
            port: RUNNER_PORT.into()  // TODO see above
        });
    
        let register_response = controller_client.register(register_request).await;
        println!("registration response: {:?}", register_response);
    });

    // Communicate with the controller, register the runner with it
    join_all([runner_service_thread, controller_client_thread]).await;
    
    Ok(())
}