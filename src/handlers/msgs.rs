use rand::Rng;
use std::{error::Error, sync::Arc};
use teloxide::{prelude::*, types::Me};

use crate::{
    ai::OpenAI,
    consts::{MAX_CONTEXT_MESSAGES, RANDOM_REPLY_CHANCE},
    db::Db,
};

pub async fn handle_message(
    bot: Bot,
    msg: Message,
    cs: Arc<Db>,
    ai: Arc<OpenAI>,
    me: Me,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let chat_id = msg.chat.id;

    // Store the message in the database
    let user = msg.from();
    let username = user.and_then(|u| u.username.clone()).unwrap_or_default();
    let user_id = user.map(|u| u.id.0).unwrap_or_default();
    let raw = serde_json::to_string(&msg)?;
    let text = msg.text();

    cs.store_msg(
        chat_id.0,
        msg.id.0,
        &username,
        text,
        msg.date.timestamp(),
        user_id,
        &raw,
    )?;

    if let None = text {
        return Ok(());
    }

    // Check if bot should respond
    let mentioned = is_bot_mentioned(&msg, &me.username.clone().unwrap_or_default());
    let is_private = chat_id.0 > 0;
    let should_respond = mentioned || (chat_id.0 < 0 && should_random_reply()) || is_private;

    if !should_respond {
        return Ok(());
    }

    // Get conversation context
    let context = prepare_context(chat_id.0, &cs)?;

    // If bot is mentioned, use set it to the caller, or use the last message's user
    // let prompter = if mentioned {
    //     username.clone()
    // } else {
    //     context
    //         .iter()
    //         .last()
    //         .map(|(role, _)| role.clone())
    //         .unwrap_or("user".to_string())
    // };

    // Generate AI response
    match ai.generate_response(context).await {
        Ok(response) => {
            let sent_msg = bot.send_message(chat_id, &response).await?;
            cs.store_msg(
                chat_id.0,
                sent_msg.id.0,
                &me.username.clone().unwrap_or_default(),
                Some(&response),
                sent_msg.date.timestamp(),
                me.id.0,
                &serde_json::to_string(&sent_msg)?,
            )?;
        }
        Err(e) => {
            log::error!("Failed to generate AI response: {:?}", e);
        }
    }

    Ok(())
}

fn is_bot_mentioned(msg: &Message, bot_username: &str) -> bool {
    if let Some(text) = msg.text() {
        // Check for direct mention with @ syntax
        if text.contains(&format!("@{}", bot_username)) {
            return true;
        }

        // Check if message is a reply to the bot
        if let Some(reply) = msg.reply_to_message() {
            if let Some(user) = reply.from() {
                return user.is_bot && user.username == Some(bot_username.to_string());
            }
        }
    }
    false
}

fn should_random_reply() -> bool {
    let mut rng = rand::rng();
    rng.random::<f32>() < RANDOM_REPLY_CHANCE
}

fn prepare_context(
    chat_id: i64,
    cs: &Arc<Db>,
) -> Result<Vec<(String, String)>, Box<dyn Error + Send + Sync>> {
    // Start with system message
    let mut context = vec![];

    // Add recent message history for context
    let history = cs.get_recent_messages(chat_id, MAX_CONTEXT_MESSAGES)?;
    let ids = history.iter().map(|msg| msg.user_id).collect::<Vec<_>>();
    let id_name_map = cs.get_name_id_map(ids);
    for msg in history {
        let name = id_name_map.get(&msg.user_id).unwrap_or(&msg.user);
        context.push((name.to_string(), msg.text));
    }

    // Since this fn is called after storing the current message, there's no need to add it
    Ok(context)
}
