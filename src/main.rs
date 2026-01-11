use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version)]
#[command(about = "Process reddit dumps")]
struct Args {
    /// Input file.
    input: String,
}

mod conv;
use crate::conv::to_mib;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("Input: {:?}", args.input);

    let metadata = fs::metadata(&args.input)?;
    println!("Size: {} MiB", to_mib(metadata.len() as f64));

    Ok(())
}
