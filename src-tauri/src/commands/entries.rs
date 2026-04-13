use crate::db::DbConnection;
use crate::models::{Entry, Result};
use chrono::Utc;
use rusqlite::params;

/// Create a new journal entry
pub fn create_entry(
    db: &DbConnection,
    title: String,
    body: String,
    mood: Option<i32>,
) -> Result<Entry> {
    let entry = Entry::new(title.clone(), body.clone(), mood);

    db.run_with_search_index_repair(|conn| {
        conn.execute(
            "INSERT INTO entries (id, created_at, updated_at, title, body, mood, pinned, deleted_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                &entry.id,
                entry.created_at,
                entry.updated_at,
                &entry.title,
                &entry.body,
                entry.mood,
                if entry.pinned { 1 } else { 0 },
                entry.deleted_at,
            ],
        )?;

        Ok(())
    })?;

    Ok(entry)
}

/// Update an existing journal entry
pub fn update_entry(
    db: &DbConnection,
    id: String,
    title: String,
    body: String,
    mood: Option<i32>,
    created_at: Option<i64>,
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

    let entry = db.run_with_search_index_repair(|conn| {
        conn.execute(
            "UPDATE entries
             SET title = ?1,
                 body = ?2,
                 mood = ?3,
                 created_at = COALESCE(?4, created_at),
                 updated_at = ?5
             WHERE id = ?6",
            params![&title, &body, mood, created_at, now, &id],
        )?;

        let mut stmt = conn.prepare(
            "SELECT id, created_at, updated_at, title, body, mood, pinned, deleted_at FROM entries WHERE id = ?1",
        )?;

        let entry = stmt.query_row(params![&id], |row| Entry::try_from(row))?;

        Ok(entry)
    })?;

    Ok(entry)
}

