use anyhow::Result;
use sqlx::sqlite::SqlitePool;
use std::{collections::HashMap, sync::Arc};

pub struct Db {
    pub pool: SqlitePool,
}

#[derive(Debug, PartialEq)]
struct UserPercent {
    user: String,
    user_id: i64,
    percent: f64,
}

// Add a new struct to store message history
#[derive(Debug)]
pub struct MessageHistory {
    pub user: String,
    pub user_id: i64,
    pub text: String,
}

impl Db {
    pub async fn new(db_path: String) -> Result<Arc<Self>> {
        // Create the database URL with proper formatting
        let db_url = format!("sqlite:{}", db_path);

        // Run migrations before connecting
        sqlx::migrate!("./migrations")
            .run(&SqlitePool::connect(&db_url).await?)
            .await?;

        // Create the connection pool
        let pool = SqlitePool::connect(&db_url).await?;

        // Set pragmas
        sqlx::query("PRAGMA journal_mode = WAL")
            .execute(&pool)
            .await?;
        sqlx::query("PRAGMA synchronous = NORMAL")
            .execute(&pool)
            .await?;
        sqlx::query("PRAGMA cache_size = -2000")
            .execute(&pool)
            .await?;
        sqlx::query("PRAGMA temp_store = MEMORY")
            .execute(&pool)
            .await?;
        sqlx::query("PRAGMA mmap_size = 30000000000")
            .execute(&pool)
            .await?;

        Ok(Arc::new(Db { pool }))
    }

    #[allow(dead_code)]
    pub async fn in_memory() -> Result<Self> {
        let pool = SqlitePool::connect("sqlite::memory:").await?;

        // Run migrations on the in-memory database
        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(Db { pool })
    }

    pub async fn store_msg(
        &self,
        chat_id: i64,
        msg_id: i32,
        username: &str,
        text: Option<&str>,
        time: i64,
        user_id: u64,
        raw: &str,
    ) -> Result<()> {
        let msg_id = msg_id as i64;
        let user_id = user_id as i64;
        sqlx::query!(
            r#"
            INSERT INTO message_records(group_id, user, text, time, msg_id, user_id, raw) 
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
            chat_id,
            username,
            text,
            time,
            msg_id,
            user_id,
            raw
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_tot_msg(&self, chat_id: i64) -> Result<i32> {
        let count = sqlx::query!(
            r#"
            SELECT COUNT(*) as count FROM message_records WHERE group_id = ?
            "#,
            chat_id
        )
        .fetch_one(&self.pool)
        .await?
        .count;

        Ok(count)
    }

    async fn get_group_percent(&self, chat_id: i64) -> Result<Vec<UserPercent>> {
        let rows = sqlx::query!(
            r#"
            SELECT user, user_id, COUNT(*) * 100.0 / SUM(COUNT(*)) OVER () as percent 
            FROM message_records 
            WHERE group_id = ? AND user_id != 0 
            GROUP BY group_id, user_id 
            ORDER BY percent DESC
            "#,
            chat_id
        )
        .fetch_all(&self.pool)
        .await?;

        let results = rows
            .into_iter()
            .map(|row| UserPercent {
                user: row.user,
                user_id: row.user_id,
                percent: row.percent.unwrap_or_default(),
            })
            .collect();

        Ok(results)
    }

    pub async fn get_group_percent_str(&self, chat_id: i64) -> Result<String> {
        let data = self.get_group_percent(chat_id).await?;

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
    pub async fn get_recent_messages(
        &self,
        chat_id: i64,
        limit: i64,
    ) -> Result<Vec<MessageHistory>> {
        let rows = sqlx::query!(
            r#"
            SELECT user, user_id, text, time FROM message_records 
            WHERE group_id = ? AND text IS NOT NULL AND text != ''
            ORDER BY time DESC LIMIT ?
            "#,
            chat_id,
            limit
        )
        .fetch_all(&self.pool)
        .await?;

        let mut messages: Vec<MessageHistory> = rows
            .into_iter()
            .filter_map(|row| match row.text {
                Some(text) if !text.is_empty() => Some(MessageHistory {
                    user: row.user,
                    user_id: row.user_id,
                    text,
                }),
                _ => None,
            })
            .collect();

        // Reverse to get chronological order
        messages.reverse();
        Ok(messages)
    }

    /// Get latest user ID and name mapping
    pub async fn get_name_id_map(&self, ids: Vec<i64>) -> Result<HashMap<i64, String>> {
        let mut result = HashMap::new();

        if ids.is_empty() {
            return Ok(result);
        }

        // SQLx doesn't directly support dynamic IN clauses, so we need to handle ids differently
        // For simplicity, we'll query for each ID separately
        for id in ids {
            let row = sqlx::query!(
                r#"
                SELECT user, user_id FROM message_records 
                WHERE user_id = ? 
                ORDER BY time DESC LIMIT 1
                "#,
                id
            )
            .fetch_optional(&self.pool)
            .await?;

            if let Some(row) = row {
                result.insert(row.user_id, row.user);
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_setup() {
        let db = Db::in_memory().await;
        assert!(db.is_ok());
    }
}
