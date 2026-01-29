use clap::{Parser, Subcommand};
use std::{io::Write, time::Instant};

mod io;
mod json;
mod json_to_parquet;
mod parquet;
mod utils;
use crate::io::{open_file_or_zstd, prepare_output_writer};
use crate::json::{CountMap, count_fields_from_reader};
use crate::utils::{Maybe, to_mib};

#[derive(Parser, Debug)]
#[command(version)]
#[command(about = "Process Reddit dumps")]
struct Args {
    /// Top-level command to run.
    #[command(subcommand)]
    command: Cmd,
}

/// Top-level commands.
#[derive(Subcommand, Debug, Clone)]
enum Cmd {
    /// Count JSON field occurrences in the given files.
    CountFields {
        /// Input files.
        #[arg(required = true)]
        input: Vec<String>,

        /// Output directory (if not provided, print to stdout).
        #[arg(short, long, default_value = "")]
        output: String,
    },

    /// Serialize JSON entries to Parquet.
    ToParquet {
        /// Input files.
        #[arg(required = true)]
        input: Vec<String>,

        /// Output directory (filenames will be deduced from input filenames).
        #[arg(short, long, required = true)]
        output: String,
    },
}

fn main() -> Maybe {
    let args = Args::parse();
    let start = Instant::now();
    let nbytes = run_cmd(args.command)?;
    let elapsed = start.elapsed().as_secs_f64();
    let mib_processed = to_mib(nbytes as f64);

    eprintln!(
        "Processed {:.2} MiB in {:.3} seconds ({:.2} MiB/s)",
        mib_processed,
        elapsed,
        mib_processed / elapsed,
    );

    Ok(())
}

/// Top-level command runner.
fn run_cmd(cmd: Cmd) -> Maybe<usize> {
    match cmd {
        Cmd::CountFields { input, output } => count_fields_cmd(input, output),
        Cmd::ToParquet { input, output } => {
            let nbytes = crate::parquet::run_to_parquet(input, output)?;
            Ok(nbytes)
        }
    }
}

/// Command handler for `count-fields`.
fn count_fields_cmd(input_files: Vec<String>, out_dir: String) -> Maybe<usize> {
    let mut nbytes = 0usize;
    for in_path in input_files {
        nbytes += count_fields_impl(in_path, out_dir.clone())?;
    }
    Ok(nbytes)
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
