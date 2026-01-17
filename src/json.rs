use crossbeam::channel::bounded;
use sonic_rs::{to_object_iter, ObjectJsonIter};
use std::borrow::Cow;
use std::collections::HashMap;
use std::io::BufRead;
use std::thread;

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

// Replace the existing count_fields_from_reader function with this parallel streaming version
pub fn count_fields_from_reader<R: BufRead>(mut reader: R) -> Maybe<FieldCounts> {
    let num_workers = 3;
    let (sender, receiver) = bounded::<String>(32);

    // Spawn worker threads.
    let mut workers = Vec::new();
    for _ in 0..num_workers {
        let receiver = receiver.clone();
        let handle = thread::spawn(move || {
            let mut counts: CountMap = HashMap::new();

            // Process lines until channel is closed
            while let Ok(line) = receiver.recv() {
                let iter = to_object_iter(line.as_str());
                count_fields(iter, &mut counts);
            }

            counts
        });
        workers.push(handle);
    }

    // Read and distribute lines.
    let mut total_bytes: usize = 0;
    foreach_line(&mut reader, |line| {
        total_bytes += line.len();
        sender.send(line.to_string())
    })?;

    // Close the channel and collect the results.
    drop(sender);
    let mut total_counts: CountMap = HashMap::new();
    for handle in workers {
        let local_counts = handle
            .join()
            .map_err(|e| std::io::Error::other(format!("Worker thread panicked: {:?}", e)))?;
        for (key, count) in local_counts {
            *total_counts.entry(key).or_insert(0) += count;
        }
    }

    Ok(FieldCounts {
        map: total_counts,
        nbytes: total_bytes,
    })
}
