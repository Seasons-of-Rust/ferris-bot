use std::{collections::HashMap, sync::Arc};

use git_version::git_version;
use regex::Regex;
use serde_json::{from_str, json, Value};
use serenity::{
    async_trait,
    model::{
        channel::GuildChannel,
        gateway::Ready,
        guild::{Guild, PremiumTier},
        interactions::{
            application_command::{
                ApplicationCommandInteraction, ApplicationCommandInteractionDataOption,
                ApplicationCommandInteractionDataOptionValue, ApplicationCommandOptionType,
            },
            Interaction, InteractionResponseType,
        },
        prelude::GuildId,
    },
    prelude::*,
    utils::MessageBuilder,
};
use tracing::{error, info, trace, warn};

#[derive(Clone)]
pub struct GitlabProject {
    pub name: String,
    pub id: String,
    pub discord_thread_prefix: String,
}

impl GitlabProject {
    pub fn new(name: &str, id: usize, discord_thread_prefix: &str) -> GitlabProject {
        GitlabProject {
            name: name.to_string(),
            id: id.to_string(),
            discord_thread_prefix: discord_thread_prefix.to_string(),
        }
    }
}

pub struct Handler {
    pub client: reqwest::Client,
    pub gitlab_token: String,
    pub gitlab_projects: HashMap<&'static str, GitlabProject>,
    pub periodic_task_context: Arc<RwLock<Option<Context>>>,
}

const REVIEW_STRING: &str = "review";
const APPROVE_STRING: &str = "approve";
const VERSION_STRING: &str = "labbot-version";

pub const VELOREN_SERVER_ID: u64 = 449602562165833758;

impl Handler {
    async fn interaction_create(
        &self,
        context: Context,
        interaction: Interaction,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Get the slash command, or return if it's not a slash command.
        let slash_command = if let Some(slash_command) = interaction.application_command() {
            slash_command
        } else {
            return Ok(());
        };

        if let Err(e) = slash_command.channel_id.to_channel(&context).await {
            warn!("Error getting channel: {:?}", e);
        };

        match &slash_command.data.name[..] {
            REVIEW_STRING => {
                let merge_request_number = slash_command
                    .data
                    .options
                    .get(0)
                    .expect("Expected int option")
                    .resolved
                    .as_ref()
                    .expect("Expected int object");

                let gitlab_project_name_option = slash_command.data.options.get(1);

                let gitlab_project_name = get_gitlab_project_name(gitlab_project_name_option)
                    .ok_or("Gitlab project name isn't a string")?;

                let gitlab_project = self
                    .get_gitlab_project(context.clone(), slash_command.clone(), gitlab_project_name)
                    .await?
                    .ok_or(format!(
                        "Unknown Gitlab project name: {}",
                        gitlab_project_name
                    ))?;

                if let ApplicationCommandInteractionDataOptionValue::Integer(number) =
                    merge_request_number
                {
                    self.open_discord_mr_thread(
                        context,
                        slash_command.clone(),
                        *number as i64,
                        gitlab_project,
                    )
                    .await?
                } else {
                    warn!("Merge request isn't a number");
                    return Ok(());
                }
            }
            APPROVE_STRING => {
                let merge_request_number = slash_command
                    .data
                    .options
                    .get(0)
                    .expect("Expected int option")
                    .resolved
                    .as_ref()
                    .expect("Expected int object");

                let gitlab_project_name_option = slash_command.data.options.get(1);

                let gitlab_project_name = get_gitlab_project_name(gitlab_project_name_option)
                    .ok_or("Gitlab project name isn't a string")?;

                let gitlab_project = self
                    .get_gitlab_project(context.clone(), slash_command.clone(), gitlab_project_name)
                    .await?
                    .ok_or(format!(
                        "Unknown Gitlab project name: {}",
                        gitlab_project_name
                    ))?;

                if let ApplicationCommandInteractionDataOptionValue::Integer(number) =
                    merge_request_number
                {
                    self.approve_mr(
                        context,
                        slash_command.clone(),
                        *number as i64,
                        gitlab_project,
                    )
                    .await?
                } else {
                    warn!("Merge request isn't a number");
                    return Ok(());
                }
            }
            VERSION_STRING => {
                slash_command
                    .create_interaction_response(&context.http, |response| {
                        response
                            .kind(InteractionResponseType::ChannelMessageWithSource)
                            .interaction_response_data(|message| {
                                message.content(
                                    MessageBuilder::new()
                                        // This will show an error with
                                        // rust-analyzer, but it compiles just fine
                                        // https://github.com/rust-analyzer/rust-analyzer/issues/6835
                                        //
                                        // Example output: `git:efe04ac-modified`
                                        .push(git_version!(prefix = "git:", fallback = "unknown"))
                                        .build(),
                                )
                            })
                    })
                    .await?;
            }
            _ => {
                warn!("should not happen");
                return Ok(());
            }
        }

        Ok(())
    }

