use crate::db::DbConnection;
use crate::models::{AppError, Entry, Result};
use chrono::Utc;
use rusqlite::params;

#[derive(Debug, Clone, PartialEq, Eq)]
struct BodyProjection {
    html: String,
    text: String,
    word_count: i32,
}

#[derive(Debug, PartialEq, Eq)]
struct ParsedTag {
    name: String,
    is_closing: bool,
    length: usize,
}

fn project_body(input: &str) -> BodyProjection {
    let html = sanitize_html(input);
    let text = extract_text(&html);
    let word_count = count_words(&text);

    BodyProjection {
        html,
        text,
        word_count,
    }
}

fn sanitize_html(input: &str) -> String {
    const ALLOWED_TAGS: [&str; 10] = [
        "p", "strong", "em", "h1", "h2", "h3", "ul", "ol", "li", "br",
    ];

    let mut output = String::with_capacity(input.len());
    let mut index = 0;
    let mut blocked_tag: Option<&str> = None;

    while index < input.len() {
        let remaining = &input[index..];

        if let Some(tag) = blocked_tag {
            if let Some(parsed) = parse_tag(remaining) {
                if parsed.is_closing && parsed.name == tag {
                    blocked_tag = None;
                }
                index += parsed.length;
            } else if let Some(next_tag) = remaining.find('<') {
                index += next_tag;
            } else {
                break;
            }
            continue;
        }

        if remaining.starts_with('<') {
            if let Some(parsed) = parse_tag(remaining) {
                if matches!(parsed.name.as_str(), "script" | "style" | "iframe")
                    && !parsed.is_closing
                {
                    blocked_tag = Some(match parsed.name.as_str() {
                        "script" => "script",
                        "style" => "style",
                        "iframe" => "iframe",
                        _ => unreachable!(),
                    });
                } else if ALLOWED_TAGS.contains(&parsed.name.as_str()) {
                    if parsed.name == "br" {
                        output.push_str("<br>");
                    } else if parsed.is_closing {
                        output.push_str("</");
                        output.push_str(&parsed.name);
                        output.push('>');
                    } else {
                        output.push('<');
                        output.push_str(&parsed.name);
                        output.push('>');
                    }
                }
                index += parsed.length;
                continue;
            }

            output.push_str("&lt;");
            index += 1;
            continue;
        }

        let ch = remaining
            .chars()
            .next()
            .expect("remaining input should contain a character");
        match ch {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' => output.push_str("&quot;"),
            '\'' => output.push_str("&#39;"),
            _ => output.push(ch),
        }
        index += ch.len_utf8();
    }

    output
}

fn parse_tag(input: &str) -> Option<ParsedTag> {
    if !input.starts_with('<') {
        return None;
    }

    let end = input.find('>')?;
    let mut content = input[1..end].trim();
    let is_closing = content.starts_with('/');
    if is_closing {
        content = content[1..].trim_start();
    }

    if content.starts_with('!') || content.starts_with('?') {
        return Some(ParsedTag {
            name: String::new(),
            is_closing,
            length: end + 1,
        });
    }

    let name_end = content
        .find(|ch: char| ch.is_whitespace() || ch == '/')
        .unwrap_or(content.len());
    if name_end == 0 {
        return None;
    }

    Some(ParsedTag {
        name: content[..name_end].to_ascii_lowercase(),
        is_closing,
        length: end + 1,
    })
}

fn extract_text(html: &str) -> String {
    let mut text = String::with_capacity(html.len());
    let mut index = 0;

    while index < html.len() {
        let remaining = &html[index..];

        if remaining.starts_with('<') {
            if let Some(parsed) = parse_tag(remaining) {
                if matches!(parsed.name.as_str(), "p" | "h1" | "h2" | "h3" | "li" | "br") {
                    text.push(' ');
                }
                index += parsed.length;
                continue;
            }
        }

        if remaining.starts_with('&') {
            if let Some((decoded, length)) = decode_entity(remaining) {
                text.push(decoded);
                index += length;
                continue;
            }
        }

        let ch = remaining
            .chars()
            .next()
            .expect("remaining HTML should contain a character");
        text.push(ch);
        index += ch.len_utf8();
    }

    normalize_whitespace(&text)
}

fn decode_entity(input: &str) -> Option<(char, usize)> {
    let end = input.find(';')?;
    if end > 12 {
        return None;
    }

    let entity = &input[1..end];
    let decoded = match entity {
        "amp" => '&',
        "lt" => '<',
        "gt" => '>',
        "quot" => '"',
        "apos" => '\'',
        "#39" => '\'',
        "nbsp" => ' ',
        _ if entity.starts_with("#x") => {
            let codepoint = u32::from_str_radix(&entity[2..], 16).ok()?;
            char::from_u32(codepoint)?
        }
        _ if entity.starts_with('#') => {
            let codepoint = entity[1..].parse::<u32>().ok()?;
            char::from_u32(codepoint)?
        }
        _ => return None,
    };

    Some((decoded, end + 1))
}

