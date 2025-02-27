-- Create initial database schema

CREATE TABLE IF NOT EXISTS message_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    group_id INTEGER NOT NULL,
    user TEXT NOT NULL,
    text TEXT,
    time INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    msg_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    raw TEXT,
    UNIQUE(group_id, msg_id)
);
