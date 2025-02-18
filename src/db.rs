use rusqlite::{Connection, Result};

pub fn get_db(path: Option<&str>) -> Result<Connection> {
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
            message_id INTEGER NOT NULL,
            username TEXT NOT NULL,
            message_text TEXT,
            message_time INTEGER NOT NULL,
            utime INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
            UNIQUE(group_id, message_id)
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