/// Delete (soft delete) a journal entry
pub fn delete_entry(db: &DbConnection, id: String) -> Result<()> {
    let now = Utc::now().timestamp_millis();

    let rows_affected = db.run_with_search_index_repair(|conn| {
        let rows_affected = conn.execute(
            "UPDATE entries SET deleted_at = ?1 WHERE id = ?2 AND deleted_at IS NULL",
            params![now, &id],
        )?;

        Ok(rows_affected)
    })?;

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
        "SELECT id, created_at, updated_at, title, body, mood, pinned, deleted_at 
         FROM entries 
         WHERE deleted_at IS NULL 
         ORDER BY created_at DESC",
    )?;

    let entries = stmt.query_map([], |row| Entry::try_from(row))?;

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
        "SELECT id, created_at, updated_at, title, body, mood, pinned, deleted_at 
         FROM entries 
         WHERE id = ?1 AND deleted_at IS NULL",
    )?;

    let entry = stmt.query_row(params![&id], |row| Entry::try_from(row));

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
        "SELECT id, created_at, updated_at, title, body, mood, pinned, deleted_at 
         FROM entries 
         WHERE deleted_at IS NULL AND pinned = 1 
         ORDER BY created_at DESC",
    )?;

    let entries = stmt.query_map([], |row| Entry::try_from(row))?;

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
        let entry = create_entry(
            &db,
            "Test title".to_string(),
            "Test entry".to_string(),
            Some(4),
        )
        .expect("Failed to create entry");

        assert!(!entry.id.is_empty());
        assert_eq!(entry.title, "Test title");
        assert_eq!(entry.body, "Test entry");
        assert_eq!(entry.mood, Some(4));
        assert!(!entry.pinned);
        assert!(entry.deleted_at.is_none());
    }

    #[test]
    fn test_get_entries() {
        let db = setup_db();
        create_entry(&db, "Entry 1".to_string(), "Body 1".to_string(), Some(1)).unwrap();
        create_entry(&db, "Entry 2".to_string(), "Body 2".to_string(), Some(5)).unwrap();

        let entries = get_entries(&db).expect("Failed to get entries");
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn test_update_entry() {
        let db = setup_db();
        let entry = create_entry(
            &db,
            "Original title".to_string(),
            "Original".to_string(),
            Some(2),
        )
        .expect("Failed to create entry");

        let updated = update_entry(
            &db,
            entry.id.clone(),
            "Updated title".to_string(),
            "Updated".to_string(),
            Some(5),
            None,
        )
        .expect("Failed to update entry");

        assert_eq!(updated.title, "Updated title");
        assert_eq!(updated.body, "Updated");
        assert_eq!(updated.mood, Some(5));
        assert!(
            updated.updated_at >= entry.updated_at,
            "updated_at should be at least as recent as the original timestamp"
        );
    }

    #[test]
    fn test_backdate_entry() {
        let db = setup_db();
        let entry = create_entry(
            &db,
            "Original title".to_string(),
            "Original".to_string(),
            Some(2),
        )
        .expect("Failed to create entry");
        let backdated = entry.created_at - 86_400_000;

        let updated = update_entry(
            &db,
            entry.id,
            "Backdated title".to_string(),
            "Backdated".to_string(),
            Some(2),
            Some(backdated),
        )
        .expect("Failed to backdate entry");

        assert_eq!(updated.created_at, backdated);
    }

    #[test]
    fn test_delete_entry() {
        let db = setup_db();
        let entry = create_entry(&db, "To delete".to_string(), "To delete".to_string(), None)
            .expect("Failed to create entry");

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
        let entry = create_entry(
            &db,
            "Pinnable entry".to_string(),
            "Pinnable entry".to_string(),
            None,
        )
        .expect("Failed to create entry");

        set_pinned(&db, entry.id.clone(), true).expect("Failed to set pinned");

        let updated = get_entry(&db, entry.id)
            .expect("Failed to retrieve")
            .expect("Entry should exist");
        assert!(updated.pinned);
    }

    #[test]
    fn test_get_pinned_entries() {
        let db = setup_db();
        let entry1 = create_entry(&db, "Entry 1".to_string(), "Body 1".to_string(), None)
            .expect("Failed to create entry");
        let _entry2 = create_entry(&db, "Entry 2".to_string(), "Body 2".to_string(), None)
            .expect("Failed to create entry");

        set_pinned(&db, entry1.id, true).unwrap();

        let pinned = get_pinned_entries(&db).expect("Failed to get pinned");
        assert_eq!(pinned.len(), 1);
        assert_eq!(pinned[0].title, "Entry 1");
        assert_eq!(pinned[0].body, "Body 1");
    }

    #[test]
    fn test_update_entry_refreshes_search_index() {
        let db = setup_db();
        let entry = create_entry(
            &db,
            "Original title".to_string(),
            "Original body".to_string(),
            None,
        )
        .expect("Failed to create entry");

        update_entry(
            &db,
            entry.id,
            "Updated title".to_string(),
            "Updated body".to_string(),
            None,
            None,
        )
        .expect("Failed to update entry");

        let original_matches: i32 = db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM entries_fts WHERE entries_fts MATCH 'Original'",
                [],
                |row| row.get(0),
            )
            .expect("Failed to query original token matches");
        let updated_matches: i32 = db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM entries_fts WHERE entries_fts MATCH 'Updated'",
                [],
                |row| row.get(0),
            )
            .expect("Failed to query updated token matches");

        assert_eq!(original_matches, 0, "Old terms should be removed from FTS");
        assert_eq!(updated_matches, 1, "Updated terms should be indexed in FTS");
    }

    #[test]
    fn test_delete_entry_removes_search_index_match() {
        let db = setup_db();
        let entry = create_entry(
            &db,
            "Disposable title".to_string(),
            "Disposable keyword".to_string(),
            None,
        )
        .expect("Failed to create entry");

        delete_entry(&db, entry.id).expect("Failed to delete entry");

        let matches: i32 = db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM entries_fts WHERE entries_fts MATCH 'Disposable'",
                [],
                |row| row.get(0),
            )
            .expect("Failed to query FTS matches after delete");

        assert_eq!(
            matches, 0,
            "Soft-deleted entries should not remain searchable"
        );
    }
}