fn normalize_whitespace(input: &str) -> String {
    input.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn count_words(input: &str) -> i32 {
    input
        .split_whitespace()
        .filter(|word| !word.is_empty())
        .count() as i32
}

const MAX_ENTRY_BODY_CHARS: usize = 50_000;
const DEFAULT_LIST_LIMIT: i64 = 50;
const MAX_LIST_LIMIT: i64 = 200;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EntrySummary {
    pub id: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub title: String,
    pub mood: Option<i32>,
    pub pinned: bool,
    pub word_count: i32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct ListEntriesFilters {
    pub date_from_ms: Option<i64>,
    pub date_to_ms: Option<i64>,
    pub mood: Option<i32>,
    pub pinned: Option<bool>,
    pub tag_id: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ListEntriesResponse {
    pub entries: Vec<EntrySummary>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ListCursor {
    pinned: i32,
    created_at: i64,
    id: String,
}

impl ListCursor {
    fn parse(raw: &str) -> Result<Self> {
        let mut parts = raw.splitn(3, '|');
        let pinned = parts
            .next()
            .ok_or_else(|| AppError::InvalidInput("Invalid cursor format".to_string()))?
            .parse::<i32>()
            .map_err(|_| AppError::InvalidInput("Invalid cursor pinned value".to_string()))?;
        let created_at = parts
            .next()
            .ok_or_else(|| AppError::InvalidInput("Invalid cursor format".to_string()))?
            .parse::<i64>()
            .map_err(|_| AppError::InvalidInput("Invalid cursor timestamp".to_string()))?;
        let id = parts
            .next()
            .ok_or_else(|| AppError::InvalidInput("Invalid cursor format".to_string()))?
            .to_string();

        if !(0..=1).contains(&pinned) || id.is_empty() {
            return Err(AppError::InvalidInput(
                "Invalid cursor component values".to_string(),
            ));
        }

        Ok(Self {
            pinned,
            created_at,
            id,
        })
    }

    fn encode(&self) -> String {
        format!("{}|{}|{}", self.pinned, self.created_at, self.id)
    }
}

fn validate_entry_payload(body: &str, mood: Option<i32>) -> Result<()> {
    if let Some(mood_value) = mood {
        if !(1..=5).contains(&mood_value) {
            return Err(AppError::InvalidInput(
                "Mood must be between 1 and 5".to_string(),
            ));
        }
    }

    if body.chars().count() > MAX_ENTRY_BODY_CHARS {
        return Err(AppError::InvalidInput(format!(
            "Entry body must be at most {} characters",
            MAX_ENTRY_BODY_CHARS
        )));
    }

    Ok(())
}

/// Create a new journal entry
pub fn create_entry(
    db: &DbConnection,
    title: String,
    body: String,
    mood: Option<i32>,
) -> Result<Entry> {
    validate_entry_payload(&body, mood)?;
    let body_projection = project_body(&body);
    let mut entry = Entry::new(title.clone(), body_projection.html.clone(), mood);
    entry.body_html = body_projection.html;
    entry.body_text = body_projection.text;
    entry.word_count = body_projection.word_count;

    db.run_with_search_index_repair(|conn| {
        conn.execute(
            "INSERT INTO entries (
                id, created_at, updated_at, title, body, body_html, body_text, word_count, mood, pinned, deleted_at
             )
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                &entry.id,
                entry.created_at,
                entry.updated_at,
                &entry.title,
                &entry.body,
                &entry.body_html,
                &entry.body_text,
                entry.word_count,
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
    validate_entry_payload(&body, mood)?;
    let now = Utc::now().timestamp_millis();
    let body_projection = project_body(&body);
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
                 body_html = ?3,
                 body_text = ?4,
                 word_count = ?5,
                 mood = ?6,
                 created_at = COALESCE(?7, created_at),
                 updated_at = ?8
             WHERE id = ?9",
            params![
                &title,
                &body_projection.html,
                &body_projection.html,
                &body_projection.text,
                body_projection.word_count,
                mood,
                created_at,
                now,
                &id
            ],
        )?;

        let mut stmt = conn.prepare(
            "SELECT id, created_at, updated_at, title, body, mood, pinned, deleted_at, body_html, body_text, word_count
             FROM entries WHERE id = ?1",
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
        "SELECT id, created_at, updated_at, title, body, mood, pinned, deleted_at, body_html, body_text, word_count
         FROM entries 
         WHERE deleted_at IS NULL 
         ORDER BY pinned DESC, created_at DESC",
    )?;

    let entries = stmt.query_map([], |row| Entry::try_from(row))?;

    let mut result = Vec::new();
    for entry in entries {
        result.push(entry?);
    }

    Ok(result)
}

pub fn list_entries(
    db: &DbConnection,
    cursor: Option<String>,
    limit: Option<i64>,
    filters: Option<ListEntriesFilters>,
) -> Result<ListEntriesResponse> {
    let effective_limit = limit.unwrap_or(DEFAULT_LIST_LIMIT);
    if !(1..=MAX_LIST_LIMIT).contains(&effective_limit) {
        return Err(AppError::InvalidInput(format!(
            "Limit must be between 1 and {}",
            MAX_LIST_LIMIT
        )));
    }

    let parsed_cursor = match cursor {
        Some(raw) => Some(ListCursor::parse(&raw)?),
        None => None,
    };
    let effective_filters = filters.unwrap_or_default();
    if let Some(mood) = effective_filters.mood {
        if !(1..=5).contains(&mood) {
            return Err(AppError::InvalidInput(
                "Mood must be between 1 and 5".to_string(),
            ));
        }
    }

    let cursor_pinned = parsed_cursor.as_ref().map(|value| value.pinned);
    let cursor_created_at = parsed_cursor.as_ref().map(|value| value.created_at);
    let cursor_id = parsed_cursor.as_ref().map(|value| value.id.clone());
    let filter_pinned = effective_filters
        .pinned
        .map(|value| if value { 1 } else { 0 });
    let fetch_limit = effective_limit + 1;

    let conn = db.conn();
    let mut stmt = conn.prepare(
        "SELECT
            id,
            created_at,
            updated_at,
            title,
            mood,
            pinned,
            word_count
         FROM entries
         WHERE deleted_at IS NULL
           AND (?1 IS NULL OR created_at >= ?1)
           AND (?2 IS NULL OR created_at <= ?2)
           AND (?3 IS NULL OR mood = ?3)
           AND (?4 IS NULL OR pinned = ?4)
           AND (?5 IS NULL OR EXISTS (
                SELECT 1
                FROM entry_tags et
                WHERE et.entry_id = entries.id
                  AND et.tag_id = ?5
           ))
           AND (?6 IS NULL OR (
                pinned < ?6 OR
                (pinned = ?6 AND created_at < ?7) OR
                (pinned = ?6 AND created_at = ?7 AND id < ?8)
           ))
         ORDER BY pinned DESC, created_at DESC, id DESC
         LIMIT ?9",
    )?;

    let rows = stmt.query_map(
        params![
            effective_filters.date_from_ms,
            effective_filters.date_to_ms,
            effective_filters.mood,
            filter_pinned,
            effective_filters.tag_id,
            cursor_pinned,
            cursor_created_at,
            cursor_id,
            fetch_limit
        ],
        |row| {
            Ok(EntrySummary {
                id: row.get(0)?,
                created_at: row.get(1)?,
                updated_at: row.get(2)?,
                title: row.get(3)?,
                mood: row.get(4)?,
                pinned: row.get::<_, i32>(5)? != 0,
                word_count: row.get(6)?,
            })
        },
    )?;

    let mut all_rows = Vec::new();
    for row in rows {
        all_rows.push(row?);
    }

    let has_more = all_rows.len() as i64 > effective_limit;
    if has_more {
        all_rows.truncate(effective_limit as usize);
    }

    let next_cursor = if has_more {
        all_rows.last().map(|entry| {
            ListCursor {
                pinned: if entry.pinned { 1 } else { 0 },
                created_at: entry.created_at,
                id: entry.id.clone(),
            }
            .encode()
        })
    } else {
        None
    };

    Ok(ListEntriesResponse {
        entries: all_rows,
        next_cursor,
    })
}

/// Get a single entry by ID
pub fn get_entry(db: &DbConnection, id: String) -> Result<Option<Entry>> {
    let conn = db.conn();
    let mut stmt = conn.prepare(
        "SELECT id, created_at, updated_at, title, body, mood, pinned, deleted_at, body_html, body_text, word_count
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
pub fn set_pinned(db: &DbConnection, id: String, pinned: bool) -> Result<Entry> {
    let conn = db.conn();
    let rows_affected = conn.execute(
        "UPDATE entries SET pinned = ?1, updated_at = ?2 WHERE id = ?3 AND deleted_at IS NULL",
        params![
            if pinned { 1 } else { 0 },
            Utc::now().timestamp_millis(),
            &id
        ],
    )?;

    if rows_affected == 0 {
        return Err(crate::models::AppError::NotFound(format!(
            "Entry {} not found or is deleted",
            id
        )));
    }

    let entry = conn.query_row(
        "SELECT id, created_at, updated_at, title, body, mood, pinned, deleted_at, body_html, body_text, word_count
         FROM entries
         WHERE id = ?1",
        params![&id],
        |row| Entry::try_from(row),
    )?;

    Ok(entry)
}

/// Get all pinned entries
pub fn get_pinned_entries(db: &DbConnection) -> Result<Vec<Entry>> {
    let conn = db.conn();
    let mut stmt = conn.prepare(
        "SELECT id, created_at, updated_at, title, body, mood, pinned, deleted_at, body_html, body_text, word_count
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
    fn test_sanitize_html_allows_only_mvp_tags_without_attributes() {
        let projection = project_body(
            r#"<h1 onclick="bad()">Title</h1><p>Hello <strong>bold</strong> <a href="https://example.com">link</a><img src=x><script>alert(1)</script><br style="x"></p>"#,
        );

        assert_eq!(
            projection.html,
            "<h1>Title</h1><p>Hello <strong>bold</strong> link<br></p>"
        );
        assert_eq!(projection.text, "Title Hello bold link");
        assert_eq!(projection.word_count, 4);
    }

    #[test]
    fn test_sanitize_html_escapes_plain_text_and_decodes_projection() {
        let projection = project_body("Tea & > \"today\"");

        assert_eq!(projection.html, "Tea &amp; &gt; &quot;today&quot;");
        assert_eq!(projection.text, "Tea & > \"today\"");
        assert_eq!(projection.word_count, 4);
    }

    #[test]
    fn test_create_entry() {
        let db = setup_db();
        let entry = create_entry(
            &db,
            "Test title".to_string(),
            "<p>Test <em>entry</em></p>".to_string(),
            Some(4),
        )
        .expect("Failed to create entry");

        assert!(!entry.id.is_empty());
        assert_eq!(entry.title, "Test title");
        assert_eq!(entry.body, "<p>Test <em>entry</em></p>");
        assert_eq!(entry.body_html, "<p>Test <em>entry</em></p>");
        assert_eq!(entry.body_text, "Test entry");
        assert_eq!(entry.word_count, 2);
        assert_eq!(entry.mood, Some(4));
        assert!(!entry.pinned);
        assert!(entry.deleted_at.is_none());
    }

    #[test]
    fn test_create_entry_rejects_invalid_mood() {
        let db = setup_db();
        let result = create_entry(
            &db,
            "Invalid mood".to_string(),
            "<p>Body</p>".to_string(),
            Some(6),
        );
        assert!(matches!(result, Err(AppError::InvalidInput(_))));
    }

    #[test]
    fn test_create_entry_rejects_body_over_limit() {
        let db = setup_db();
        let oversized = "a".repeat(MAX_ENTRY_BODY_CHARS + 1);
        let result = create_entry(&db, "Too long".to_string(), oversized, None);
        assert!(matches!(result, Err(AppError::InvalidInput(_))));
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
    fn test_list_entries_rejects_invalid_limit() {
        let db = setup_db();
        let result = list_entries(&db, None, Some(0), None);
        assert!(matches!(result, Err(AppError::InvalidInput(_))));
    }

    #[test]
    fn test_list_entries_rejects_invalid_cursor() {
        let db = setup_db();
        let result = list_entries(&db, Some("bad-cursor".to_string()), Some(10), None);
        assert!(matches!(result, Err(AppError::InvalidInput(_))));
    }

    #[test]
    fn test_list_entries_rejects_invalid_filter_mood() {
        let db = setup_db();
        let result = list_entries(
            &db,
            None,
            Some(10),
            Some(ListEntriesFilters {
                mood: Some(6),
                ..Default::default()
            }),
        );
        assert!(matches!(result, Err(AppError::InvalidInput(_))));
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
        assert_eq!(updated.body_html, "Updated");
        assert_eq!(updated.body_text, "Updated");
        assert_eq!(updated.word_count, 1);
        assert_eq!(updated.mood, Some(5));
        assert!(
            updated.updated_at >= entry.updated_at,
            "updated_at should be at least as recent as the original timestamp"
        );
    }

    #[test]
    fn test_update_entry_rejects_invalid_mood() {
        let db = setup_db();
        let entry = create_entry(&db, "Mood test".to_string(), "Body".to_string(), Some(3))
            .expect("Failed to create entry");

        let result = update_entry(
            &db,
            entry.id,
            "Mood test".to_string(),
            "Body".to_string(),
            Some(0),
            None,
        );
        assert!(matches!(result, Err(AppError::InvalidInput(_))));
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
