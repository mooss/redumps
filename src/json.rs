use serde_json::Value;
use std::collections::HashMap;
use std::io::BufRead;

fn count_fields(value: &Value, counts: &mut HashMap<String, usize>) {
    if let Value::Object(map) = value {
        for (key, _) in map {
            // Don't clone the key unless absolutely necessary.
            if let Some(count) = counts.get_mut(key.as_str()) {
                *count += 1;
            } else {
                counts.insert(key.clone(), 1);
            }
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
) -> Result<(HashMap<String, usize>, usize), Box<dyn std::error::Error>> {
    let mut total_counts: HashMap<String, usize> = HashMap::new();
    let mut total_bytes: usize = 0;
    let mut line = String::new();

    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => break, // EOF
            Ok(_) => {
                total_bytes += line.len();
                let json: Value = serde_json::from_str(&line)?;
                count_fields(&json, &mut total_counts);
            }
            Err(e) => return Err(Box::new(e)),
        }
    }

    Ok((total_counts, total_bytes))
}
