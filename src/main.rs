mod conv;
mod io;
mod json;

use clap::Parser;
use std::time::Instant;

use crate::conv::to_mib;
use crate::io::open_file_or_zstd;
use crate::json::{count_fields_from_reader, CountMap};

#[derive(Parser, Debug)]
#[command(version)]
#[command(about = "Process reddit dumps")]
struct Args {
    /// Input file.
    input: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let reader = open_file_or_zstd(&args.input)?;

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

/////////////////////
// Local utilities //

pub fn print_sorted_counts(counts: CountMap) {
    let mut entries: Vec<_> = counts.into_iter().collect();
    entries.sort_by(|a, b| b.1.cmp(&a.1));

    for (field, count) in entries {
        println!("{}: {}", field, count);
    }
}
