use crate::db::DbConnection;
use crate::models::{Result, SummaryStats, TopTag};
use chrono::{Datelike, Duration, Local, LocalResult, NaiveDate, TimeZone};
use rusqlite::params;
use std::collections::BTreeSet;

fn millis_to_local_date(millis: i64) -> Option<NaiveDate> {
    match Local.timestamp_millis_opt(millis) {
        LocalResult::Single(dt) => Some(dt.date_naive()),
        LocalResult::Ambiguous(a, b) => Some(a.min(b).date_naive()),
        LocalResult::None => None,
    }
}

fn calculate_writing_streak(created_at_millis: &[i64]) -> i32 {
    let dates: BTreeSet<NaiveDate> = created_at_millis
        .iter()
        .filter_map(|millis| millis_to_local_date(*millis))
        .collect();

    let Some(mut cursor) = dates.iter().next_back().copied() else {
        return 0;
    };

    let mut streak = 1;
    loop {
        let prev = cursor - Duration::days(1);
        if dates.contains(&prev) {
            streak += 1;
            cursor = prev;
            continue;
        }
        break;
    }

    streak
}

pub fn get_summary_stats(db: &DbConnection) -> Result<SummaryStats> {
    let conn = db.conn();

    let (total_entries, total_word_count): (i32, i32) = conn.query_row(
        "SELECT COUNT(*), COALESCE(SUM(word_count), 0)
         FROM entries
         WHERE deleted_at IS NULL",
        [],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;

    let now = Local::now();
    let month_start = Local
        .with_ymd_and_hms(now.year(), now.month(), 1, 0, 0, 0)
        .single()
        .expect("valid current month start")
        .timestamp_millis();

    let (next_year, next_month) = if now.month() == 12 {
        (now.year() + 1, 1)
    } else {
        (now.year(), now.month() + 1)
    };

    let next_month_start = Local
        .with_ymd_and_hms(next_year, next_month, 1, 0, 0, 0)
        .single()
        .expect("valid next month start")
        .timestamp_millis();

    let entries_this_month: i32 = conn.query_row(
        "SELECT COUNT(*)
         FROM entries
         WHERE deleted_at IS NULL
           AND created_at >= ?1
           AND created_at < ?2",
        params![month_start, next_month_start],
        |row| row.get(0),
    )?;

    let mut dates_stmt = conn.prepare(
        "SELECT created_at
         FROM entries
         WHERE deleted_at IS NULL",
    )?;
    let dates_iter = dates_stmt.query_map([], |row| row.get::<_, i64>(0))?;
    let mut created_at_millis = Vec::new();
    for created_at in dates_iter {
        created_at_millis.push(created_at?);
    }
    let writing_streak_days = calculate_writing_streak(&created_at_millis);

    let mut tags_stmt = conn.prepare(
        "SELECT t.id, t.name, COUNT(DISTINCT e.id) AS usage_count
         FROM tags t
         JOIN entry_tags et ON et.tag_id = t.id
         JOIN entries e ON e.id = et.entry_id
         WHERE e.deleted_at IS NULL
         GROUP BY t.id, t.name
         ORDER BY usage_count DESC, t.name ASC
         LIMIT 10",
    )?;
    let top_tags_iter = tags_stmt.query_map([], |row| {
        Ok(TopTag {
            id: row.get(0)?,
            name: row.get(1)?,
            usage_count: row.get(2)?,
        })
    })?;
    let mut top_tags = Vec::new();
    for tag in top_tags_iter {
        top_tags.push(tag?);
    }

    Ok(SummaryStats {
        writing_streak_days,
        total_entries,
        total_word_count,
        entries_this_month,
        top_tags,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculate_writing_streak_counts_contiguous_days() {
        let d1 = Local
            .with_ymd_and_hms(2026, 5, 7, 9, 0, 0)
            .single()
            .expect("valid date")
            .timestamp_millis();
        let d2 = Local
            .with_ymd_and_hms(2026, 5, 8, 9, 0, 0)
            .single()
            .expect("valid date")
            .timestamp_millis();
        let d3 = Local
            .with_ymd_and_hms(2026, 5, 9, 9, 0, 0)
            .single()
            .expect("valid date")
            .timestamp_millis();
        let gap = Local
            .with_ymd_and_hms(2026, 5, 2, 9, 0, 0)
            .single()
            .expect("valid date")
            .timestamp_millis();

        let streak = calculate_writing_streak(&[d1, d2, d3, gap, d3]);
        assert_eq!(streak, 3);
    }
}
