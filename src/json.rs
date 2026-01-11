use serde_json::Value;
use std::collections::HashMap;

/// Increment `counts` with the field occurrences found in `json`.
pub fn count_fields_into(value: &Value, counts: &mut HashMap<String, usize>) {
    count_fields_recursive(value, counts);
}

fn count_fields_recursive(value: &Value, counts: &mut HashMap<String, usize>) {
    match value {
        Value::Object(map) => {
            for (key, val) in map {
                // Don't clone the key unless absolutely necessary.
                if let Some(count) = counts.get_mut(key.as_str()) {
                    *count += 1;
                } else {
                    counts.insert(key.clone(), 1);
                }
                count_fields_recursive(val, counts);
            }
        }
        Value::Array(arr) => {
            for item in arr {
                count_fields_recursive(item, counts);
            }
        }
        _ => {}
    }
}

pub fn print_sorted_counts(counts: HashMap<String, usize>) {
    let mut entries: Vec<_> = counts.into_iter().collect();
    entries.sort_by(|a, b| b.1.cmp(&a.1));

    for (field, count) in entries {
        println!("{}: {}", field, count);
    }
}
