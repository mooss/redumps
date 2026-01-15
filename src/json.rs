use sonic_rs::{to_object_iter, ObjectJsonIter};
use std::borrow::Cow;
use std::collections::HashMap;
use std::io::BufRead;

use crate::io::foreach_line;
use crate::utils::Maybe;

pub type CountMap = HashMap<Cow<'static, str>, usize>;

fn count_fields(object: ObjectJsonIter, counts: &mut CountMap) {
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
pub struct FieldCounts {
    pub map: CountMap,
    pub nbytes: usize,
}

/// Read JSON lines from a BufRead source, count field occurrences, and return counts and total bytes.
pub fn count_fields_from_reader<R: BufRead>(reader: R) -> Maybe<FieldCounts> {
    // PERF: Cow<'static, str> is faster than String, probably because sonic_rs Cow<'_, str> and/or
    // because of borrow schenanigans.
    let mut total_counts: CountMap = HashMap::new();
    let mut nbytes: usize = 0;

    foreach_line(reader, |line| {
        nbytes += line.len();
        let iter = to_object_iter(line);
        count_fields(iter, &mut total_counts);
    })?;

    Ok(FieldCounts {
        map: total_counts,
        nbytes,
    })
}
