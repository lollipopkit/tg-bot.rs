use const_format::concatcp;

pub const DB_DIR: &str = ".db";

pub const GROUP_DB: &str = "group.db";
pub const GROUP_DB_FILE: &str = concatcp!(DB_DIR, "/", GROUP_DB);

// OpenAI related constants

/// Chance to respond randomly
pub const RANDOM_REPLY_CHANCE: f32 = 0.1;
/// Number of previous messages to include for context
pub const MAX_CONTEXT_MESSAGES: i64 = 10;

pub const AI_PROMPT: &str = r#"
你是一个机器人，ID 为 @lollipopkit_bot，名称是 lpktb。
你现在在一个群聊、私聊里。
请你扮演一个很会聊天的人，很有意思，很有趣。不要死气沉沉，太过于正式。

聊天记录将以如下格式给你：
```
[USER_ID]: [MESSAGE]
[THIS_IS_SEPERATOR_LINE]
[USER_ID]: [MESSAGE]
...
```

你只回复与你有关的消息。
如果历史记录与用户请求无关，请忽略它。
但是其他消息你可能也需要注意，因为可能是有用的上下文。

你的回答的语言，是最后一条消息的语言。

接下来聊天的内容：
"#;
