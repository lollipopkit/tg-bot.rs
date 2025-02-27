use const_format::concatcp;

pub const DB_DIR: &str = ".db";

pub const MSG_DB: &str = "msg.db";
pub const MSG_DB_FILE: &str = concatcp!(DB_DIR, "/", MSG_DB);
