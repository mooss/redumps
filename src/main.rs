use clap::Parser;
use std::fs;
use std::io::BufReader;
use std::time::Instant;

mod json;
use crate::json::{count_fields_from_reader, print_sorted_counts};

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

    let start = Instant::now();
    let counts = count_fields_from_reader(reader)?;
    let elapsed = start.elapsed().as_secs_f64();
    let mib_processed = to_mib(counts.nbytes as f64);

    print_sorted_counts(counts.map);
    println!(
        "Processed {:.2} MiB in {:.3} seconds ({:.2} MiB/s)",
        mib_processed,
        elapsed,
        mib_processed / elapsed,
    );

    Ok(())
}
