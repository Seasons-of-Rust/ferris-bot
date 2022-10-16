use async_trait::async_trait;
use futures::Future;
use futures::lock::Mutex;
use runner_common::controller::{RegisterRequest, RegisterResponse};
use runner_common::controller::controller_server::{Controller};
use runner_common::runner::Empty;
use runner_common::tonic::{transport, Request, Response, Status};
use runner_common::runner::{runner_client::RunnerClient};
use std::sync::{RwLock};
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

#[derive(Debug, Default)]
pub struct RunnerController {
    // Used to assign IDs to controllers
    runner_counter: Arc<RwLock<i32>>,
    // Runner connection pool
    // todo: unwrap this onion
    //clients: Arc<Mutex<Vec<RunnerClient<transport::Channel>>>>,
    clients: Arc<RwLock<Vec<RunnerClient<transport::Channel>>>>,
    // Last controller used (index in vec, not controller ID)
    last_controller: Arc<RwLock<usize>>
}

impl RunnerController {
  // Creates a new RunnerControllerService instance
  // fn new() -> RunnerController {
  //     RunnerController { 
  //         runner_counter: Arc::new(RwLock::new(0)),
  //         clients: Vec::new(),
  //         last_controller: Arc::new(RwLock::new(0))
  //     }
  // }

  // Returns the next client ID
  fn next_client_id(&self) -> i32 {
      let mut id_ref = self.runner_counter.write().unwrap();
      let current_id = (*id_ref).clone();
      *id_ref += 1;
      current_id
  }

  // Fetches the next client round-robin style
  pub fn rr_next_client(&self) -> RunnerClient<transport::Channel> {
      let num_clients = self.clients.read().unwrap().len();
      let next_client_index = (*self.last_controller.read().unwrap() + 1) % num_clients;
      // TODO: everything below this is nasty
      *(self.last_controller.write().unwrap()) = next_client_index;
      self.clients.read().unwrap().get(next_client_index).unwrap().clone()
  }

  // Adds a new client, returns new client ID
  fn add_client(&self, client: RunnerClient<transport::Channel>) -> i32 {
      self.clients.write().unwrap().push(client);
      self.next_client_id()
  }
}

#[derive(Debug, Default)]
pub struct RunnerControllerService {
    pub controller: Arc<RwLock<RunnerController>>
}

#[async_trait]
impl Controller for RunnerControllerService {
    async fn register(
        &self,
        request: Request<RegisterRequest>
    ) -> Result<Response<RegisterResponse>, Status> {
        let req_data = request.into_inner();
        println!("register {}:{}", req_data.host, req_data.port);
        // Try to connect to the runner service using the registration info provided
        // very secure i know
        // this is supposed to be ran in an isolated network so no biggie
        let client_connection = RunnerClient::new_host_port(req_data.host, req_data.port).await;

        // Add the host to the connection pool
        println!("connected to client, register client...");
        let next_id = self.controller.read().unwrap().add_client(client_connection);
        
        let res = RegisterResponse { 
            nodeid: next_id
        };

        println!("Respond: {:?}", res);
        Ok(Response::new(res))
    }
}