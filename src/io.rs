use std::{
    error::Error,
    fs,
    io::{BufRead, BufReader},
};

pub fn readlines<R: BufRead, F>(mut reader: R, mut f: F) -> Result<(), Box<dyn Error>>
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
pub fn open_file_or_zstd(filename: &str) -> Result<Box<dyn BufRead>, Box<dyn std::error::Error>> {
    let file = fs::File::open(filename)?;

    match filename {
        f if f.ends_with(".zst") || f.ends_with(".zstd") => {
            let decoder = zstd::Decoder::new(file)?;
            Ok(Box::new(BufReader::new(decoder)))
        }
        _ => Ok(Box::new(BufReader::new(file))),
    }
}
