use std::{error::Error, sync::Arc};
use teloxide::{prelude::*, utils::command::BotCommands};

use crate::{consts::BOT_ID, db::Chat};

#[derive(BotCommands, PartialEq, Debug)]
enum Command {
    #[command(rename = "lowercase")]
    GroupStats,
    #[command(rename = "lowercase")]
    UserStats(String),
}

pub async fn handle(
    bot: Bot,
    m: Message,
    cs: Arc<Chat>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let chat_id = m.chat.id;

    // Telegram uses negative numbers for groups' `chat_id`
    if chat_id.0 > 0 {
        return Ok(());
    }

    log::debug!("Received message: {:?}", m);

    if let Some(text) = m.text() {
        if let Ok(command) = Command::parse(text, BOT_ID) {
            handle_cmd(&bot, &m, cs.clone(), command).await?;
        }
    };

    handle_non_cmd(&bot, &m, cs.clone()).await?;

    Ok(())
}

async fn handle_cmd(
    bot: &Bot,
    m: &Message,
    cs: Arc<Chat>,
    cmd: Command,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let chat_id = m.chat.id;

    let response = match cmd {
        Command::GroupStats => {
            format!(
                "Total: {}\n{}",
                cs.get_tot_msg(chat_id.0)?,
                cs.get_group_percent_str(chat_id.0)?
            )
        }
        Command::UserStats(username) => cs.get_user_percent_str(chat_id.0, &username)?,
    };

    if !response.is_empty() {
        bot.send_message(chat_id, response).await?;
    }

    Ok(())
}

async fn handle_non_cmd(
    bot: &Bot,
    m: &Message,
    cs: Arc<Chat>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let chat_id = m.chat.id;
    let resp = String::new();

    let user = m.from();
    let username = user
        .map(|u| u.username.clone())
        .flatten()
        .unwrap_or_default();
    let user_id = user.map(|u| u.id.0).unwrap_or_default();
    let raw = serde_json::to_string(&m)?;
    cs.store_msg(
        chat_id.0,
        m.id.0,
        &username,
        m.text(),
        m.date.timestamp(),
        user_id,
        &raw,
    )?;

    if !resp.is_empty() {
        bot.send_message(chat_id, resp).await?;
    }

    Ok(())
}
