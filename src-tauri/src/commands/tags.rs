use crate::db::DbConnection;
use crate::models::{Tag, Result};
use rusqlite::params;

/// Create a new tag
pub fn create_tag(db: &DbConnection, name: String) -> Result<Tag> {
    let tag = Tag::new(name.clone());
    let conn = db.conn();

    conn.execute(
        "INSERT INTO tags (id, name) VALUES (?1, ?2)",
        params![&tag.id, &tag.name],
    )?;

    Ok(tag)
}

/// Delete a tag
pub fn delete_tag(db: &DbConnection, id: String) -> Result<()> {
    let conn = db.conn();
    let rows_affected = conn.execute(
        "DELETE FROM tags WHERE id = ?1",
        params![&id],
    )?;

    if rows_affected == 0 {
        return Err(crate::models::AppError::NotFound(format!(
            "Tag {} not found",
            id
        )));
    }

    Ok(())
}

/// Rename a tag
pub fn rename_tag(db: &DbConnection, id: String, new_name: String) -> Result<Tag> {
    let conn = db.conn();

    // Check if tag exists
    let exists: bool = conn.query_row(
        "SELECT COUNT(*) FROM tags WHERE id = ?1",
        params![&id],
        |row| Ok(row.get::<_, i32>(0)? > 0),
    )?;

    if !exists {
        return Err(crate::models::AppError::NotFound(format!(
            "Tag {} not found",
            id
        )));
    }

    conn.execute(
        "UPDATE tags SET name = ?1 WHERE id = ?2",
        params![&new_name, &id],
    )?;

    Ok(Tag::from_row(id, new_name))
}

/// Assign a tag to an entry
pub fn assign_tag_to_entry(
    db: &DbConnection,
    entry_id: String,
    tag_id: String,
) -> Result<()> {
    let conn = db.conn();

    // Check if entry exists
    let entry_exists: bool = conn.query_row(
        "SELECT COUNT(*) FROM entries WHERE id = ?1 AND deleted_at IS NULL",
        params![&entry_id],
        |row| Ok(row.get::<_, i32>(0)? > 0),
    )?;

    if !entry_exists {
        return Err(crate::models::AppError::NotFound(format!(
            "Entry {} not found",
            entry_id
        )));
    }

    // Check if tag exists
    let tag_exists: bool = conn.query_row(
        "SELECT COUNT(*) FROM tags WHERE id = ?1",
        params![&tag_id],
        |row| Ok(row.get::<_, i32>(0)? > 0),
    )?;

    if !tag_exists {
        return Err(crate::models::AppError::NotFound(format!(
            "Tag {} not found",
            tag_id
        )));
    }

    // Insert the relationship (ignore if already exists)
    conn.execute(
        "INSERT OR IGNORE INTO entry_tags (entry_id, tag_id) VALUES (?1, ?2)",
        params![&entry_id, &tag_id],
    )?;

    Ok(())
}

/// Remove a tag from an entry
pub fn remove_tag_from_entry(
    db: &DbConnection,
    entry_id: String,
    tag_id: String,
) -> Result<()> {
    let conn = db.conn();
    let rows_affected = conn.execute(
        "DELETE FROM entry_tags WHERE entry_id = ?1 AND tag_id = ?2",
        params![&entry_id, &tag_id],
    )?;

    if rows_affected == 0 {
        return Err(crate::models::AppError::NotFound(format!(
            "No tag relationship found between entry {} and tag {}",
            entry_id, tag_id
        )));
    }

    Ok(())
}

/// Get all tags for a specific entry
pub fn get_tags_for_entry(db: &DbConnection, entry_id: String) -> Result<Vec<Tag>> {
    let conn = db.conn();
    let mut stmt = conn.prepare(
        "SELECT t.id, t.name FROM tags t
         JOIN entry_tags et ON t.id = et.tag_id
         WHERE et.entry_id = ?1",
    )?;

    let tags = stmt.query_map(params![&entry_id], |row| {
        Ok(Tag::from_row(row.get(0)?, row.get(1)?))
    })?;

    let mut result = Vec::new();
    for tag in tags {
        result.push(tag?);
    }

    Ok(result)
}

