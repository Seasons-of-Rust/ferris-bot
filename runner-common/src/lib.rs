///! Common library code for clients and servers that run code
///! Communication takes place over gRPC

/// Runner module, code is generated by tonic based on the runner proto file
pub mod runner {
    use std::sync::Arc;

    use tonic::transport::Channel;

    use self::runner_client::RunnerClient;

    tonic::include_proto!("runner");

    pub struct SharedRunnerClient {
        pub client: RunnerClient<Channel>
    }

    impl SharedRunnerClient {
        /// Creates a new instance with default client options
        pub async fn new() -> SharedRunnerClient {
            SharedRunnerClient {
                client: RunnerClient::connect("http://[::1]:50051").await.unwrap()
            }
        }
        
        /// Gets a client to work with
        pub fn get(&self) -> RunnerClient<Channel> {
            // Cloning the tonic client / channel is low cost, see
            // https://docs.rs/tonic/latest/tonic/transport/struct.Channel.html#multiplexing-requests
            self.client.clone()
        }
    }
}

/// Re-export tonic because I am lazy
pub use tonic as tonic;