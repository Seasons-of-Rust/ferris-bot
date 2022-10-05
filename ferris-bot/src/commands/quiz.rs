use std::fs::File;
use std::io::Read;
use std::str::FromStr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serenity::futures::StreamExt;

use serenity::model::interactions::InteractionResponseType;

use serenity::prelude::Mentionable;
use serenity::utils::MessageBuilder;

use crate::model::question::QuestionTF;
use crate::Error;

const QUESTION_TIME: u64 = 30;

/// Starts a quiz
#[poise::command(slash_command)]
pub async fn quiz(
    ctx: poise::ApplicationContext<'_, crate::Data, crate::Error>,
    #[description = "The choice you want to choose"] question_number: i64,
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

    if question_number > answers.len() as i64 {
        println!("Question number out of bounds");
        return Ok(());
    }

    // Get the current number of seconds since the epoch
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Load text from file question/q1.rs
    let mut file = File::open(format!("questions/q{}.rs", question_number))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

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

        if question_choice == answers[(question_number - 1) as usize] {
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
                    .push(answers[(question_number - 1) as usize].to_string())
                    .push("```rust\n")
                    .push(&contents)
                    .push("```")
                    .build(),
            )
        })
        .await
        .unwrap();

    Ok(())
}
