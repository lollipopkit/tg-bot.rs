use const_format::concatcp;

pub const DB_DIR: &str = ".db";

pub const GROUP_DB: &str = "group.db";
pub const GROUP_DB_FILE: &str = concatcp!(DB_DIR, "/", GROUP_DB);

pub const BOT_ID: &str = "lollipopkit_bot";