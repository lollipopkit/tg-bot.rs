use anyhow::Result;
use sqlite::Connection;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

pub struct Chat {
    pub database: Arc<Mutex<Connection>>,
}

#[derive(Debug, PartialEq)]
struct UserPercent {
    user: String,
    user_id: i64,
    percent: f32,
}

// Add a new struct to store message history
#[derive(Debug)]
pub struct MessageHistory {
    pub user: String,
    pub user_id: i64,
    pub text: String,
}

impl Chat {
    pub fn new(db_path: String) -> Result<Arc<Self>> {
        let conn = get_db(Some(db_path.as_str()))?;
        let database = Arc::new(Mutex::new(conn));

        Ok(Arc::new(Chat { database }))
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
        let query = "SELECT user, user_id, count(*) * 100.0 / sum(count(*)) over () as percent \
                    FROM message_records \
                    WHERE group_id = ? AND user_id != 0 \
                    GROUP BY group_id, user_id \
                    ORDER BY percent DESC";

        let mut stmt = lock.prepare(query)?;
        stmt.bind((1, chat_id))?;

        let mut results = Vec::new();
        while let Ok(sqlite::State::Row) = stmt.next() {
            results.push(UserPercent {
                user: stmt.read::<String, _>("user")?,
                user_id: stmt.read::<i64, _>("user_id")?,
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

    // Add method to get recent messages for context
    pub fn get_recent_messages(&self, chat_id: i64, limit: i64) -> Result<Vec<MessageHistory>> {
        let lock = self.database.lock().unwrap();
        let query = "SELECT user, user_id, text, time FROM message_records 
                     WHERE group_id = ? AND text IS NOT NULL AND text != ''
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
                    user_id: stmt.read::<i64, _>("user_id")?,
                    text,
                });
            }
        }

        // Reverse to get chronological order
        messages.reverse();
        Ok(messages)
    }

    /// Get latest user ID and name mapping
    ///
    /// ```
    /// user_id1 username1 msg
    /// user_id2 username2 msg
    /// user_id1 username3 msg
    /// ```
    ///
    /// Returns:
    /// ```json
    /// {user_id1: "username3", user_id2: "username2"}
    /// ```
    pub fn get_name_id_map(&self, ids: Vec<i64>) -> HashMap<i64, String> {
        let mut result = HashMap::new();

        if ids.is_empty() {
            return result;
        }

        // Create parameters for SQL IN clause
        let params = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");

        let query = format!(
            "SELECT user, user_id FROM message_records 
             WHERE user_id IN ({}) 
             ORDER BY time DESC",
            params
        );

        if let Ok(lock) = self.database.lock() {
            if let Ok(mut stmt) = lock.prepare(&query) {
                // Bind all parameters
                for (i, id) in ids.iter().enumerate() {
                    if let Err(_) = stmt.bind((i + 1, *id)) {
                        continue;
                    }
                }

                // Process results
                while let Ok(sqlite::State::Row) = stmt.next() {
                    if let (Ok(username), Ok(user_id)) = (
                        stmt.read::<String, _>("user"),
                        stmt.read::<i64, _>("user_id"),
                    ) {
                        // Only insert if this username is not already mapped
                        // This keeps the most recent username for each user ID
                        if !result.contains_key(&user_id) {
                            result.insert(user_id, username);
                        }
                    }
                }
            }
        }

        result
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