    async fn get_gitlab_project(
        &self,
        context: Context,
        application_command: ApplicationCommandInteraction,
        gitlab_project_name: &str,
    ) -> Result<Option<&GitlabProject>, Box<dyn std::error::Error>> {
        match self.gitlab_projects.get(gitlab_project_name) {
            Some(project) => Ok(Some(project)),
            None => {
                application_command
                    .create_interaction_response(&context.http, |response| {
                        response
                            .kind(InteractionResponseType::ChannelMessageWithSource)
                            .interaction_response_data(|message| {
                                message.content(format!(
                                    "!{} is not a valid project",
                                    gitlab_project_name
                                ))
                            })
                    })
                    .await?;
                Ok(None)
            }
        }
    }

    async fn verify_mr(
        &self,
        merge_request_number: i64,
        gitlab_project: &GitlabProject,
    ) -> Result<Result<String, String>, Box<dyn std::error::Error>> {
        // Query the GitLab API with the Veloren repo to find the MR
        let request = self
            .client
            .get(format!(
                "https://gitlab.com/api/v4/projects/{}/merge_requests/{}",
                gitlab_project.id, merge_request_number
            ))
            .build()?;
        let gitlab_response = self.client.execute(request).await?;
        let gitlab_response_body = gitlab_response.text().await?;
        let body_json: Value = from_str(&gitlab_response_body)?;

        // If the API call didn't return a valid title, send a response
        // to the channel
        if body_json["title"] == Value::Null {
            return Ok(Err(format!(
                "Error: !{} is not a valid Merge Request in !{}",
                merge_request_number, gitlab_project.name
            )));
        }

        // If the MR isn't opened, respond with an error. This is to try
        // and prevent people from accidentally creating threads on dead
        // issues
        if body_json["state"] != "opened" {
            return Ok(Err(format!(
                "Error: !{} is not an opened Merge Request in !{}",
                merge_request_number, gitlab_project.name
            )));
        }
        Ok(Ok(body_json["title"].to_string()))
    }

    async fn verify_role(
        &self,
        context: &Context,
        application_command: &ApplicationCommandInteraction,
        role_name: &str,
    ) -> Result<(GuildId, Guild), Box<dyn std::error::Error>> {
        let guild_id = application_command.guild_id.ok_or("Cannot get guild_id")?;
        let guild = guild_id
            .to_guild_cached(&context)
            .ok_or("Cannot get guild")?;

        // Make sure the author is at least a contributor
        if let Some(role) = guild.role_by_name(role_name) {
            if application_command
                .user
                .has_role(&context.http, guild_id, role)
                .await?
            {
                Ok((guild_id, guild))
            } else {
                application_command
                    .create_interaction_response(&context.http, |response| {
                        response
                            .kind(InteractionResponseType::ChannelMessageWithSource)
                            .interaction_response_data(|message| {
                                message.content(format!(
                                    "You need to be a {} to use this command",
                                    role_name
                                ))
                            })
                    })
                    .await?;
                None.ok_or("user doesnt has role")?
            }
        } else {
            warn!(?role_name, "role doesnt exist");
            None.ok_or("couldn't verify role")?
        }
    }

