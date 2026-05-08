use chrono::Utc;
use rusqlite::Row;
use serde::ser::SerializeStruct;
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
    pub body_html: String, // Sanitized rich text HTML
    pub body_text: String, // Plain-text projection for search/previews/stats
    pub mood: Option<i32>, // 1-5 scale, nullable
    pub pinned: bool,
    pub deleted_at: Option<i64>, // Soft delete timestamp
    pub word_count: i32,
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
            body: body.clone(),
            body_html: body.clone(),
            body_text: body,
            mood,
            pinned: false,
            deleted_at: None,
            word_count: 0,
        }
    }
}

impl TryFrom<&Row<'_>> for Entry {
    type Error = rusqlite::Error;

    fn try_from(row: &Row<'_>) -> std::result::Result<Self, Self::Error> {
        Ok(Self {
            id: row.get(0)?,
            created_at: row.get(1)?,
            updated_at: row.get(2)?,
            title: row.get(3)?,
            body: row.get(4)?,
            mood: row.get(5)?,
            pinned: row.get::<_, i32>(6)? != 0,
            deleted_at: row.get(7)?,
            body_html: row.get(8)?,
            body_text: row.get(9)?,
            word_count: row.get(10)?,
        })
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

    #[error("Application state lock error: {0}")]
    StateLock(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::error::Error),
}

impl AppError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Database(_) => "DATABASE",
            Self::CorruptDatabase(_) => "CORRUPT_DATABASE",
            Self::Encryption(_) => "ENCRYPTION",
            Self::Decryption(_) => "DECRYPTION",
            Self::NotFound(_) => "NOT_FOUND",
            Self::InvalidInput(_) => "INVALID_INPUT",
            Self::StateLock(_) => "STATE_LOCK",
            Self::Io(_) => "IO",
            Self::Serde(_) => "SERIALIZATION",
        }
    }

    pub fn recoverable(&self) -> bool {
        match self {
            Self::CorruptDatabase(_) | Self::StateLock(_) => false,
            Self::Database(_)
            | Self::Encryption(_)
            | Self::Decryption(_)
            | Self::NotFound(_)
            | Self::InvalidInput(_)
            | Self::Io(_)
            | Self::Serde(_) => true,
        }
    }
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("AppError", 3)?;
        state.serialize_field("code", self.code())?;
        state.serialize_field("message", &self.to_string())?;
        state.serialize_field("recoverable", &self.recoverable())?;
        state.end()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn app_error_serializes_to_frontend_contract() {
        let error = AppError::InvalidInput("Mood must be between 1 and 5".to_string());
        let serialized = serde_json::to_value(error).expect("error should serialize");

        assert_eq!(
            serialized,
            json!({
                "code": "INVALID_INPUT",
                "message": "Invalid input: Mood must be between 1 and 5",
                "recoverable": true
            })
        );
    }
}
