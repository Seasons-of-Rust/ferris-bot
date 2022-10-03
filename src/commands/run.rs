use crate::model::runnable::*;
use crate::Error;
use serenity::prelude::Mentionable;

use std::io::ErrorKind;

/// Given some stdout or stderr data, format it so that it can be rendered by discord
fn format_output(response: String, syntax_highlight: Option<&str>) -> String {
    if response.len() < 1000 {
        // Response falls within size constraints
        format!("```{}\n{}\n```", syntax_highlight.unwrap_or(""), response)
    } else {
        // For UX, truncate components to 1000 chars... should be long enough
        let short_repsonse = &response[0..1000];
        format!(
            "```{}\n{}[TRUNCATED]```",
            syntax_highlight.unwrap_or(""),
            short_repsonse
        )
    }
}

async fn reply(
    ctx: poise::ApplicationContext<'_, crate::Data, crate::Error>,
    code: String,
    stdout: Option<String>,
    stderr: Option<String>,
) -> Result<(), Error> {
    let interaction = ctx.interaction.unwrap();
    let channel = match interaction.channel_id.to_channel(&ctx.discord.http).await {
        Ok(channel) => channel,
        Err(why) => {
            println!("Error getting channel: {:?}", why);
            return Ok(());
        }
    };
    let member = interaction.member.clone().unwrap();

    // TODO: probably a nicer way to do this
    let mut fields = vec![("Code", format_output(code, Some("rs")), true)];

    // If stdout is present, add it to the fields
    if let Some(stdout) = stdout {
        // Ensure that the stdout is not empty
        if !stdout.is_empty() {
            fields.push(("Output", format_output(stdout, None), false));
        }
    }

    // If stderr is present, add it to the fields
    if let Some(stderr) = stderr {
        // Ensure stderr is not empty
        if !stderr.is_empty() {
            fields.push(("Error", format_output(stderr, None), false));
        }
    }

    channel
        .id()
        .send_message(&ctx.discord.http, |m| {
            m.content(format!("{} ran", member.mention()));
            m.embed(|e| {
                e.fields(fields);
                e
            })
        })
        .await?;
    Ok(())
}

#[derive(Debug, poise::Modal)]
#[allow(dead_code)] // fields only used for Debug print
struct RunModal {
    #[name = "Code you want to run"]
    #[placeholder = "fn main() {\n    println!(\"Hello, world!\");\n}"]
    #[paragraph]
    code_to_run: String,
}

/// Runs whatever code you throw at it
#[poise::command(slash_command)]
pub async fn run(
    ctx: poise::ApplicationContext<'_, crate::Data, crate::Error>,
) -> Result<(), Error> {
    use poise::Modal as _;

    let _channel = match ctx
        .interaction
        .unwrap()
        .channel_id
        .to_channel(&ctx.discord.http)
        .await
    {
        Ok(channel) => channel,
        Err(why) => {
            println!("Error getting channel: {:?}", why);
            return Ok(());
        }
    };

    let modal_data = RunModal::execute(ctx).await?;
    let raw_code = modal_data.code_to_run;

    // This leverages the runnable trait we created for executing arbitrary strings of code
    let run_result = raw_code.run().await;

    match run_result {
        Ok(output) => {
            let mut stdout = String::new();
            let mut stderr = String::new();

            if !output.stdout.is_empty() {
                stdout = match String::from_utf8(output.stdout) {
                    Ok(v) => v,
                    Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
                };

                println!("Got stdout\n\"{}\"", stdout);
            } else {
                println!("No stdout");
            }

            if !output.stderr.is_empty() {
                stderr = match String::from_utf8(output.stderr) {
                    Ok(v) => v,
                    Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
                };
                println!("Got stderr \"{}\"", stderr);
            } else {
                println!("No stderr");
            }

            // TODO: better response classification
            // in the original rustbot we used reactions to indicate successful or failed compilation
            // or timeouts. With discord's command framework, it's a little more tricky.
            // For now we just use a canned response for everything, in the future it would be nice to add more
            // detailed responses for each type of response.
            reply(ctx, raw_code, Some(stdout), Some(stderr)).await?;
        }
        Err(error) => {
            // TODO: find out ways this can blow up
            //println!("TIMEOUT on {}'s code", interaction.);
            match error.kind() {
                ErrorKind::TimedOut => {
                    // Took too long to run, complain to user
                    //msg.react(&ctx, CROSS_MARK_EMOJI).await?;
                    //msg.react(&ctx, CLOCK_EMOJI).await?;
                    reply(
                        ctx,
                        raw_code,
                        None,
                        Some("Your program took too long to run.".to_owned()),
                    )
                    .await?;
                }
                _ => {
                    println!("Error: {:?}", error);
                }
            }
        }
    }

    Ok(())
}