    async fn open_discord_mr_thread(
        &self,
        context: Context,
        application_command: ApplicationCommandInteraction,
        merge_request_number: i64,
        gitlab_project: &GitlabProject,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Verify that the MR is open
        let mr = self.verify_mr(merge_request_number, gitlab_project).await?;
        let title = match mr {
            Ok(title) => title,
            Err(content) => {
                application_command
                    .create_interaction_response(&context.http, |response| {
                        response
                            .kind(InteractionResponseType::ChannelMessageWithSource)
                            .interaction_response_data(|message| message.content(content))
                    })
                    .await?;
                return Ok(());
            }
        };

        // Make sure that the user is a Contributor to create thread
        let (_guild_id, guild) = self
            .verify_role(&context, &application_command, "Contributor")
            .await?;

        // If there is already a thread with the MR number, return a message
        // that links to it
        let mr_threads = get_merge_request_threads(guild.clone(), gitlab_project)?;
        if let Some(thread) = mr_threads.get(&merge_request_number) {
            application_command
                .create_interaction_response(&context.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| {
                            message.content(
                                MessageBuilder::new()
                                    .push(format!(
                                        "A thread for MR {} of {} already exists here: ",
                                        merge_request_number, gitlab_project.name,
                                    ))
                                    .mention(thread)
                                    .build(),
                            )
                        })
                })
                .await?;
            return Ok(());
        }

        // Now that all errors are handled, we can create the thread
        if let Some(code_reviwer_role) = guild.role_by_name("Code Reviewer") {
            let merge_request_response = MessageBuilder::new()
                .push("MR Thread for ")
                .push(title.clone())
                .build();

            application_command
                .create_interaction_response(&context.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| {
                            message.content(merge_request_response)
                        })
                })
                .await?;

            // Set the time for the thread to live before auto archive to the
            // max possible based on the tier of the server
            let archive_duration = match guild.premium_tier {
                PremiumTier::Tier0 => 1440,
                PremiumTier::Tier1 => 4320,
                PremiumTier::Tier2 => 10080,
                PremiumTier::Tier3 => 10080,
                _ => 1440,
            };

            // This will open a thread for the max amount of time possible
            let public_thread_json = json!(
                {
                    "name": format!("{}{} {}", gitlab_project.discord_thread_prefix, merge_request_number, title),
                    "auto_archive_duration": archive_duration,
                    "kind": "GUILD_PUBLIC_THREAD"
                }
            );

            let new_mr_thread = context
                .http
                .create_public_thread(
                    *application_command.channel_id.as_u64(),
                    *application_command
                        .get_interaction_response(&context.http)
                        .await?
                        .id
                        .as_u64(),
                    public_thread_json.as_object().unwrap(),
                )
                .await?;

            // Send the message that pings the parties involved in the MR
            let thread_ping_message = new_mr_thread
                .send_message(&context.http, |m| {
                    m.content(
                        MessageBuilder::new()
                            .push("This thread was created by ")
                            .mention(&application_command.user.id)
                            .push("\n")
                            .mention(code_reviwer_role)
                            .push(" this is ready for review!")
                            .push("\n\n")
                            .push(format!(
                                "https://gitlab.com/veloren/{}/-/merge_requests/{} ",
                                gitlab_project.name, merge_request_number,
                            ))
                            .build(),
                    )
                })
                .await?;

            // Pin the message
            thread_ping_message.pin(&context.http).await?;
        }
        Ok(())
    }

    async fn approve_mr(
        &self,
        context: Context,
        application_command: ApplicationCommandInteraction,
        merge_request_number: i64,
        gitlab_project: &GitlabProject,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Verify that the MR is open
        let mr = self.verify_mr(merge_request_number, gitlab_project).await?;
        let _ = match mr {
            Ok(title) => title,
            Err(content) => {
                application_command
                    .create_interaction_response(&context.http, |response| {
                        response
                            .kind(InteractionResponseType::ChannelMessageWithSource)
                            .interaction_response_data(|message| message.content(content))
                    })
                    .await?;
                return Ok(());
            }
        };

        // Make sure that the user is a Contributor to approve
        let (_guild_id, _guild) = self
            .verify_role(&context, &application_command, "Contributor")
            .await?;

        // Post a note on that MR
        let request = self
            .client
            .post(format!(
                "https://gitlab.com/api/v4/projects/{}/merge_requests/{}/notes",
                gitlab_project.id, merge_request_number
            ))
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .header("PRIVATE-TOKEN", &self.gitlab_token)
            .body(format!(
                r#"{{ "body": "Approved by Discord user: {}" }}"#,
                application_command.user.name
            ))
            .build()?;
        let gitlab_response = self.client.execute(request).await?;
        let gitlab_response_body = gitlab_response.text().await?;

        tracing::trace!(?gitlab_response_body, "gitlab response after note");

        // Now that all errors are handled, we can approve a mr
        let request = self
            .client
            .post(format!(
                "https://gitlab.com/api/v4/projects/{}/merge_requests/{}/approve",
                gitlab_project.id, merge_request_number
            ))
            .header("PRIVATE-TOKEN", &self.gitlab_token)
            .build()?;
        let gitlab_response = self.client.execute(request).await?;
        let gitlab_response_body = gitlab_response.text().await?;

        tracing::trace!(?gitlab_response_body, "gitlab response after mr");

        application_command
            .create_interaction_response(&context.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| {
                        message.content(
                            MessageBuilder::new()
                                .push(format!("MR !{} approved", merge_request_number))
                                .mention(&application_command.user.id)
                                .build(),
                        )
                    })
            })
            .await?;

        Ok(())
    }
}

