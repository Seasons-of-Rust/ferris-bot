///! Example: invoke hello world program on the runner a bunch of times
///! Needs the runner service to be running
///! cargo run --example many-hello-world
use futures::future::join_all;
use runner_common::tonic;
use runner_common::runner::{ExecuteRequest, Language};
use runner_common::runner::runner_client::RunnerClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut futures = Vec::new();
    let client = RunnerClient::connect("http://[::1]:50051").await.unwrap();

    for i in 0..128 {
        let mut client_handle = client.clone();
        futures.push(tokio::spawn(async move {
            println!("Execute {}", i);
            let request = tonic::Request::new(ExecuteRequest {
                language: Language::LangRust.into(),
                program: format!("fn main() {{\nprintln!(\"Hello World {}!\");\n}}", i).into(),
                args: "".into()
            });
            let res = client_handle.execute(request).await;
                println!("RESPONSE={:?}", res);
            }));
    }
    join_all(futures).await;
    Ok(())
}