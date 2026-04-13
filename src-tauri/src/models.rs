use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a single journal entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub id: String,
    pub created_at: i64, // Unix timestamp in milliseconds
    pub updated_at: i64, // Unix timestamp in milliseconds
    pub title: String,
    pub body: String,      // Rich text content (HTML or Markdown)
    pub mood: Option<i32>, // 1-5 scale, nullable
    pub pinned: bool,
    pub deleted_at: Option<i64>, // Soft delete timestamp
}

/// Represents a tag for organizing entries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: String,
    pub name: String,
}

/// Represents the association between an entry and a tag
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryTag {
    pub entry_id: String,
    pub tag_id: String,
}

impl Entry {
    pub fn new(title: String, body: String, mood: Option<i32>) -> Self {
        let now = Utc::now().timestamp_millis();
        Self {
            id: Uuid::new_v4().to_string(),
            created_at: now,
            updated_at: now,
            title,
            body,
            mood,
            pinned: false,
            deleted_at: None,
        }
    }

    pub fn from_row(
        id: String,
        created_at: i64,
        updated_at: i64,
        title: String,
        body: String,
        mood: Option<i32>,
        pinned: i32,
        deleted_at: Option<i64>,
    ) -> Self {
        Self {
            id,
            created_at,
            updated_at,
            title,
            body,
            mood,
            pinned: pinned != 0,
            deleted_at,
        }
    }
}

impl Tag {
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
        }
    }

    pub fn from_row(id: String, name: String) -> Self {
        Self { id, name }
    }
}

/// Custom error types for database and crypto operations
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Database corruption detected: {0}")]
    CorruptDatabase(String),

    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("Decryption error: {0}")]
    Decryption(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::error::Error),
}

pub type Result<T> = std::result::Result<T, AppError>;
