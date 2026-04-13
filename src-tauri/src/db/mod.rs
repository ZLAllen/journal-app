use crate::models::Result;
use rusqlite::Connection;

pub mod schema;

fn is_malformed_error(error: &rusqlite::Error) -> bool {
    match error {
        rusqlite::Error::SqliteFailure(code, message) => {
            if code.extended_code == rusqlite::ffi::SQLITE_CORRUPT
                || code.extended_code == rusqlite::ffi::SQLITE_CORRUPT_VTAB
            {
                return true;
            }

            message.as_deref().is_some_and(|details| {
                let lower = details.to_ascii_lowercase();
                lower.contains("malformed") || lower.contains("corrupt")
            })
        }
        _ => false,
    }
}

/// Database connection wrapper
pub struct DbConnection {
    conn: Connection,
}

impl DbConnection {
    /// Create or open a database at the specified path
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;

        // Enable foreign keys
        conn.execute_batch("PRAGMA foreign_keys = ON")?;

        let db = DbConnection { conn };
        db.run_migrations()?;
        db.repair_search_index_if_needed()?;
        db.validate_integrity()?;
        Ok(db)
    }

    /// Create in-memory database for testing
    pub fn new_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys = ON")?;

        let db = DbConnection { conn };
        db.run_migrations()?;
        db.repair_search_index_if_needed()?;
        db.validate_integrity()?;
        Ok(db)
    }

    /// Run all migrations on the database
    fn run_migrations(&self) -> Result<()> {
        // Create migrations table if it doesn't exist
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS migrations (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                applied_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        // Apply each migration
        let migrations = schema::get_migrations();
        for migration in migrations {
            let mut stmt = self
                .conn
                .prepare("SELECT id FROM migrations WHERE name = ?")?;

            let already_applied = stmt.exists([migration.name])?;

            if !already_applied {
                // Run the migration
                self.conn.execute_batch(migration.sql)?;

                // Record that it was applied
                self.conn.execute(
                    "INSERT INTO migrations (name) VALUES (?1)",
                    [migration.name],
                )?;
            }
        }

        Ok(())
    }

    fn validate_integrity(&self) -> Result<()> {
        let quick_check: String = self
            .conn
            .query_row("PRAGMA quick_check", [], |row| row.get(0))?;

        if quick_check.to_lowercase() != "ok" {
            return Err(crate::models::AppError::CorruptDatabase(quick_check));
        }

        self.validate_search_index()?;

        Ok(())
    }

    fn validate_search_index(&self) -> Result<()> {
        self.conn.execute(
            "INSERT INTO entries_fts(entries_fts) VALUES('integrity-check')",
            [],
        )?;

        Ok(())
    }

    fn rebuild_search_index(&self) -> Result<()> {
        self.conn
            .execute("INSERT INTO entries_fts(entries_fts) VALUES('rebuild')", [])?;
        self.validate_search_index()
    }

    fn repair_search_index_if_needed(&self) -> Result<()> {
        match self.validate_search_index() {
            Ok(()) => Ok(()),
            Err(crate::models::AppError::Database(error)) if is_malformed_error(&error) => {
                self.rebuild_search_index()
            }
            Err(error) => Err(error),
        }
    }

    pub fn run_with_search_index_repair<T, F>(&self, operation: F) -> Result<T>
    where
        F: Fn(&Connection) -> Result<T>,
    {
        match operation(&self.conn) {
            Ok(value) => Ok(value),
            Err(crate::models::AppError::Database(error)) if is_malformed_error(&error) => {
                self.rebuild_search_index()?;
                operation(&self.conn)
            }
            Err(error) => Err(error),
        }
    }

    /// Get the underlying connection reference
    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    /// Get the underlying connection as mutable reference
    pub fn conn_mut(&mut self) -> &mut Connection {
        &mut self.conn
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_memory_database() {
        let db = DbConnection::new_memory().expect("Failed to create in-memory DB");

        // Verify that migrations table exists
        let result = db.conn().query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='migrations'",
            [],
            |row| row.get::<_, i32>(0),
        );

        assert!(result.is_ok(), "Migrations table should exist");
    }

    #[test]
    fn test_migrations_applied_once() {
        let db = DbConnection::new_memory().expect("Failed to create in-memory DB");

        let migration_count: i32 = db
            .conn()
            .query_row("SELECT COUNT(*) FROM migrations", [], |row| row.get(0))
            .expect("Failed to count migrations");

        assert!(
            migration_count > 0,
            "At least one migration should be applied"
        );

        // Running migrations again should not duplicate entries
        db.run_migrations()
            .expect("Second migration run should succeed");

        let migration_count_after: i32 = db
            .conn()
            .query_row("SELECT COUNT(*) FROM migrations", [], |row| row.get(0))
            .expect("Failed to count migrations");

        assert_eq!(
            migration_count, migration_count_after,
            "Migration count should not change on second run"
        );
    }

    #[test]
    fn test_schema_tables_exist() {
        let db = DbConnection::new_memory().expect("Failed to create in-memory DB");

        // Check for entries table
        let entries_exist: i32 = db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='entries'",
                [],
                |row| row.get(0),
            )
            .expect("Failed to check entries table");

        assert_eq!(entries_exist, 1, "Entries table should exist");

        // Check for tags table
        let tags_exist: i32 = db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='tags'",
                [],
                |row| row.get(0),
            )
            .expect("Failed to check tags table");

        assert_eq!(tags_exist, 1, "Tags table should exist");

        // Check for entry_tags table
        let entry_tags_exist: i32 = db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='entry_tags'",
                [],
                |row| row.get(0),
            )
            .expect("Failed to check entry_tags table");

        assert_eq!(entry_tags_exist, 1, "entry_tags table should exist");
    }

    #[test]
    fn test_search_index_triggers_exist() {
        let db = DbConnection::new_memory().expect("Failed to create in-memory DB");

        let trigger_count: i32 = db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='trigger' AND name LIKE 'entries_%_fts'",
                [],
                |row| row.get(0),
            )
            .expect("Failed to check FTS triggers");

        assert_eq!(trigger_count, 3, "FTS maintenance triggers should exist");
    }
}
