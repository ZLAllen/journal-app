use journal::commands::entries;
use journal::db::DbConnection;
use std::env;
use std::time::Instant;

const DEFAULT_ITERATIONS: usize = 10;
const BODY_CHAR_TARGET: usize = 50_000;

struct Options {
    db_path: String,
    iterations: usize,
}

fn print_usage() {
    println!("Usage: cargo run --bin benchmark_editor_50k -- [options]");
    println!();
    println!("Options:");
    println!("  --db-path <path>      SQLite database path (required)");
    println!(
        "  --iterations <n>      Number of create/update cycles (default: {})",
        DEFAULT_ITERATIONS
    );
}

fn parse_options() -> Result<Options, String> {
    let mut db_path = String::new();
    let mut iterations = DEFAULT_ITERATIONS;

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

    Ok(Options {
        db_path,
        iterations,
    })
}

fn build_html_body() -> String {
    let seed = "<p>focus planning reflection execution journaling</p>";
    let mut body = String::with_capacity(BODY_CHAR_TARGET + 128);
    while body.len() < BODY_CHAR_TARGET {
        body.push_str(seed);
    }
    body.truncate(BODY_CHAR_TARGET);
    body
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = parse_options().map_err(|e| format!("{e}\nUse --help for usage."))?;
    let db = DbConnection::new(&options.db_path)?;

    let body_50k = build_html_body();
    let mut create_times_ms = Vec::with_capacity(options.iterations);
    let mut update_times_ms = Vec::with_capacity(options.iterations);

    for index in 0..options.iterations {
        let title = format!("Editor50k-{index:03}");
        let create_start = Instant::now();
        let entry = entries::create_entry(&db, title.clone(), body_50k.clone(), Some(3))?;
        create_times_ms.push(create_start.elapsed().as_millis());

        let update_start = Instant::now();
        let _updated = entries::update_entry(
            &db,
            entry.id,
            format!("{title}-updated"),
            body_50k.clone(),
            Some(4),
            None,
        )?;
        update_times_ms.push(update_start.elapsed().as_millis());
    }

    let create_avg =
        create_times_ms.iter().copied().sum::<u128>() as f64 / options.iterations as f64;
    let update_avg =
        update_times_ms.iter().copied().sum::<u128>() as f64 / options.iterations as f64;
    let create_max = create_times_ms.iter().copied().max().unwrap_or(0);
    let update_max = update_times_ms.iter().copied().max().unwrap_or(0);

    println!(
        "benchmark_editor_50k: chars={} iterations={} create_avg_ms={:.2} create_max_ms={} update_avg_ms={:.2} update_max_ms={}",
        BODY_CHAR_TARGET, options.iterations, create_avg, create_max, update_avg, update_max
    );

    Ok(())
}
