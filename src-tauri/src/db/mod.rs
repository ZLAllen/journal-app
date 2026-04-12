use crate::models::Result;
use rusqlite::Connection;

pub mod schema;

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
        Ok(db)
    }

    /// Create in-memory database for testing
    pub fn new_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys = ON")?;
        
        let db = DbConnection { conn };
        db.run_migrations()?;
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
            let mut stmt = self.conn.prepare(
                "SELECT id FROM migrations WHERE name = ?"
            )?;
            
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
        
        let migration_count: i32 = db.conn().query_row(
            "SELECT COUNT(*) FROM migrations",
            [],
            |row| row.get(0),
        ).expect("Failed to count migrations");
        
        assert!(migration_count > 0, "At least one migration should be applied");
        
        // Running migrations again should not duplicate entries
        db.run_migrations().expect("Second migration run should succeed");
        
        let migration_count_after: i32 = db.conn().query_row(
            "SELECT COUNT(*) FROM migrations",
            [],
            |row| row.get(0),
        ).expect("Failed to count migrations");
        
        assert_eq!(migration_count, migration_count_after, "Migration count should not change on second run");
    }

    #[test]
    fn test_schema_tables_exist() {
        let db = DbConnection::new_memory().expect("Failed to create in-memory DB");
        
        // Check for entries table
        let entries_exist: i32 = db.conn().query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='entries'",
            [],
            |row| row.get(0),
        ).expect("Failed to check entries table");
        
        assert_eq!(entries_exist, 1, "Entries table should exist");
        
        // Check for tags table
        let tags_exist: i32 = db.conn().query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='tags'",
            [],
            |row| row.get(0),
        ).expect("Failed to check tags table");
        
        assert_eq!(tags_exist, 1, "Tags table should exist");
        
        // Check for entry_tags table
        let entry_tags_exist: i32 = db.conn().query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='entry_tags'",
            [],
            |row| row.get(0),
        ).expect("Failed to check entry_tags table");
        
        assert_eq!(entry_tags_exist, 1, "entry_tags table should exist");
    }
}
