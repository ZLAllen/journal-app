use chrono::{Duration, Utc};
use journal::db::DbConnection;
use rusqlite::params;
use std::env;

const DEFAULT_COUNT: usize = 10_000;
const DEFAULT_MAX_BODY_CHARS: usize = 50_000;
const DEFAULT_TAG_POOL_SIZE: usize = 32;

struct Options {
    db_path: String,
    count: usize,
    max_body_chars: usize,
}

fn parse_options() -> Result<Options, String> {
    let mut db_path = String::from("journal-fixture.db");
    let mut count = DEFAULT_COUNT;
    let mut max_body_chars = DEFAULT_MAX_BODY_CHARS;

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--db-path" => {
                let value = args
                    .next()
                    .ok_or_else(|| String::from("Missing value for --db-path"))?;
                db_path = value;
            }
            "--count" => {
                let value = args
                    .next()
                    .ok_or_else(|| String::from("Missing value for --count"))?;
                count = value
                    .parse::<usize>()
                    .map_err(|_| String::from("Invalid integer for --count"))?;
            }
            "--max-body-chars" => {
                let value = args
                    .next()
                    .ok_or_else(|| String::from("Missing value for --max-body-chars"))?;
                max_body_chars = value
                    .parse::<usize>()
                    .map_err(|_| String::from("Invalid integer for --max-body-chars"))?;
            }
            "--help" | "-h" => {
                print_usage();
                std::process::exit(0);
            }
            _ => {
                return Err(format!("Unknown argument: {arg}"));
            }
        }
    }

    if count == 0 {
        return Err(String::from("--count must be greater than 0"));
    }
    if max_body_chars == 0 {
        return Err(String::from("--max-body-chars must be greater than 0"));
    }

    Ok(Options {
        db_path,
        count,
        max_body_chars,
    })
}

fn print_usage() {
    println!("Usage: cargo run --bin generate_fixtures -- [options]");
    println!();
    println!("Options:");
    println!("  --db-path <path>          SQLite database path (default: journal-fixture.db)");
    println!("  --count <n>               Number of entries to create (default: 10000)");
    println!("  --max-body-chars <n>      Maximum body_text length (default: 50000)");
}

fn repeat_to_len(seed: &str, len: usize) -> String {
    if len == 0 {
        return String::new();
    }
    if seed.is_empty() {
        return "x".repeat(len);
    }

    let mut out = String::with_capacity(len);
    while out.len() < len {
        out.push_str(seed);
    }
    out.truncate(len);
    out
}

fn target_body_len(index: usize, max_body_chars: usize) -> usize {
    if index == 0 {
        return max_body_chars.min(DEFAULT_MAX_BODY_CHARS);
    }

    // Deterministic spread from short notes to long entries.
    let bucket = index % 8;
    match bucket {
        0 => 64,
        1 => 180,
        2 => 600,
        3 => 1_200,
        4 => 4_000,
        5 => 12_000,
        6 => 25_000,
        _ => max_body_chars.min(40_000),
    }
    .min(max_body_chars)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = parse_options().map_err(|e| format!("{e}\nUse --help for usage."))?;
    let db = DbConnection::new(&options.db_path)?;
    let conn = db.conn();

    let now = Utc::now();
    let tx = conn.unchecked_transaction()?;

    for tag_index in 0..DEFAULT_TAG_POOL_SIZE {
        let id = format!("fixture-tag-{tag_index:03}");
        let name = format!("fixture-tag-{tag_index:03}");
        tx.execute(
            "INSERT OR IGNORE INTO tags (id, name) VALUES (?1, ?2)",
            params![id, name],
        )?;
    }

    for index in 0..options.count {
        let entry_id = format!("fixture-entry-{index:05}");
        let created_at = (now - Duration::days((index % 365) as i64)).timestamp_millis();
        let updated_at = created_at + 60_000;
        let mood = ((index % 5) + 1) as i32;
        let pinned = if index % 25 == 0 { 1 } else { 0 };

        let len = target_body_len(index, options.max_body_chars);
        let phrase = format!(
            "entry {index} focus planning review journal reflection mood {} ",
            mood
        );
        let body_text = repeat_to_len(&phrase, len);
        let body_html = format!("<p>{body_text}</p>");
        let word_count = body_text.split_whitespace().count() as i32;
        let title = format!("Fixture Entry {index:05}");

        tx.execute(
            "INSERT OR REPLACE INTO entries (
                id, created_at, updated_at, title, body, body_html, body_text, word_count, mood, pinned, deleted_at
             )
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, NULL)",
            params![
                entry_id,
                created_at,
                updated_at,
                title,
                body_html,
                body_html,
                body_text,
                word_count,
                mood,
                pinned,
            ],
        )?;

        let primary_tag = format!("fixture-tag-{:03}", index % DEFAULT_TAG_POOL_SIZE);
        let secondary_tag = format!("fixture-tag-{:03}", (index + 7) % DEFAULT_TAG_POOL_SIZE);
        tx.execute(
            "INSERT OR IGNORE INTO entry_tags (entry_id, tag_id) VALUES (?1, ?2)",
            params![format!("fixture-entry-{index:05}"), primary_tag],
        )?;
        tx.execute(
            "INSERT OR IGNORE INTO entry_tags (entry_id, tag_id) VALUES (?1, ?2)",
            params![format!("fixture-entry-{index:05}"), secondary_tag],
        )?;
    }

    tx.commit()?;

    // Keep FTS in sync after bulk writes.
    conn.execute("INSERT INTO entries_fts(entries_fts) VALUES('rebuild')", [])?;

    println!(
        "Generated {} fixture entries in {}",
        options.count, options.db_path
    );

    Ok(())
}
