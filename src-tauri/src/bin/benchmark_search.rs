use journal::commands::search;
use journal::db::DbConnection;
use std::env;
use std::time::Instant;

const DEFAULT_ITERATIONS: usize = 15;
const DEFAULT_WARMUP: usize = 3;
const DEFAULT_LIMIT: i64 = 50;
const DEFAULT_OFFSET: i64 = 0;
const DEFAULT_MAX_MS: u128 = 300;

struct Options {
    db_path: String,
    query: String,
    iterations: usize,
    warmup: usize,
    limit: i64,
    offset: i64,
    max_ms: u128,
}

fn print_usage() {
    println!("Usage: cargo run --bin benchmark_search -- [options]");
    println!();
    println!("Options:");
    println!("  --db-path <path>         SQLite database path (required)");
    println!("  --query <text>           Search query (required)");
    println!("  --iterations <n>         Measured iterations (default: 15)");
    println!("  --warmup <n>             Warmup iterations (default: 3)");
    println!("  --limit <n>              Search result limit (default: 50)");
    println!("  --offset <n>             Search offset (default: 0)");
    println!("  --max-ms <n>             Threshold for average time (default: 300)");
}

fn parse_options() -> Result<Options, String> {
    let mut db_path = String::new();
    let mut query = String::new();
    let mut iterations = DEFAULT_ITERATIONS;
    let mut warmup = DEFAULT_WARMUP;
    let mut limit = DEFAULT_LIMIT;
    let mut offset = DEFAULT_OFFSET;
    let mut max_ms = DEFAULT_MAX_MS;

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--db-path" => {
                db_path = args
                    .next()
                    .ok_or_else(|| String::from("Missing value for --db-path"))?;
            }
            "--query" => {
                query = args
                    .next()
                    .ok_or_else(|| String::from("Missing value for --query"))?;
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
            "--limit" => {
                limit = args
                    .next()
                    .ok_or_else(|| String::from("Missing value for --limit"))?
                    .parse::<i64>()
                    .map_err(|_| String::from("Invalid integer for --limit"))?;
            }
            "--offset" => {
                offset = args
                    .next()
                    .ok_or_else(|| String::from("Missing value for --offset"))?
                    .parse::<i64>()
                    .map_err(|_| String::from("Invalid integer for --offset"))?;
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
    if query.trim().is_empty() {
        return Err(String::from("--query is required"));
    }
    if iterations == 0 {
        return Err(String::from("--iterations must be greater than 0"));
    }

    Ok(Options {
        db_path,
        query,
        iterations,
        warmup,
        limit,
        offset,
        max_ms,
    })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = parse_options().map_err(|e| format!("{e}\nUse --help for usage."))?;
    let db = DbConnection::new(&options.db_path)?;

    for _ in 0..options.warmup {
        let _ = search::search_entries(
            &db,
            options.query.clone(),
            Some(options.limit),
            Some(options.offset),
        )?;
    }

    let mut elapsed_samples = Vec::with_capacity(options.iterations);
    let mut last_result_count = 0usize;
    for _ in 0..options.iterations {
        let start = Instant::now();
        let response = search::search_entries(
            &db,
            options.query.clone(),
            Some(options.limit),
            Some(options.offset),
        )?;
        let elapsed = start.elapsed().as_millis();
        elapsed_samples.push(elapsed);
        last_result_count = response.results.len();
    }

    let sum: u128 = elapsed_samples.iter().copied().sum();
    let average = sum as f64 / options.iterations as f64;
    let min = elapsed_samples.iter().copied().min().unwrap_or(0);
    let max = elapsed_samples.iter().copied().max().unwrap_or(0);

    println!(
        "benchmark_search: query=\"{}\" results={} avg_ms={:.2} min_ms={} max_ms={} iterations={} warmup={}",
        options.query, last_result_count, average, min, max, options.iterations, options.warmup
    );

    if average > options.max_ms as f64 {
        return Err(format!(
            "Average search latency {:.2}ms exceeded threshold {}ms",
            average, options.max_ms
        )
        .into());
    }

    Ok(())
}
