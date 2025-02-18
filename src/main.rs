mod chat_server;
mod db;
pub mod handler;

use std::{env, sync::Arc};
use teloxide::prelude::*;
use tokio::fs;

use crate::{chat_server::ChatServer, handler::handle};

#[tokio::main]
async fn main() {
    init().await;
    run().await;
}

async fn init() {
    let log_level = env::var("RUST_LOG").unwrap_or(
        cfg!(debug_assertions)
            .then(|| "debug")
            .unwrap_or("info")
            .to_string(),
    );
    pretty_env_logger::formatted_builder()
        .filter_level(log::LevelFilter::Info)
        .parse_filters(&log_level)
        .init();

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
