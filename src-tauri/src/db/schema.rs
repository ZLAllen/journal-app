/// Migration structure
pub struct Migration {
    pub name: &'static str,
    pub sql: &'static str,
}

/// Get all migrations in order
pub fn get_migrations() -> Vec<Migration> {
    vec![
        Migration {
            name: "001_init_schema",
            sql: include_str!("../../../migrations/001_init_schema.sql"),
        },
        Migration {
            name: "002_fts5_index",
            sql: include_str!("../../../migrations/002_fts5_index.sql"),
        },
    ]
}
