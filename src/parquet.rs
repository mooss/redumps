pub mod column;
pub mod json_to_parquet;
pub mod writer;

use std::path::Path;

use crate::io::{foreach_line, open_file_or_zstd};
use crate::parquet::json_to_parquet::json_entry_to_parquet_row;
use crate::parquet::writer::ParquetBatchWriter;
use crate::utils::Maybe;

/// Convert a collection of JSON line files into a Parquet file, returning the total number of
/// processed bytes on success.
///
/// * `input_files` - Paths to JSON line files (plain or ZSTD-compressed).
/// * `output_path` - Directory where the resulting Parquet files will be placed.
///
/// The function creates one Parquet file per input file, preserving the original filename stem.
/// It also extracts metadata from the first successfully row to embed in the Parquet file's
/// key-value metadata.
pub fn run_to_parquet(input_files: Vec<String>, output_path: String) -> Maybe<usize> {
    let mut writer: Option<ParquetBatchWriter> = None;
    let mut metadata_found = false;
    let mut total_bytes: usize = 0;

    for in_path in input_files {
        let reader = open_file_or_zstd(&in_path)?;
        foreach_line(reader, |line| {
            total_bytes += line.len();

            if let Some(row) = json_entry_to_parquet_row(line) {
                if !metadata_found {
                    // Extract metadata from the first valid row.
                    let meta = vec![
                        (
                            "subreddit".to_string(),
                            row.subreddit.clone().unwrap_or_default(),
                        ),
                        (
                            "subreddit_id".to_string(),
                            row.subreddit_id.clone().unwrap_or_default(),
                        ),
                        (
                            "subreddit_name_prefixed".to_string(),
                            row.subreddit_name_prefixed.clone().unwrap_or_default(),
                        ),
                    ];

                    // Determine final output path.
                    let final_path = {
                        let stem = Path::new(&in_path)
                            .file_stem()
                            .unwrap_or_default()
                            .to_string_lossy();
                        format!("{}/{}.parquet", output_path, stem)
                    };

                    writer = Some(ParquetBatchWriter::new(&final_path, meta).unwrap());
                    metadata_found = true;
                }

                if let Some(w) = &mut writer {
                    w.write_row(row);
                }
            }
        })?;
    }

    if let Some(mut w) = writer {
        w.close()?;
    }
    Ok(total_bytes)
}
