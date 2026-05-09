use journal::commands::{entries, tags};
use journal::db::DbConnection;
use std::collections::HashMap;
use std::env;
use std::time::Instant;

const DEFAULT_ITERATIONS: usize = 15;
const DEFAULT_WARMUP: usize = 3;
const DEFAULT_PAGE_SIZE: usize = 50;
const DEFAULT_MAX_MS: u128 = 300;

struct Options {
    db_path: String,
    iterations: usize,
    warmup: usize,
    page_size: usize,
    max_ms: u128,
}

fn print_usage() {
    println!("Usage: cargo run --bin benchmark_timeline_first_page -- [options]");
    println!();
    println!("Options:");
    println!("  --db-path <path>         SQLite database path (required)");
    println!("  --iterations <n>         Measured iterations (default: 15)");
    println!("  --warmup <n>             Warmup iterations (default: 3)");
    println!("  --page-size <n>          First page size (default: 50)");
    println!("  --max-ms <n>             Threshold for average time (default: 300)");
}

fn parse_options() -> Result<Options, String> {
    let mut db_path = String::new();
    let mut iterations = DEFAULT_ITERATIONS;
    let mut warmup = DEFAULT_WARMUP;
    let mut page_size = DEFAULT_PAGE_SIZE;
    let mut max_ms = DEFAULT_MAX_MS;

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--db-path" => {
                db_path = args
                    .next()
                    .ok_or_else(|| String::from("Missing value for --db-path"))?;
            }
            "--iterations" => {
                iterations = args
                    .next()
                    .ok_or_else(|| String::from("Missing value for --iterations"))?
                    .parse::<usize>()
                    .map_err(|_| String::from("Invalid integer for --iterations"))?;
            }
            "--warmup" => {
                warmup = args
                    .next()
                    .ok_or_else(|| String::from("Missing value for --warmup"))?
                    .parse::<usize>()
                    .map_err(|_| String::from("Invalid integer for --warmup"))?;
            }
            "--page-size" => {
                page_size = args
                    .next()
                    .ok_or_else(|| String::from("Missing value for --page-size"))?
                    .parse::<usize>()
                    .map_err(|_| String::from("Invalid integer for --page-size"))?;
            }
            "--max-ms" => {
                max_ms = args
                    .next()
                    .ok_or_else(|| String::from("Missing value for --max-ms"))?
                    .parse::<u128>()
                    .map_err(|_| String::from("Invalid integer for --max-ms"))?;
            }
            "--help" | "-h" => {
                print_usage();
                std::process::exit(0);
            }
            _ => return Err(format!("Unknown argument: {arg}")),
        }
    }

    if db_path.is_empty() {
        return Err(String::from("--db-path is required"));
    }
    if iterations == 0 {
        return Err(String::from("--iterations must be greater than 0"));
    }
    if page_size == 0 {
        return Err(String::from("--page-size must be greater than 0"));
    }

    Ok(Options {
        db_path,
        iterations,
        warmup,
        page_size,
        max_ms,
    })
}

fn run_first_page_load(
    db: &DbConnection,
    page_size: usize,
) -> Result<usize, Box<dyn std::error::Error>> {
    let mut all_entries = entries::get_entries(db)?;
    let tag_map = tags::get_all_entry_tags(db)?;

    all_entries.sort_by(|a, b| {
        if a.pinned != b.pinned {
            return if a.pinned {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            };
        }
        b.created_at.cmp(&a.created_at)
    });

    let mut resolved_tags: HashMap<String, usize> = HashMap::with_capacity(all_entries.len());
    let first_page = all_entries.into_iter().take(page_size);
    let mut count = 0usize;
    for entry in first_page {
        let tag_count = tag_map.get(&entry.id).map(|v| v.len()).unwrap_or(0);
        resolved_tags.insert(entry.id, tag_count);
        count += 1;
    }

    Ok(count)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = parse_options().map_err(|e| format!("{e}\nUse --help for usage."))?;
    let db = DbConnection::new(&options.db_path)?;

    for _ in 0..options.warmup {
        let _ = run_first_page_load(&db, options.page_size)?;
    }

    let mut elapsed_samples = Vec::with_capacity(options.iterations);
    let mut loaded_count = 0usize;
    for _ in 0..options.iterations {
        let start = Instant::now();
        loaded_count = run_first_page_load(&db, options.page_size)?;
        elapsed_samples.push(start.elapsed().as_millis());
    }

    let sum: u128 = elapsed_samples.iter().copied().sum();
    let average = sum as f64 / options.iterations as f64;
    let min = elapsed_samples.iter().copied().min().unwrap_or(0);
    let max = elapsed_samples.iter().copied().max().unwrap_or(0);

    println!(
        "benchmark_timeline_first_page: loaded={} avg_ms={:.2} min_ms={} max_ms={} iterations={} warmup={} page_size={}",
        loaded_count, average, min, max, options.iterations, options.warmup, options.page_size
    );

    if average > options.max_ms as f64 {
        return Err(format!(
            "Average timeline first-page latency {:.2}ms exceeded threshold {}ms",
            average, options.max_ms
        )
        .into());
    }

    Ok(())
}
