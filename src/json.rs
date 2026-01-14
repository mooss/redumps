use sonic_rs::{to_object_iter, ObjectJsonIter};
use std::collections::HashMap;
use std::error::Error;
use std::io::BufRead;

fn count_fields(object: ObjectJsonIter, counts: &mut HashMap<String, usize>) {
    // We ignore errors and only count valid fields.
    for (key, _) in object.filter_map(|res| res.ok()) {
        let key = key.to_string();
        // Don't clone the key unless absolutely necessary.
        if let Some(count) = counts.get_mut(&key) {
            *count += 1;
        } else {
            counts.insert(key, 1);
        }
    }
}

pub fn print_sorted_counts(counts: HashMap<String, usize>) {
    let mut entries: Vec<_> = counts.into_iter().collect();
    entries.sort_by(|a, b| b.1.cmp(&a.1));

    for (field, count) in entries {
        println!("{}: {}", field, count);
    }
}

/// Read JSON lines from a BufRead source, count field occurrences, and return counts and total bytes.
pub fn count_fields_from_reader<R: BufRead>(
    mut reader: R,
) -> Result<(HashMap<String, usize>, usize), Box<dyn Error>> {
    let mut total_counts: HashMap<String, usize> = HashMap::new();
    let mut total_bytes: usize = 0;
    let mut line = String::new();

    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => break, // EOF
            Ok(_) => {
                total_bytes += line.len();
                let iter = to_object_iter(line.as_str());
                count_fields(iter, &mut total_counts);
            }
            Err(e) => return Err(Box::new(e)),
        }
    }

    Ok((total_counts, total_bytes))
}
