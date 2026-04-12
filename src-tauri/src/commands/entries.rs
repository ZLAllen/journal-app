use crate::db::DbConnection;
use crate::models::{Entry, Result};
use chrono::Utc;
use rusqlite::params;

/// Create a new journal entry
pub fn create_entry(db: &DbConnection, body: String, mood: Option<i32>) -> Result<Entry> {
    let entry = Entry::new(body.clone(), mood);

    let conn = db.conn();
    conn.execute(
        "INSERT INTO entries (id, created_at, updated_at, body, mood, pinned, deleted_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            &entry.id,
            entry.created_at,
            entry.updated_at,
            &entry.body,
            entry.mood,
            if entry.pinned { 1 } else { 0 },
            entry.deleted_at,
        ],
    )?;

    // Insert into FTS table
    conn.execute(
        "INSERT INTO entries_fts(rowid, body) VALUES ((SELECT rowid FROM entries WHERE id = ?1), ?2)",
        params![&entry.id, &body],
    )?;

    Ok(entry)
}

/// Update an existing journal entry
pub fn update_entry(
    db: &DbConnection,
    id: String,
    body: String,
    mood: Option<i32>,
) -> Result<Entry> {
    let now = Utc::now().timestamp_millis();
    let conn = db.conn();

    // Check if entry exists and is not deleted
    let exists: bool = conn.query_row(
        "SELECT COUNT(*) FROM entries WHERE id = ?1 AND deleted_at IS NULL",
        params![&id],
        |row| Ok(row.get::<_, i32>(0)? > 0),
    )?;

    if !exists {
        return Err(crate::models::AppError::NotFound(format!(
            "Entry {} not found or is deleted",
            id
        )));
    }

    conn.execute(
        "UPDATE entries SET body = ?1, mood = ?2, updated_at = ?3 WHERE id = ?4",
        params![&body, mood, now, &id],
    )?;

    // Update FTS table
    conn.execute(
        "UPDATE entries_fts SET body = ?1 WHERE rowid = (SELECT rowid FROM entries WHERE id = ?2)",
        params![&body, &id],
    )?;

    // Fetch and return the updated entry
    let mut stmt = conn.prepare(
        "SELECT id, created_at, updated_at, body, mood, pinned, deleted_at FROM entries WHERE id = ?1",
    )?;

    let entry = stmt.query_row(params![&id], |row| {
        Ok(Entry::from_row(
            row.get(0)?,
            row.get(1)?,
            row.get(2)?,
            row.get(3)?,
            row.get(4)?,
            row.get(5)?,
            row.get(6)?,
        ))
    })?;

    Ok(entry)
}

/// Delete (soft delete) a journal entry
pub fn delete_entry(db: &DbConnection, id: String) -> Result<()> {
    let now = Utc::now().timestamp_millis();
    let conn = db.conn();

    let rows_affected = conn.execute(
        "UPDATE entries SET deleted_at = ?1 WHERE id = ?2 AND deleted_at IS NULL",
        params![now, &id],
    )?;

    if rows_affected == 0 {
        return Err(crate::models::AppError::NotFound(format!(
            "Entry {} not found or already deleted",
            id
        )));
    }

    Ok(())
}

/// Get all non-deleted entries, ordered by creation time (newest first)
pub fn get_entries(db: &DbConnection) -> Result<Vec<Entry>> {
    let conn = db.conn();
    let mut stmt = conn.prepare(
        "SELECT id, created_at, updated_at, body, mood, pinned, deleted_at 
         FROM entries 
         WHERE deleted_at IS NULL 
         ORDER BY created_at DESC",
    )?;

    let entries = stmt.query_map([], |row| {
        Ok(Entry::from_row(
            row.get(0)?,
            row.get(1)?,
            row.get(2)?,
            row.get(3)?,
            row.get(4)?,
            row.get(5)?,
            row.get(6)?,
        ))
    })?;

    let mut result = Vec::new();
    for entry in entries {
        result.push(entry?);
    }

    Ok(result)
}

