use std::{error::Error, sync::Arc};
use teloxide::prelude::*;

use crate::db::Chat;

pub async fn handle_message(
    _bot: Bot,
    msg: Message,
    cs: Arc<Chat>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let chat_id = msg.chat.id;

    // Telegram uses negative numbers for groups' `chat_id`
    if chat_id.0 > 0 {
        return Ok(());
    }

    log::debug!("Processing non-command message: {:?}", msg);

    // Process message for stats/storage
    let user = msg.from();
    let username = user.and_then(|u| u.username.clone()).unwrap_or_default();
    let user_id = user.map(|u| u.id.0).unwrap_or_default();
    let raw = serde_json::to_string(&msg)?;
    cs.store_msg(
        chat_id.0,
        msg.id.0,
        &username,
        msg.text(),
        msg.date.timestamp(),
        user_id,
        &raw,
    )?;

    Ok(())
}
