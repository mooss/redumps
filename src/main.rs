use clap::Parser;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};

mod json;
use crate::json::{count_fields, print_sorted_counts};

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
    let reader = BufReader::new(file);

    let mut total_counts: HashMap<String, usize> = HashMap::new();

    for line_res in reader.lines() {
        let line = line_res?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let json: Value = serde_json::from_str(trimmed)?;
        let counts = count_fields(&json);
        for (key, value) in counts {
            *total_counts.entry(key).or_insert(0) += value;
        }
    }

    print_sorted_counts(total_counts);
    let metadata = fs::metadata(&args.input)?;
    println!("Size: {} MiB", to_mib(metadata.len() as f64));

    Ok(())
}