/// Get a single entry by ID
pub fn get_entry(db: &DbConnection, id: String) -> Result<Option<Entry>> {
    let conn = db.conn();
    let mut stmt = conn.prepare(
        "SELECT id, created_at, updated_at, body, mood, pinned, deleted_at 
         FROM entries 
         WHERE id = ?1 AND deleted_at IS NULL",
    )?;

    let entry = stmt.query_row(params![&id], |row| {
        Ok(Entry::from_row(
            row.get(0)?,
            row.get(1)?,
            row.get(2)?,
            row.get(3)?,
            row.get(4)?,
            row.get(5)?,
            row.get(6)?,
        ))
    });

    match entry {
        Ok(e) => Ok(Some(e)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// Set the pinned status of an entry
pub fn set_pinned(db: &DbConnection, id: String, pinned: bool) -> Result<()> {
    let conn = db.conn();
    let rows_affected = conn.execute(
        "UPDATE entries SET pinned = ?1 WHERE id = ?2 AND deleted_at IS NULL",
        params![if pinned { 1 } else { 0 }, &id],
    )?;

    if rows_affected == 0 {
        return Err(crate::models::AppError::NotFound(format!(
            "Entry {} not found or is deleted",
            id
        )));
    }

    Ok(())
}

/// Get all pinned entries
pub fn get_pinned_entries(db: &DbConnection) -> Result<Vec<Entry>> {
    let conn = db.conn();
    let mut stmt = conn.prepare(
        "SELECT id, created_at, updated_at, body, mood, pinned, deleted_at 
         FROM entries 
         WHERE deleted_at IS NULL AND pinned = 1 
         ORDER BY created_at DESC",
    )?;

    let entries = stmt.query_map([], |row| {
        Ok(Entry::from_row(
            row.get(0)?,
            row.get(1)?,
            row.get(2)?,
            row.get(3)?,
            row.get(4)?,
            row.get(5)?,
            row.get(6)?,
        ))
    })?;

    let mut result = Vec::new();
    for entry in entries {
        result.push(entry?);
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_db() -> DbConnection {
        DbConnection::new_memory().expect("Failed to create test DB")
    }

    #[test]
    fn test_create_entry() {
        let db = setup_db();
        let entry =
            create_entry(&db, "Test entry".to_string(), Some(4)).expect("Failed to create entry");

        assert!(!entry.id.is_empty());
        assert_eq!(entry.body, "Test entry");
        assert_eq!(entry.mood, Some(4));
        assert!(!entry.pinned);
        assert!(entry.deleted_at.is_none());
    }

    #[test]
    fn test_get_entries() {
        let db = setup_db();
        create_entry(&db, "Entry 1".to_string(), Some(1)).unwrap();
        create_entry(&db, "Entry 2".to_string(), Some(5)).unwrap();

        let entries = get_entries(&db).expect("Failed to get entries");
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn test_update_entry() {
        let db = setup_db();
        let entry =
            create_entry(&db, "Original".to_string(), Some(2)).expect("Failed to create entry");

        let updated = update_entry(&db, entry.id.clone(), "Updated".to_string(), Some(5))
            .expect("Failed to update entry");

        assert_eq!(updated.body, "Updated");
        assert_eq!(updated.mood, Some(5));
        assert!(updated.updated_at >= entry.updated_at, "updated_at should be at least as recent as the original timestamp");
    }

    #[test]
    fn test_delete_entry() {
        let db = setup_db();
        let entry =
            create_entry(&db, "To delete".to_string(), None).expect("Failed to create entry");

        delete_entry(&db, entry.id.clone()).expect("Failed to delete entry");

        let retrieved = get_entry(&db, entry.id).expect("Failed to retrieve entry");
        assert!(
            retrieved.is_none(),
            "Deleted entry should not be retrievable"
        );
    }

    #[test]
    fn test_set_pinned() {
        let db = setup_db();
        let entry =
            create_entry(&db, "Pinnable entry".to_string(), None).expect("Failed to create entry");

        set_pinned(&db, entry.id.clone(), true).expect("Failed to set pinned");

        let updated = get_entry(&db, entry.id)
            .expect("Failed to retrieve")
            .expect("Entry should exist");
        assert!(updated.pinned);
    }

    #[test]
    fn test_get_pinned_entries() {
        let db = setup_db();
        let entry1 =
            create_entry(&db, "Entry 1".to_string(), None).expect("Failed to create entry");
        let _entry2 =
            create_entry(&db, "Entry 2".to_string(), None).expect("Failed to create entry");

        set_pinned(&db, entry1.id, true).unwrap();

        let pinned = get_pinned_entries(&db).expect("Failed to get pinned");
        assert_eq!(pinned.len(), 1);
        assert_eq!(pinned[0].body, "Entry 1");
    }
}
