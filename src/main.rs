mod consts;
mod db;
mod handlers;

use anyhow::Result;
use consts::{DB_DIR, GROUP_DB_FILE};
use std::{env, sync::Arc};
use teloxide::prelude::*;
use tokio::fs;

use crate::{
    db::Chat,
    handlers::{commands, msgs},
};

#[tokio::main]
async fn main() -> Result<()> {
    init().await?;
    run().await?;

    Ok(())
}

async fn init() -> Result<()> {
    init_logger();
    fs::create_dir_all(DB_DIR).await?;

    Ok(())
}

fn init_logger() {
    let log_level = if cfg!(debug_assertions) {
        "debug"
    } else {
        "info"
    };

    let log_level = std::env::var("RUST_LOG").unwrap_or_else(|_| log_level.into());
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level)).init();
}

async fn run() -> Result<()> {
    let db_path = env::var("DB_PATH").unwrap_or(GROUP_DB_FILE.to_string());
    let chat_server = Arc::new(Chat::new(db_path)?);

    let bot = Bot::from_env();

    log::info!("Starting teloxide bot...");

    let handler = Update::filter_message()
        .branch(
            dptree::entry()
                .filter_command::<commands::Command>()
                .endpoint(commands::answer),
        )
        .branch(
            dptree::filter(|msg: Message| !msg.text().map_or(false, |text| text.starts_with('/')))
                .endpoint(msgs::handle_message),
        );

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![chat_server])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}
