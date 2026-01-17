use clap::Parser;
use std::{io::Write, time::Instant};

mod io;
mod json;
mod utils;
use crate::io::{open_file_or_zstd, prepare_output_writer};
use crate::json::{count_fields_from_reader, CountMap};
use crate::utils::{to_mib, Maybe};

#[derive(Parser, Debug)]
#[command(version)]
#[command(about = "Process reddit dumps")]
struct Args {
    /// Input files.
    input: Vec<String>,

    /// Output directory (if not provided, print to stdout).
    #[arg(short, long, default_value = "")]
    output: String,
}

fn main() -> Maybe {
    let args = Args::parse();

    let mut total_nbytes = 0usize;
    let start = Instant::now();

    for input_path in &args.input {
        total_nbytes += count_fields_impl(input_path.clone(), args.output.clone())?;
    }

    let elapsed = start.elapsed().as_secs_f64();
    let mib_processed = to_mib(total_nbytes as f64);

    eprintln!(
        "{:.2} MiB processed in {:.3} seconds ({:.2} MiB/s)",
        mib_processed,
        elapsed,
        mib_processed / elapsed,
    );

    Ok(())
}

/////////////////////
// Local utilities //

pub fn count_fields_impl(input_path: String, output_path: String) -> Maybe<usize> {
    let reader = open_file_or_zstd(&input_path)?;
    let counts = count_fields_from_reader(reader)?;
    let mut writer = prepare_output_writer(output_path, input_path, ".fields.json")?;
    print_sorted_counts(counts.map, &mut writer)?;
    Ok(counts.nbytes)
}

pub fn print_sorted_counts<W: Write>(counts: CountMap, writer: &mut W) -> std::io::Result<()> {
    let mut entries: Vec<_> = counts.into_iter().collect();
    entries.sort_by(|a, b| b.1.cmp(&a.1));

    writeln!(writer, "{{")?;
    if let Some((last, rest)) = entries.split_last() {
        for (field, count) in rest {
            writeln!(writer, "  \"{}\": {},", field, count)?;
        }
        writeln!(writer, "  \"{}\": {}", last.0, last.1)?;
    }
    writeln!(writer, "}}")?;

    Ok(())
}