/// Get all tags
pub fn get_all_tags(db: &DbConnection) -> Result<Vec<Tag>> {
    let conn = db.conn();
    let mut stmt = conn.prepare(
        "SELECT id, name FROM tags ORDER BY name ASC",
    )?;

    let tags = stmt.query_map([], |row| {
        Ok(Tag::from_row(row.get(0)?, row.get(1)?))
    })?;

    let mut result = Vec::new();
    for tag in tags {
        result.push(tag?);
    }

    Ok(result)
}

/// Get all entries with a specific tag
pub fn get_entries_with_tag(db: &DbConnection, tag_id: String) -> Result<Vec<String>> {
    let conn = db.conn();
    let mut stmt = conn.prepare(
        "SELECT entry_id FROM entry_tags WHERE tag_id = ?1",
    )?;

    let entry_ids = stmt.query_map(params![&tag_id], |row| {
        row.get(0)
    })?;

    let mut result = Vec::new();
    for entry_id in entry_ids {
        result.push(entry_id?);
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::entries;

    fn setup_db() -> DbConnection {
        DbConnection::new_memory().expect("Failed to create test DB")
    }

    #[test]
    fn test_create_tag() {
        let db = setup_db();
        let tag = create_tag(&db, "work".to_string())
            .expect("Failed to create tag");

        assert!(!tag.id.is_empty());
        assert_eq!(tag.name, "work");
    }

    #[test]
    fn test_rename_tag() {
        let db = setup_db();
        let tag = create_tag(&db, "work".to_string())
            .expect("Failed to create tag");

        let renamed = rename_tag(&db, tag.id.clone(), "personal".to_string())
            .expect("Failed to rename tag");

        assert_eq!(renamed.name, "personal");
    }

    #[test]
    fn test_delete_tag() {
        let db = setup_db();
        let tag = create_tag(&db, "temporary".to_string())
            .expect("Failed to create tag");

        delete_tag(&db, tag.id.clone()).expect("Failed to delete tag");

        let all_tags = get_all_tags(&db).expect("Failed to get tags");
        assert_eq!(all_tags.len(), 0);
    }

    #[test]
    fn test_assign_tag_to_entry() {
        let db = setup_db();
        let entry = entries::create_entry(&db, "Test entry".to_string(), None)
            .expect("Failed to create entry");
        let tag = create_tag(&db, "important".to_string())
            .expect("Failed to create tag");

        assign_tag_to_entry(&db, entry.id.clone(), tag.id.clone())
            .expect("Failed to assign tag");

        let tags = get_tags_for_entry(&db, entry.id)
            .expect("Failed to get tags");
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].name, "important");
    }

    #[test]
    fn test_remove_tag_from_entry() {
        let db = setup_db();
        let entry = entries::create_entry(&db, "Test entry".to_string(), None)
            .expect("Failed to create entry");
        let tag = create_tag(&db, "important".to_string())
            .expect("Failed to create tag");

        assign_tag_to_entry(&db, entry.id.clone(), tag.id.clone()).unwrap();
        remove_tag_from_entry(&db, entry.id.clone(), tag.id.clone()).unwrap();

        let tags = get_tags_for_entry(&db, entry.id)
            .expect("Failed to get tags");
        assert_eq!(tags.len(), 0);
    }

    #[test]
    fn test_get_entries_with_tag() {
        let db = setup_db();
        let entry1 = entries::create_entry(&db, "Entry 1".to_string(), None).unwrap();
        let entry2 = entries::create_entry(&db, "Entry 2".to_string(), None).unwrap();
        let tag = create_tag(&db, "work".to_string()).unwrap();

        assign_tag_to_entry(&db, entry1.id.clone(), tag.id.clone()).unwrap();
        assign_tag_to_entry(&db, entry2.id.clone(), tag.id.clone()).unwrap();

        let entries_with_tag = get_entries_with_tag(&db, tag.id)
            .expect("Failed to get entries with tag");
        assert_eq!(entries_with_tag.len(), 2);
    }
}
