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
        Migration {
            name: "003_fts5_triggers",
            sql: include_str!("../../../migrations/003_fts5_triggers.sql"),
        },
        Migration {
            name: "004_entry_title",
            sql: include_str!("../../../migrations/004_entry_title.sql"),
        },
        Migration {
            name: "005_entry_body_projection",
            sql: include_str!("../../../migrations/005_entry_body_projection.sql"),
        },
        Migration {
            name: "006_performance_indexes",
            sql: include_str!("../../../migrations/006_performance_indexes.sql"),
        },
    ]
}
