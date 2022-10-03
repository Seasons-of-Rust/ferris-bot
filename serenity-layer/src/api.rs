use crate::{
    // command_history::CommandHistory,
    commands::{Args, Auth},
    Error,
};
use indexmap::IndexMap;
use serenity::{model::prelude::*, utils::parse_username};
use std::sync::Arc;
use tracing::info;

/// Send a reply to the channel the message was received on.  
// pub async fn send_reply(args: Arc<Args>, message: &str) -> Result<(), Error> {
//     if let Some(response_id) = response_exists(args.clone()).await {
//         info!("editing message: {:?}", response_id);
//         args.msg
//             .channel_id
//             .edit_message(&args.clone().cx, response_id, |msg| msg.content(message))
//             .await?;
//     } else {
//         let command_id = args.msg.id;
//         let response = args.clone().msg.channel_id.say(&args.cx, message).await?;

//         let mut data = args.cx.data.write().await;
//     }

//     Ok(())
// }

// async fn response_exists(args: Arc<Args>) -> Option<MessageId> {
//     let data = args.cx.data.read().await;
// }

/// Determine if a member sending a message has the `Role`.  
pub fn has_role(args: Arc<Args>, role: &RoleId) -> Result<bool, Error> {
    Ok(args
        .msg
        .member
        .as_ref()
        .ok_or("Unable to fetch member")?
        .roles
        .contains(role))
}

fn check_permission(args: Arc<Args>, role: Option<String>) -> Result<bool, Error> {
    use std::str::FromStr;
    if let Some(role_id) = role {
        Ok(has_role(
            args.clone(),
            &RoleId::from(u64::from_str(&role_id)?),
        )?)
    } else {
        Ok(false)
    }
}
