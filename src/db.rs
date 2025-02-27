use anyhow::Result;
use sqlite::Connection;
use std::sync::{Arc, Mutex};

pub struct Chat {
    pub database: Arc<Mutex<Connection>>,
}

#[derive(Debug, PartialEq)]
struct UserPercent {
    user: String,
    percent: f32,
}

// Add a new struct to store message history
#[derive(Debug)]
pub struct MessageHistory {
    pub user: String,
    pub text: String,
}

impl Chat {
    pub fn new(db_path: String) -> Result<Self> {
        let conn = get_db(Some(db_path.as_str()))?;
        let database = Arc::new(Mutex::new(conn));

        Ok(Chat { database })
    }

    #[allow(dead_code)]
    pub fn in_memory() -> Result<Self> {
        let conn = get_db(None)?;
        let database = Arc::new(Mutex::new(conn));

        Ok(Chat { database })
    }

    pub fn store_msg(
        &self,
        chat_id: i64,
        msg_id: i32,
        username: &str,
        text: Option<&str>,
        time: i64,
        user_id: u64,
        raw: &str,
    ) -> Result<()> {
        let lock = self.database.lock().unwrap();
        let query =
            "INSERT INTO message_records(group_id, user, text, time, msg_id, user_id, raw) \
                    VALUES (?, ?, ?, ?, ?, ?, ?)";
        let mut stmt = lock.prepare(query)?;
        stmt.bind((1, chat_id))?;
        stmt.bind((2, username))?;
        stmt.bind((3, text))?;
        stmt.bind((4, time))?;
        stmt.bind((5, msg_id as i64))?;
        stmt.bind((6, user_id as i64))?;
        stmt.bind((7, raw))?;
        stmt.next()?;

        Ok(())
    }

    pub fn get_tot_msg(&self, chat_id: i64) -> Result<i64> {
        let lock = self.database.lock().unwrap();
        let query = "SELECT count(*) from message_records where group_id = ?";
        let mut stmt = lock.prepare(query)?;
        stmt.bind((1, chat_id))?;

        if let sqlite::State::Row = stmt.next()? {
            Ok(stmt.read::<i64, _>(0)?)
        } else {
            Ok(0)
        }
    }

    fn get_group_percent(&self, chat_id: i64) -> Result<Vec<UserPercent>> {
        let lock = self.database.lock().unwrap();
        let query = "SELECT user, count(*) * 100.0 / sum(count(*)) over () as percent \
                    FROM message_records \
                    WHERE group_id = ? \
                    GROUP BY group_id, user \
                    ORDER BY percent DESC";

        let mut stmt = lock.prepare(query)?;
        stmt.bind((1, chat_id))?;

        let mut results = Vec::new();
        while let Ok(sqlite::State::Row) = stmt.next() {
            results.push(UserPercent {
                user: stmt.read::<String, _>("user")?,
                percent: stmt.read::<f64, _>("percent")? as f32,
            });
        }

        Ok(results)
    }

    pub fn get_group_percent_str(&self, chat_id: i64) -> Result<String> {
        let data = self.get_group_percent(chat_id)?;

        let mut builder = String::new();
        for (index, data) in data.iter().enumerate() {
            if index == 0 {
                builder.push_str("🥇 ");
            } else if index == 1 {
                builder.push_str("🥈 ");
            } else if index == 2 {
                builder.push_str("🥉 ")
            }

            builder.push_str(format!("{} {:.2}%\n", data.user, data.percent).as_str());
        }
        builder.push_str("\n#stats");

        Ok(builder)
    }

    pub fn get_user_percent_str(&self, chat_id: i64, username: &String) -> Result<String> {
        let lock = self.database.lock().unwrap();
        let query = "SELECT mr.user, count(*) * 100.0 / ( \
                    SELECT count(*) FROM message_records WHERE group_id = mr.group_id \
                    ) as percent \
                    FROM message_records as mr \
                    WHERE mr.group_id = ? AND mr.user = ?";

        let mut stmt = lock.prepare(query)?;
        stmt.bind((1, chat_id))?;
        stmt.bind((2, username.as_str()))?;

        if let Ok(sqlite::State::Row) = stmt.next() {
            let percent = stmt.read::<f64, _>("percent")? as f32;
            Ok(format!("{:.2}%", percent))
        } else {
            Ok("0.00%".to_string())
        }
    }

    // Add method to get recent messages for context
    pub fn get_recent_messages(&self, chat_id: i64, limit: i64) -> Result<Vec<MessageHistory>> {
        let lock = self.database.lock().unwrap();
        let query = "SELECT user, text, time FROM message_records 
                     WHERE group_id = ? AND text IS NOT NULL 
                     ORDER BY time DESC LIMIT ?";

        let mut stmt = lock.prepare(query)?;
        stmt.bind((1, chat_id))?;
        stmt.bind((2, limit))?;

        let mut messages = Vec::new();
        while let Ok(sqlite::State::Row) = stmt.next() {
            let text = stmt.read::<String, _>("text")?;
            // Skip empty messages or those without text
            if !text.is_empty() {
                messages.push(MessageHistory {
                    user: stmt.read::<String, _>("user")?,
                    text,
                });
            }
        }

        // Reverse to get chronological order
        messages.reverse();
        Ok(messages)
    }
}

fn get_db(path: Option<&str>) -> Result<Connection> {
    let db = match path {
        Some(path) => Connection::open(path)?,
        None => Connection::open(":memory:")?,
    };

    // Set pragmas
    let pragmas = [
        "PRAGMA journal_mode = WAL",
        "PRAGMA synchronous = NORMAL",
        "PRAGMA cache_size = -2000",
        "PRAGMA temp_store = MEMORY",
        "PRAGMA mmap_size = 30000000000",
    ];

    for pragma in pragmas {
        db.execute(pragma)?;
    }

    run_migrations(&db)?;
    Ok(db)
}

fn run_migrations(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS message_records (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            group_id INTEGER NOT NULL,
            user TEXT NOT NULL,
            text TEXT,
            time INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
            msg_id INTEGER NOT NULL,
            user_id INTEGER NOT NULL,
            raw TEXT,
            UNIQUE(group_id, msg_id)
        );",
    )?;

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_setup() {
        let db = get_db(None);
        assert_eq!(db.is_ok(), true);
    }
}
