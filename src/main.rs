mod ai;
mod consts;
mod db;
mod handlers;

use anyhow::Result;
use consts::{DB_DIR, GROUP_DB_FILE};
use teloxide::prelude::*;

use crate::{
    ai::OpenAI,
    db::Db,
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
    tokio::fs::create_dir_all(DB_DIR).await?;

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
    let db_path = std::env::var("DB_PATH").unwrap_or(GROUP_DB_FILE.to_string());
    let chat_db = Db::new(db_path)?;
    let openai = OpenAI::new()?;

    let bot = Bot::from_env();
    let me = bot.get_me().await?;
    log::info!("Starting teloxide bot as @{}", me.username());

    let cmds_branch = dptree::entry()
        .filter_command::<commands::Command>()
        .endpoint(commands::answer);
    // Filter out messages starting with '/'.
    // If this msg has no text, maybe it's a system msg, record it anyway
    let msgs_branch =
        dptree::filter(|msg: Message| msg.text().map_or(true, |text| !text.starts_with('/')))
            .endpoint(msgs::handle_message);

    let handler = Update::filter_message()
        .branch(cmds_branch)
        .branch(msgs_branch);

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![chat_db, openai, me])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}
