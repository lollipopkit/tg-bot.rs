use rusqlite::{Connection, Result};

pub fn get_db(path: Option<&str>) -> Result<Connection> {
    let db = match path {
        Some(path) => {
            let path = path;
            Connection::open(&path)?
        }
        None => {
            Connection::open_in_memory()?
        }
    };
    run_migrations(&db)?;
    Ok(db)
}

fn run_migrations(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS message_records (
            group_id INTEGER NOT NULL,
            username TEXT NOT NULL,
            utime    INTEGER NOT NULL DEFAULT (strftime('%s', 'now')) 
        );", [],
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