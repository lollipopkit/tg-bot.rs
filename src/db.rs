use anyhow::Result;
use rusqlite::{params, Connection};
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct Chat {
    pub database: Arc<Mutex<Connection>>,
}

#[derive(Debug, PartialEq)]
struct UserPercent {
    user: String,
    percent: f32,
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
        let mut stmt = lock.prepare(
            "INSERT INTO message_records(
                group_id, user, text, time, msg_id, user_id, raw
            ) VALUES (?, ?, ?, ?, ?, ?, ?)",
        )?;
        stmt.execute(params![chat_id, username, text, time, msg_id, user_id, raw])?;

        Ok(())
    }

    pub fn get_tot_msg(&self, chat_id: i64) -> Result<i64> {
        let lock = self.database.lock().unwrap();
        let mut stmt = lock.prepare(
            "SELECT count(*)
            from message_records
            where group_id = ?;",
        )?;

        let tot = stmt.query_row([chat_id], |row| Ok(row.get(0)?)).unwrap();

        Ok(tot)
    }

    fn get_group_percent(&self, chat_id: i64) -> Result<Vec<UserPercent>> {
        let lock = self.database.lock().unwrap();
        let mut stmt = lock.prepare(
            "SELECT user, count(*) * 100.0 / sum(count(*)) over () as percent
             FROM message_records
             WHERE group_id = ?
             GROUP BY group_id, user
             ORDER BY percent DESC;",
        )?;

        let percents_iter = stmt
            .query_map([chat_id], |row| {
                Ok(UserPercent {
                    user: row.get(0)?,
                    percent: row.get(1)?,
                })
            })
            .unwrap();

        let perc_vec: Vec<UserPercent> = percents_iter.map(|d| d.unwrap()).collect();

        Ok(perc_vec)
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
        builder.push_str(format!("\n#stats").as_str());

        Ok(builder)
    }

    pub fn get_user_percent_str(&self, chat_id: i64, username: &String) -> Result<String> {
        let lock = self.database.lock().unwrap();
        let mut stmt = lock.prepare(
            "SELECT mr.user, count(*) * 100.0 / (
                SELECT count(*) FROM message_records WHERE group_id = mr.group_id
            ) as percent
             FROM message_records as mr
             WHERE mr.group_id = ? AND mr.user = ?;",
        )?;

        let data = stmt.query_row(params![chat_id, username], |row| {
            Ok(UserPercent {
                user: row.get(0)?,
                percent: row.get(1)?,
            })
        })?;

        Ok(format!("{:.2}%", data.percent))
    }
}

fn get_db(path: Option<&str>) -> Result<Connection> {
    let db = match path {
        Some(path) => {
            let path = path;
            Connection::open(&path)?
        }
        None => Connection::open_in_memory()?,
    };

    db.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;
         PRAGMA cache_size = -2000;
         PRAGMA temp_store = MEMORY;
         PRAGMA mmap_size = 30000000000;",
    )?;

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
        [],
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
