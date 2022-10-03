use dotenv::dotenv;
use poise::serenity_prelude as serenity;

mod commands;
mod configuration;
mod model;
use crate::commands::{quiz, run};
use crate::model::container::{get_container_settings, ContainerActions};
use std::process::exit;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;
// User data, which is stored and accessible in all command invocations
pub struct Data {}

/// Registers or unregisters application commands in this guild or globally
#[poise::command(prefix_command, hide_in_help)]
async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    // Before anything, pull the latest container image for running rust code
    // We will use RustBot's runner image for this
    // https://github.com/TheConner/RustBot/pkgs/container/rustbot-runner
    if let Err(e) = get_container_settings().pull_image() {
        println!("Error pulling image: {:?}", e);

        // Fail & bail
        exit(-1);
    };

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
        .user_data_setup(move |_ctx, _ready, _framework| Box::pin(async move { Ok(Data {}) }));

    framework.run().await.unwrap();
}
