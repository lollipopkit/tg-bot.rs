use rand::Rng;
use std::{error::Error, sync::Arc};
use teloxide::{prelude::*, types::Me};

use crate::{
    ai::OpenAI,
    consts::{MAX_CONTEXT_MESSAGES, RANDOM_REPLY_CHANCE},
    db::Chat,
};

pub async fn handle_message(
    bot: Bot,
    msg: Message,
    cs: Arc<Chat>,
    ai: Arc<OpenAI>,
    me: Me,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let chat_id = msg.chat.id;

    // Store the message in the database
    let user = msg.from();
    let username = user.and_then(|u| u.username.clone()).unwrap_or_default();
    let user_id = user.map(|u| u.id.0).unwrap_or_default();
    let raw = serde_json::to_string(&msg)?;

    if let Some(text) = msg.text() {
        cs.store_msg(
            chat_id.0,
            msg.id.0,
            &username,
            Some(text),
            msg.date.timestamp(),
            user_id,
            &raw,
        )?;

        // Check if bot should respond
        let mentioned = is_bot_mentioned(&msg, &me.username.clone().unwrap_or_default());
        let should_respond = mentioned || (chat_id.0 < 0 && should_random_reply());

        if should_respond {
            // Get conversation context
            let context = prepare_context(chat_id.0, &cs, &username, text)?;

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
                    bot.send_message(chat_id, response).await?;
                }
                Err(e) => {
                    log::error!("Failed to generate AI response: {:?}", e);
                }
            }
        }
    } else {
        // Store non-text messages without triggering AI
        cs.store_msg(
            chat_id.0,
            msg.id.0,
            &username,
            None,
            msg.date.timestamp(),
            user_id,
            &raw,
        )?;
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
    cs: &Arc<Chat>,
    username: &str,
    current_msg: &str,
) -> Result<Vec<(String, String)>, Box<dyn Error + Send + Sync>> {
    // Start with system message
    let mut context = vec![];

    // Add recent message history for context
    let history = cs.get_recent_messages(chat_id, MAX_CONTEXT_MESSAGES)?;
    for msg in history {
        context.push((msg.user, msg.text));
    }

    // Add the current message
    context.push((username.to_string(), current_msg.to_string()));

    Ok(context)
}
