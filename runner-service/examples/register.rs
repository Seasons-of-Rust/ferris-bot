///! Example: connect using service discovery
use futures::future::join_all;
use runner_common::controller::RegisterRequest;
use runner_common::controller::controller_client::ControllerClient;
use runner_common::tonic;
use runner_common::runner::{Empty};
use runner_common::runner::runner_client::RunnerClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut futures = Vec::new();
    let client = ControllerClient::connect("http://[::1]:50000").await.unwrap();

    for i in 0..10 {
        let mut client_handle = client.clone();
        futures.push(tokio::spawn(async move {
            println!("Execute {}", i);
            let request = tonic::Request::new(RegisterRequest {
              host: "localhost".into(),
              port: (50001 + i).to_string()
            });
            let res = client_handle.register(request).await;
                println!("RESPONSE={:?}", res);
            }));
    }
    join_all(futures).await;
    Ok(())
}