use sonic_rs::{to_object_iter, ObjectJsonIter};
use std::borrow::Cow;
use std::collections::HashMap;
use std::error::Error;
use std::io::BufRead;

fn count_fields(object: ObjectJsonIter, counts: &mut HashMap<Cow<'static, str>, usize>) {
    // We ignore errors and only count valid fields.
    for (key, _) in object.filter_map(|res| res.ok()) {
        let key = Cow::<'static, str>::Owned(key.into_owned());

        // PERF: The entry API is slower.
        if let Some(count) = counts.get_mut(&key) {
            *count += 1;
        } else {
            counts.insert(key, 1);
        }
    }
}

pub fn print_sorted_counts(counts: HashMap<Cow<'static, str>, usize>) {
    let mut entries: Vec<_> = counts.into_iter().collect();
    entries.sort_by(|a, b| b.1.cmp(&a.1));

    for (field, count) in entries {
        println!("{}: {}", field, count);
    }
}

pub struct FieldCounts {
    pub map: HashMap<Cow<'static, str>, usize>,
    pub nbytes: usize,
}

/// Read JSON lines from a BufRead source, count field occurrences, and return counts and total bytes.
pub fn count_fields_from_reader<R: BufRead>(reader: R) -> Result<FieldCounts, Box<dyn Error>> {
    // Cow<'static, str> is faster than String, probably because sonic_rs Cow<'_, str> and/or
    // because of borrow schenanigans.
    let mut total_counts: HashMap<Cow<'static, str>, usize> = HashMap::new();
    let mut nbytes: usize = 0;

    readlines(reader, |line| {
        nbytes += line.len();
        let iter = to_object_iter(line);
        count_fields(iter, &mut total_counts);
    })?;

    Ok(FieldCounts {
        map: total_counts,
        nbytes,
    })
}

fn readlines<R: BufRead, F>(mut reader: R, mut f: F) -> Result<(), Box<dyn Error>>
where
    F: FnMut(&str),
{
    // Using read_line this way instead of iterating with reader.lines appears to be faster, it
    // looks like reader.lines is doing more allocations.
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
