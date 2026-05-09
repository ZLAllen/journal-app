use journal::db::DbConnection;
use rusqlite::params;
use serde::Serialize;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Debug, Serialize)]
struct ExportTag {
    id: String,
    name: String,
}

#[derive(Debug, Serialize)]
struct ExportEntry {
    id: String,
    created_at: i64,
    updated_at: i64,
    title: String,
    body_html: String,
    body_text: String,
    word_count: i32,
    mood: Option<i32>,
    pinned: bool,
    tags: Vec<ExportTag>,
}

#[derive(Debug, Serialize)]
struct ExportDocument {
    format: String,
    version: i32,
    exported_at_ms: i64,
    entries: Vec<ExportEntry>,
}

struct Options {
    db_path: String,
    output_path: String,
}

fn usage() {
    println!("Usage: cargo run --bin benchmark_export_10k -- [options]");
    println!();
    println!("Options:");
    println!("  --db-path <path>      SQLite database path (required)");
    println!("  --output-path <path>  JSON output file path (required)");
}

fn parse_options() -> Result<Options, String> {
    let mut db_path = String::new();
    let mut output_path = String::new();

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--db-path" => {
                db_path = args
                    .next()
                    .ok_or_else(|| String::from("Missing value for --db-path"))?;
            }
            "--output-path" => {
                output_path = args
                    .next()
                    .ok_or_else(|| String::from("Missing value for --output-path"))?;
            }
            "--help" | "-h" => {
                usage();
                std::process::exit(0);
            }
            _ => return Err(format!("Unknown argument: {arg}")),
        }
    }

    if db_path.is_empty() {
        return Err(String::from("--db-path is required"));
    }
    if output_path.is_empty() {
        return Err(String::from("--output-path is required"));
    }

    Ok(Options {
        db_path,
        output_path,
    })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = parse_options().map_err(|e| format!("{e}\nUse --help for usage."))?;
    let db = DbConnection::new(&options.db_path)?;
    let conn = db.conn();

    let start = Instant::now();

    let mut tags_stmt = conn.prepare(
        "SELECT et.entry_id, t.id, t.name
         FROM entry_tags et
         JOIN tags t ON t.id = et.tag_id",
    )?;
    let tag_rows = tags_stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            ExportTag {
                id: row.get(1)?,
                name: row.get(2)?,
            },
        ))
    })?;

    let mut tags_by_entry: HashMap<String, Vec<ExportTag>> = HashMap::new();
    for row in tag_rows {
        let (entry_id, tag) = row?;
        tags_by_entry.entry(entry_id).or_default().push(tag);
    }

    let mut entry_stmt = conn.prepare(
        "SELECT id, created_at, updated_at, title, body_html, body_text, word_count, mood, pinned
         FROM entries
         WHERE deleted_at IS NULL
         ORDER BY pinned DESC, created_at DESC",
    )?;
    let entry_rows = entry_stmt.query_map(params![], |row| {
        let id: String = row.get(0)?;
        Ok(ExportEntry {
            id: id.clone(),
            created_at: row.get(1)?,
            updated_at: row.get(2)?,
            title: row.get(3)?,
            body_html: row.get(4)?,
            body_text: row.get(5)?,
            word_count: row.get(6)?,
            mood: row.get(7)?,
            pinned: row.get::<_, i32>(8)? != 0,
            tags: tags_by_entry.remove(&id).unwrap_or_default(),
        })
    })?;

    let mut entries = Vec::new();
    for row in entry_rows {
        entries.push(row?);
    }

    let document = ExportDocument {
        format: "journal_mvp_export_v1".to_string(),
        version: 1,
        exported_at_ms: chrono::Utc::now().timestamp_millis(),
        entries,
    };

    let output_json = serde_json::to_vec(&document)?;
    let output_path = PathBuf::from(&options.output_path);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&output_path, output_json)?;

    let elapsed_ms = start.elapsed().as_millis();
    let size_bytes = fs::metadata(&output_path)?.len();
    println!(
        "benchmark_export_10k: entries={} elapsed_ms={} output_bytes={} path={}",
        document.entries.len(),
        elapsed_ms,
        size_bytes,
        output_path.display()
    );

    Ok(())
}
