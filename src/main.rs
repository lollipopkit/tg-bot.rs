mod chat_server;
mod db;
pub mod handler;

use std::{env, sync::Arc};
use teloxide::prelude::*;
use tokio::fs;

use crate::{chat_server::ChatServer, handler::handle};

#[tokio::main]
async fn main() {
    log::info!("Starting...");

    init().await;
    run().await;

    log::info!("Goodbye!");
}

async fn init() {
    fs::create_dir_all(".db").await.unwrap();
}

async fn run() {
    let db_path = env::var("DB_PATH").unwrap_or(".db/group.db".to_string());

    let chat_server = Arc::new(ChatServer::new(db_path));

    let handler = dptree::entry().branch(Update::filter_message().endpoint(handle));

    let bot = Bot::from_env().auto_send();
    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![chat_server])
        .build()
        .setup_ctrlc_handler()
        .dispatch()
        .await;
}
