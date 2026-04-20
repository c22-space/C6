use rusqlite::{Connection, Result, params};
use std::path::Path;
use std::sync::Mutex;

pub struct Database(pub Mutex<Connection>);

impl Database {
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        // Enable WAL mode for better concurrent read performance
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON; PRAGMA busy_timeout=5000;")?;
        Ok(Self(Mutex::new(conn)))
    }

    pub fn migrate(&self) -> Result<()> {
        let conn = self.0.lock().unwrap();

        // Create migration tracking table if not exists
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS _migrations (
                version INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                applied_at INTEGER NOT NULL DEFAULT (unixepoch())
            );"
        )?;

        let applied: Vec<i64> = {
            let mut stmt = conn.prepare("SELECT version FROM _migrations ORDER BY version")?;
            let rows = stmt.query_map([], |r| r.get(0))?;
            rows.collect::<Result<Vec<_>>>()?
        };

        let migrations: &[(&str, &str)] = &[
            ("001_init", include_str!("../migrations/001_init.sql")),
            ("002_seed", include_str!("../migrations/002_seed.sql")),
        ];

        for (idx, (name, sql)) in migrations.iter().enumerate() {
            let version = (idx + 1) as i64;
            if applied.contains(&version) {
                continue;
            }
            conn.execute_batch(sql)?;
            conn.execute(
                "INSERT INTO _migrations (version, name) VALUES (?1, ?2)",
                params![version, name],
            )?;
        }

        Ok(())
    }
}
