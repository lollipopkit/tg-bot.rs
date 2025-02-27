use const_format::concatcp;

pub const DB_DIR: &str = ".db";

pub const GROUP_DB: &str = "group.db";
pub const GROUP_DB_FILE: &str = concatcp!(DB_DIR, "/", GROUP_DB);

// OpenAI related constants

/// 5% chance to respond randomly
pub const RANDOM_REPLY_CHANCE: f32 = 0.05;
/// Number of previous messages to include for context
pub const MAX_CONTEXT_MESSAGES: i64 = 10;
