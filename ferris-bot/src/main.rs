use commands::run;
use futures::lock::Mutex;
use runner_common::runner::Empty;
use runner_common::tonic::Request;
use std::time::{Duration};
use controller::{RunnerController, RunnerControllerService};
use dotenv::dotenv;
use futures::future::join_all;
use poise::serenity_prelude::{self as serenity};
use runner_common::controller::controller_server::{ControllerServer};
use runner_common::tonic::transport::Server;
use tokio::time::sleep;
use crate::commands::{quiz};
use std::sync::{RwLock};
use std::{sync::Arc};

mod commands;
mod model;
mod controller;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

/// Registers or unregisters application commands in this guild or globally
#[poise::command(prefix_command, hide_in_help)]
async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;

    Ok(())
}

// User data, which is stored and accessible in all command invocations
pub struct Data {
    //shared_client: SharedRunnerClient,
    client_pool: Arc<RwLock<RunnerController>>
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let controller_service = RunnerControllerService::default();
    let bot_controller = Arc::clone(&controller_service.controller);

    let controller_future = tokio::spawn(async move {
        let addr = "[::1]:50000".parse().unwrap();
        println!("Start up service discovery");
        let _srv_result = Server::builder()
            .add_service(ControllerServer::new(controller_service))
            .serve(addr)
            .await;
    });

    let bot_future = tokio::spawn(async move {
        let framework = poise::Framework::build()
        .options(poise::FrameworkOptions {
            commands: vec![register(), quiz::quiz(), run::run()],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("!".into()),
                ..Default::default()
            },
            ..Default::default()
        })
        .token(std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN"))
        .intents(
            serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT,
        )
        .user_data_setup(move |_ctx, _ready, _framework| Box::pin(async move { Ok(Data {
            client_pool: bot_controller
        })}));
        framework.run().await;
    });

    join_all([controller_future, bot_future]).await;
    Ok(())
}
