use clap::Parser;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::time::Instant;

mod json;
use crate::json::{count_fields_into, print_sorted_counts};

#[derive(Parser, Debug)]
#[command(version)]
#[command(about = "Process reddit dumps")]
struct Args {
    /// Input file.
    input: String,
}

mod conv;
use crate::conv::to_mib;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let file = fs::File::open(&args.input)?;
    let mut reader = BufReader::new(file);

    let mut total_counts: HashMap<String, usize> = HashMap::new();
    let mut total_bytes: usize = 0;
    let start = Instant::now();

    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => break, // EOF.
            Ok(_) => {
                total_bytes += line.len();
                let json: Value = serde_json::from_str(&line)?;
                count_fields_into(&json, &mut total_counts);
            }
            Err(e) => return Err(Box::new(e)),
        }
    }

    let elapsed = start.elapsed().as_secs_f64();
    let mib_processed = to_mib(total_bytes as f64);

    print_sorted_counts(total_counts);
    println!(
        "Processed {:.2} MiB in {:.3} seconds ({:.2} MiB/s)",
        mib_processed,
        elapsed,
        mib_processed / elapsed,
    );

    Ok(())
}
