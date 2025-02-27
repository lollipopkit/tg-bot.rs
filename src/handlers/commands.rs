use std::{error::Error, sync::Arc};
use teloxide::{prelude::*, types::Me, utils::command::BotCommands};

use crate::db::Chat;

#[derive(BotCommands, Clone, Debug)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
pub enum Command {
    #[command(description = "Show stats for the current group. (Only available in groups)")]
    GroupStats,
    #[command(description = "Show your Telegram user ID.")]
    Uid,
    #[command(description = "Check if the bot is running.")]
    Ping,
    #[command(description = "Display this help message.")]
    Help,
}

pub async fn answer(
    bot: Bot,
    msg: Message,
    cmd: Command,
    cs: Arc<Chat>,
    _me: Me,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let chat_id = msg.chat.id;

    // Telegram uses negative numbers for groups' `chat_id`
    let is_group = chat_id.0 < 0;

    let response = match (cmd, is_group) {
        (Command::GroupStats, true) => {
            let tot_msg = cs.get_tot_msg(chat_id.0)?;
            let group_percent = cs.get_group_percent_str(chat_id.0)?;
            format!("Total: {}\n{}", tot_msg, group_percent)
        }
        (Command::Uid, _) => {
            let user = msg.from();
            let username = user.and_then(|u| u.username.clone()).unwrap_or_default();
            let user_id = user.map(|u| u.id.0).unwrap_or_default();
            format!("Username: {}\nUser ID: {}", username, user_id)
        }
        (Command::Ping, _) => "Pong".to_string(),
        (Command::Help, _) => Command::descriptions().to_string(),
        (_, false) => "This command is only available in groups.".to_string(),
    };

    if !response.is_empty() {
        bot.send_message(chat_id, response).await?;
    }

    Ok(())
}
