use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version)]
#[command(about = "Process reddit dumps")]
struct Args {
    /// Input file.
    input: String,

    /// Output file.
    #[arg(short, long)]
    output: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("Input: {:?}", args.input);
    println!("Output: {:?}", args.output);

    Ok(())
}
