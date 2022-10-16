use std::time::Instant;

use runner_common::tonic::{async_trait, Request, Response, Status};
use runner_common::runner::runner_server::{Runner};
use runner_common::runner::{ExecuteRequest, ExecuteResponse, DescribeResponse, ExecuteStatus, Empty};

use crate::runner::runnable::*;

#[derive(Debug, Default)]
pub struct PodmanRunnerService {}

#[async_trait]
impl Runner for PodmanRunnerService {
    async fn execute(
        &self,
        request: Request<ExecuteRequest>,
    ) -> Result<Response<ExecuteResponse>, Status> {
        // TODO: logging
        let request = request.into_inner();
        let now = Instant::now();
        let run_result = request.program.run().await;
        let elapsed = now.elapsed();

        // TODO: in the future, handle args
        let output = run_result.unwrap();
        let stdout = String::from_utf8(output.stdout).unwrap();
        let stderr = String::from_utf8(output.stderr).unwrap();

        let reply = ExecuteResponse {
            retcode: 0,
            stdout: stdout,
            stderr: stderr,
            status: ExecuteStatus::Ok.into(),
            // Highly doubt the duration will ever get to the 128-bit size...
            // gRPC's largest integer type is u64
            // TODO: deal with overflow
            duration: elapsed.as_millis() as u64
        };

        Ok(Response::new(reply))
    }

    async fn describe(&self, request: Request<Empty>) -> Result<Response<DescribeResponse>, Status> {
        let reply = DescribeResponse {
            host: "localhost".into()
        };
        Ok(Response::new(reply))
    }

    async fn heartbeat(&self, request: Request<Empty>) -> Result<Response<Empty>, Status> {
        let reply = Empty {};
        Ok(Response::new(reply))
    }
}