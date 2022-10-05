use dotenv::dotenv;
use poise::serenity_prelude::{self as serenity};
use runner_common::runner::{runner_client::RunnerClient, SharedRunnerClient};

mod commands;

mod model;
use crate::commands::{quiz, run};
use std::{process::exit, sync::Arc};

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;
// User data, which is stored and accessible in all command invocations
pub struct Data {
    shared_client: SharedRunnerClient
}

/// Registers or unregisters application commands in this guild or globally
#[poise::command(prefix_command, hide_in_help)]
async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let client = 

    println!("Starting up...");
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
            shared_client: SharedRunnerClient::new().await
        })}));

    framework.run().await.unwrap();
}
