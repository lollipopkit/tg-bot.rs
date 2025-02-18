use std::{error::Error, sync::Arc};
use teloxide::{
    prelude::*,
    types::{MediaKind, MessageKind::Common},
    utils::command::BotCommands,
};

use crate::chat_server::ChatServer;

#[derive(BotCommands, PartialEq, Debug)]
#[command(rename = "lowercase")]
enum Command {
    GroupStats,
    UserStats(String),
}

pub async fn handle(
    bot: AutoSend<Bot>,
    m: Message,
    cs: Arc<ChatServer>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let chat_id = m.chat.id.0;

    // Telegram uses negative numbers for groups' `chat_id`
    if chat_id > 0 {
        // bot.send_message(m.chat.id, "This bot is only useful in groups.")
        //     .await?;
        return Ok(());
    }

    log::debug!("Received message: {:?}", m);

    let text = match m.text() {
        Some(text) => text,
        None => {
            return Ok(());
        }
    };

    let mut response = String::from("");
    if let Ok(command) = Command::parse(text, "lollipopkit_bot") {
        response = match command {
            Command::GroupStats => {
                format!(
                    "Total: {}\n{}",
                    cs.get_tot_msg(chat_id)?,
                    cs.get_group_percent_str(chat_id)?
                )
            }
            Command::UserStats(username) => cs.get_user_percent_str(chat_id, &username)?,
        }
    } else {
        match &m.kind {
            Common(common_msg) => {
                if let Some(user) = &common_msg.from {
                    if let Some(username) = &user.username {
                        let (media_type, file_id, file_unique_id, emoji) =
                            match &common_msg.media_kind {
                                MediaKind::Sticker(sticker) => (
                                    Some("sticker"),
                                    Some(sticker.sticker.file_id.as_str()),
                                    Some(sticker.sticker.file_unique_id.as_str()),
                                    sticker.sticker.emoji.as_deref(),
                                ),
                                MediaKind::Photo(photo) => {
                                    let photo = photo.photo.last().unwrap();
                                    (
                                        Some("photo"),
                                        Some(photo.file_id.as_str()),
                                        Some(photo.file_unique_id.as_str()),
                                        None,
                                    )
                                }
                                MediaKind::Video(video) => (
                                    Some("video"),
                                    Some(video.video.file_id.as_str()),
                                    Some(video.video.file_unique_id.as_str()),
                                    None,
                                ),
                                _ => (None, None, None, None),
                            };

                        cs.store_msg(
                            chat_id,
                            m.id,
                            username,
                            m.text(),
                            m.date.timestamp(),
                            media_type,
                            file_id,
                            file_unique_id,
                            emoji,
                        )?;
                    }
                }
            }
            _ => {}
        }
    }

    if !response.is_empty() {
        bot.send_message(m.chat.id, response).await?;
    }

    Ok(())
}
