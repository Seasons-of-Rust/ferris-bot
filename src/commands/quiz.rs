use crate::model::question::QuestionTF;
use crate::playground::{get_playground_link, run_code};
use crate::Error;
use poise::serenity_prelude::interaction::InteractionResponseType;
use serenity::futures::StreamExt;
use serenity::prelude::Mentionable;
use serenity::utils::MessageBuilder;
use serenity_layer::prelude::Params;
use std::fs::File;
use std::io::Read;
use std::str::FromStr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use url::{ParseError, Url};

const QUESTION_TIME: u64 = 30;

/// Starts a quiz
#[poise::command(slash_command)]
pub async fn quiz(
    ctx: poise::ApplicationContext<'_, crate::Data, crate::Error>,
    #[description = "The choice you want to choose"] question_number: Option<i64>,
    #[description = "The Rust Playground Share link"] playground: Option<String>,
) -> Result<(), Error> {
    // Get the serenity interaction for the slash command
    // TODO: error handling...
    let slash_command = ctx.interaction.unwrap();

    let channel = match slash_command.channel_id.to_channel(&ctx.discord.http).await {
        Ok(channel) => channel,
        Err(why) => {
            println!("Error getting channel: {:?}", why);
            return Ok(());
        }
    };

    let answers = [
        QuestionTF::True,
        QuestionTF::True,
        QuestionTF::False,
        QuestionTF::False,
        QuestionTF::True,
        QuestionTF::True,
        QuestionTF::False,
    ];

    let question_number: Option<i64> = None;

    let mut contents = String::new();

    match (question_number, playground) {
        (Some(question_number), None) => {
            if question_number > answers.len() as i64 {
                println!("Question number out of bounds");
                return Ok(());
            }

            // Load text from file question/q1.rs
            let mut file = File::open(format!("questions/q{}.rs", question_number))?;
            file.read_to_string(&mut contents)?;
        }
        (None, Some(playground)) => {
            // Download the code from the playground
            // https://gist.github.com/f3a3aef951b6734cbf9eadbfd6f4c2ef
            // https://gist.githubusercontent.com/rust-play/f3a3aef951b6734cbf9eadbfd6f4c2ef/raw/4c8b98a7ddbe9c8ba06fa1323113b8c268c45457/playground.rs
            // https://gist.githubusercontent.com/rust-play/f3a3aef951b6734cbf9eadbfd6f4c2ef/raw/playground.rs

            let parse = Url::parse(&playground).unwrap();

            // Make sure the link comes in the form
            // https://gist.github.com/f3a3aef951b6734cbf9eadbfd6f4c2ef
            if parse.domain() != Some("gist.github.com") {
                println!("Invalid domain");
                return Ok(());
            }

            // Get the domain path
            let gist_id = parse.path_segments().unwrap().last().unwrap();

            // Build the link
            let gist_raw_link = format!(
                "https://gist.githubusercontent.com/rust-play/{}/raw/playground.rs",
                gist_id
            );

            // Download the code
            let mut response = reqwest::get(&gist_raw_link).await?;

            // Read the response
            contents = response.text().await?;
        }
        _ => {
            // Reply with an error
            return Ok(());
        }
    }

    // Ask the question
    ctx.interaction
        .unwrap()
        .create_interaction_response(&ctx.discord.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.content(MessageBuilder::new().push("Starting Quiz!").build())
                })
        })
        .await?;

    // Get the current number of seconds since the epoch
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let m = channel
        .id()
        .send_message(&ctx.discord.http, |m| {
            m.content(
                MessageBuilder::new()
                    .push("Does this code compile?")
                    .push("```rust\n")
                    .push(&contents)
                    .push("```")
                    .push(format!("Showing answer <t:{}:R>", now + QUESTION_TIME))
                    .build(),
            )
            .components(|c| c.add_action_row(QuestionTF::action_row()))
        })
        .await
        .unwrap();

    // Send the code to the playground to test
    let code_result = run_code(Params::default(), contents.clone()).await?;

    // The code compiled if the second last line doesn't start with "error:"
    let compiled = !code_result
        .lines()
        .nth_back(1)
        .unwrap()
        .starts_with("error:");

    // Wait for a responses within a certain amount of time
    let mut cib = m
        .await_component_interactions(&ctx.discord)
        .timeout(Duration::from_secs(QUESTION_TIME))
        .build();

    let mut correct_answers = Vec::new();

    while let Some(mci) = cib.next().await {
        println!("{:?}", mci.data);
        let question_choice = QuestionTF::from_str(&mci.data.custom_id).unwrap();

        let member = mci.member.clone().unwrap();

        if compiled == (&mci.data.custom_id == "true") {
            correct_answers.push(member);
        }

        // Acknowledge the interaction and send a reply
        mci.create_interaction_response(&ctx.discord, |r| {
            // This time we dont edit the message but reply to it
            r.kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|d| {
                    // Make the message hidden for other users by setting `ephemeral(true)`.
                    d.ephemeral(true)
                        .content(format!("You choose {}", question_choice))
                })
        })
        .await
        .unwrap();
    }

    m.delete(&ctx.discord).await.unwrap();

    // Write a message with people who got the question right
    let _m = channel
        .id()
        .send_message(&ctx.discord, |m| {
            let mut builder = MessageBuilder::new();

            builder.push("The following people got the question right:\n\n");

            for member in correct_answers {
                builder.push(member.mention()).push(" ");
            }

            m.content(
                builder
                    .push("\n\nThe correct answer was: ")
                    .push(compiled)
                    .push("```rust\n")
                    .push(&contents)
                    .push("```")
                    .push(code_result)
                    .build(),
            )
        })
        .await
        .unwrap();

    Ok(())
}
