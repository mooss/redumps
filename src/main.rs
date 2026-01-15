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
    /// Input file.
    input: String,

    /// Output directory (if not provided, print to stdout).
    #[arg(short, long)]
    output: Option<String>,
}

fn main() -> Maybe {
    let args = Args::parse();

    let reader = open_file_or_zstd(&args.input)?;

    let mut writer = prepare_output_writer(args.output.as_deref(), &args.input)?;
    let start = Instant::now();
    let counts = count_fields_from_reader(reader)?;
    let elapsed = start.elapsed().as_secs_f64();
    let mib_processed = to_mib(counts.nbytes as f64);

    print_sorted_counts(counts.map, &mut writer)?;
    eprintln!(
        "Processed {:.2} MiB in {:.3} seconds ({:.2} MiB/s)",
        mib_processed,
        elapsed,
        mib_processed / elapsed,
    );

    Ok(())
}

/////////////////////
// Local utilities //

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
