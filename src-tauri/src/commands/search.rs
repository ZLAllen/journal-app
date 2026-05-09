use crate::db::DbConnection;
use crate::models::{AppError, Entry, Result};
use rusqlite::params;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SearchResult {
    pub entry: Entry,
    pub snippet: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SearchEntriesResponse {
    pub results: Vec<SearchResult>,
    pub elapsed_ms: u128,
}

pub fn search_entries(
    db: &DbConnection,
    query: String,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<SearchEntriesResponse> {
    let trimmed_query = query.trim();
    if trimmed_query.is_empty() {
        return Err(AppError::InvalidInput(
            "Search query must not be empty".to_string(),
        ));
    }

    let start = std::time::Instant::now();
    let effective_limit = limit.unwrap_or(50).clamp(1, 200);
    let effective_offset = offset.unwrap_or(0).max(0);

    let conn = db.conn();
    let mut stmt = conn.prepare(
        "WITH matched AS (
            SELECT
                e.rowid AS rowid
            FROM entries_fts
            JOIN entries e ON e.rowid = entries_fts.rowid
            WHERE e.deleted_at IS NULL
              AND entries_fts MATCH ?1
            ORDER BY e.created_at DESC
            LIMIT ?2 OFFSET ?3
         )
         SELECT
            e.id,
            e.created_at,
            e.updated_at,
            e.title,
            '' AS body,
            e.mood,
            e.pinned,
            e.deleted_at,
            '' AS body_html,
            '' AS body_text,
            e.word_count,
            snippet(entries_fts, 0, '<mark>', '</mark>', ' ... ', 12)
         FROM matched
         JOIN entries_fts ON entries_fts.rowid = matched.rowid
         JOIN entries e ON e.rowid = matched.rowid
         WHERE entries_fts MATCH ?1
         ORDER BY e.created_at DESC",
    )?;

    let rows = stmt.query_map(
        params![trimmed_query, effective_limit, effective_offset],
        |row| {
            let entry = Entry::try_from(row)?;
            let snippet: String = row.get(11)?;
            Ok(SearchResult { entry, snippet })
        },
    )?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }

    Ok(SearchEntriesResponse {
        results,
        elapsed_ms: start.elapsed().as_millis(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::entries;

    fn setup_db() -> DbConnection {
        DbConnection::new_memory().expect("Failed to create test DB")
    }

    #[test]
    fn search_entries_returns_match_and_snippet() {
        let db = setup_db();
        entries::create_entry(
            &db,
            "Search title".to_string(),
            "<p>Today I practiced piano in the rain.</p>".to_string(),
            None,
        )
        .expect("create entry should succeed");

        let response = search_entries(&db, "piano".to_string(), Some(10), Some(0))
            .expect("search should succeed");

        assert_eq!(response.results.len(), 1);
        assert!(response.results[0].snippet.contains("<mark>piano</mark>"));
    }

    #[test]
    fn search_entries_excludes_soft_deleted_entries() {
        let db = setup_db();
        let entry = entries::create_entry(
            &db,
            "Disposable".to_string(),
            "<p>hidden keyword example</p>".to_string(),
            None,
        )
        .expect("create entry should succeed");
        entries::delete_entry(&db, entry.id).expect("delete should succeed");

        let response = search_entries(&db, "keyword".to_string(), Some(10), Some(0))
            .expect("search should succeed");

        assert!(response.results.is_empty());
    }
}
