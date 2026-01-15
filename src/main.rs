use clap::Parser;
use std::fs;
use std::io::{BufRead, BufReader};
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

// Open the given file as a reader, with support for zstd archives.
fn create_reader(filename: &str) -> Result<Box<dyn BufRead>, Box<dyn std::error::Error>> {
    let file = fs::File::open(filename)?;

    match filename {
        f if f.ends_with(".zst") || f.ends_with(".zstd") => {
            let decoder = zstd::Decoder::new(file)?;
            Ok(Box::new(BufReader::new(decoder)))
        }
        _ => Ok(Box::new(BufReader::new(file))),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let reader = create_reader(&args.input)?;

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