fn get_gitlab_project_name(
    gitlab_project_name_option: Option<&ApplicationCommandInteractionDataOption>,
) -> Option<&str> {
    if let Some(gitlab_project_name) = gitlab_project_name_option {
        let gitlab_project_name = gitlab_project_name
            .resolved
            .as_ref()
            .expect("Expected string object");
        if let ApplicationCommandInteractionDataOptionValue::String(name) = gitlab_project_name {
            Some(name)
        } else {
            warn!("Gitlab project name isn't a string");
            None
        }
    } else {
        Some("veloren")
    }
}

pub fn get_merge_request_threads(
    guild: Guild,
    gitlab_project: &GitlabProject,
) -> Result<HashMap<i64, GuildChannel>, Box<dyn std::error::Error>> {
    let re = Regex::new(&format!(
        r"^{}(\d{{1,5}}) .*$",
        gitlab_project.discord_thread_prefix
    ))
    .unwrap();
    let threads = guild
        .threads
        .iter()
        .filter_map(|thread| {
            let mut cap = re.captures_iter(&thread.name);
            if let Some(number) = cap.next() {
                return Some((number[1].parse::<i64>().unwrap(), thread.clone()));
            }
            None
        })
        .collect::<HashMap<i64, GuildChannel>>();

    Ok(threads)
}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, context: Context, interaction: Interaction) {
        if let Err(e) = self.interaction_create(context, interaction).await {
            error!(?e, "Error while processing message");
        }
    }

    async fn ready(&self, context: Context, ready: Ready) {
        let name = ready.user.name;
        info!(?name, "is connected!");

        // Create the review command for the Veloren server
        if let Err(e) = GuildId(VELOREN_SERVER_ID)
            .create_application_command(&context.http, |command| {
                command
                    .name("review")
                    .description("Review an MR")
                    .create_option(|option| {
                        option
                            .name("id")
                            .description("The MR to review")
                            .kind(ApplicationCommandOptionType::Integer)
                            .required(true)
                    })
            })
            .await
        {
            error!(?e, "Error while creating the review command");
        }

        if let Err(e) = GuildId(VELOREN_SERVER_ID)
            .set_application_commands(&context.http, |commands| {
                commands
                    // Command to create a check in about if students are
                    // understanding the topic being discussed
                    .create_application_command(|command| {
                        command
                            .name("question")
                            .description("Send an anonymous question directly to Forest")
                    })
                    .create_application_command(|command| {
                        command
                            .name("answer")
                            .description("Send an anonymous answer directly to Forest")
                    })
            })
            .await
        {
            error!(?e, "Error while creating the review command");
        }
    }

    // This mostly came from the Serenity docs
    // https://github.com/serenity-rs/serenity/blob/current/examples/e13_parallel_loops/src/main.rs
    async fn cache_ready(&self, context: Context, _guilds: Vec<GuildId>) {
        trace!("Cache built successfully!");
        *self.periodic_task_context.write().await = Some(context);
    }
}
