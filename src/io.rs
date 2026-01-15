use std::{
    fs::{create_dir_all, File},
    io::{stdout, BufRead, BufReader, Write},
    path::Path,
};

use crate::utils::Maybe;

pub fn foreach_line<R: BufRead, F>(mut reader: R, mut f: F) -> Maybe
where
    F: FnMut(&str),
{
    // PERF: Using read_line this way instead of iterating with reader.lines appears to be faster,
    // it looks like reader.lines is doing more allocations.
    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => f(&line),
            Err(e) => return Err(Box::new(e)),
        }
    }
    Ok(())
}

// Open the given file as a reader, with support for zstd archives.
pub fn open_file_or_zstd(filename: &str) -> Maybe<Box<dyn BufRead>> {
    let file = File::open(filename)?;

    match filename {
        f if f.ends_with(".zst") || f.ends_with(".zstd") => {
            let decoder = zstd::Decoder::new(file)?;
            Ok(Box::new(BufReader::new(decoder)))
        }
        _ => Ok(Box::new(BufReader::new(file))),
    }
}

/// Prepare an output writer based on the provided output directory and input filename.
///
/// Defaults to stdout if `output_dirname` is empty.
/// Will create the directory if it does not exist.
pub fn prepare_output_writer(
    output_dirname: String,
    input_filename: &str,
    suffix: &str,
) -> Maybe<Box<dyn Write>> {
    if output_dirname.is_empty() {
        return Ok(Box::new(stdout()));
    }

    let input_path = Path::new(input_filename);
    let input_stem = input_path
        .file_stem()
        .unwrap_or_else(|| input_filename.as_ref())
        .to_string_lossy();
    let output_filename = format!("{}/{}{}", &output_dirname, input_stem, suffix);

    create_dir_all(&output_dirname)?;
    let file = File::create(&output_filename)?;
    Ok(Box::new(file))
}
