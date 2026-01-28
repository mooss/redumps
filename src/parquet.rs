use std::fs::File;
use std::path::Path;
use std::sync::Arc;

use arrow::array::{ArrayBuilder, Int64Builder, StringBuilder};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use parquet::arrow::ArrowWriter;
use parquet::file::metadata::KeyValue;
use parquet::file::properties::WriterProperties;

use crate::io::{foreach_line, open_file_or_zstd};
use crate::json::{extract_parquet_row, ParquetRow};
use crate::utils::Maybe;

const BATCH_SIZE: usize = 64_000;

pub fn run_to_parquet(input_files: Vec<String>, output_path: String) -> Maybe<usize> {
    let mut writer: Option<ParquetWriterWrapper> = None;
    let mut metadata_found = false;
    let mut total_bytes: usize = 0;

    for in_path in input_files {
        let reader = open_file_or_zstd(&in_path)?;
        foreach_line(reader, |line| {
            total_bytes += line.len();

            if let Some(row) = extract_parquet_row(line) {
                if !metadata_found {
                    // Extract metadata from the first valid row
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

                    // Determine final output path
                    let final_path = if output_path.is_empty() {
                        "output.parquet".to_string()
                    } else {
                        let p = Path::new(&output_path);
                        if p.is_dir() {
                            let stem = Path::new(&in_path)
                                .file_stem()
                                .unwrap_or_default()
                                .to_string_lossy();
                            format!("{}/{}.parquet", output_path, stem)
                        } else {
                            output_path.clone()
                        }
                    };

                    writer = Some(ParquetWriterWrapper::new(&final_path, meta).unwrap());
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

struct ParquetWriterWrapper {
    writer: Option<ArrowWriter<File>>,
    schema: Schema,
    score_builder: Int64Builder,
    author_builder: StringBuilder,
    title_builder: StringBuilder,
}

impl ParquetWriterWrapper {
    fn new(path: &str, metadata: Vec<(String, String)>) -> Maybe<Self> {
        let file = File::create(path)?;
        let schema = Schema::new(vec![
            Field::new("score", DataType::Int64, true),
            Field::new("author", DataType::Utf8, true),
            Field::new("title", DataType::Utf8, true),
        ]);

        let kv_metadata: Vec<KeyValue> = metadata
            .into_iter()
            .map(|(k, v)| KeyValue {
                key: k,
                value: Some(v),
            })
            .collect();

        let props = WriterProperties::builder()
            .set_key_value_metadata(Some(kv_metadata))
            .build();

        let writer = ArrowWriter::try_new(file, Arc::new(schema.clone()), Some(props))?;

        Ok(Self {
            writer: Some(writer),
            schema,
            score_builder: Int64Builder::new(),
            author_builder: StringBuilder::new(),
            title_builder: StringBuilder::new(),
        })
    }

    fn write_row(&mut self, row: ParquetRow) {
        self.score_builder.append_option(row.score);
        self.author_builder.append_option(row.author.as_deref());
        self.title_builder.append_option(row.title.as_deref());

        if self.score_builder.len() >= BATCH_SIZE {
            self.flush_batch();
        }
    }

    fn flush_batch(&mut self) {
        if self.score_builder.is_empty() {
            return;
        }

        let score_array = self.score_builder.finish();
        let author_array = self.author_builder.finish();
        let title_array = self.title_builder.finish();

        let batch = RecordBatch::try_new(
            Arc::new(self.schema.clone()),
            vec![
                Arc::new(score_array),
                Arc::new(author_array),
                Arc::new(title_array),
            ],
        )
        .unwrap();

        self.writer.as_mut().unwrap().write(&batch).unwrap();

        // Reset builders
        self.score_builder = Int64Builder::new();
        self.author_builder = StringBuilder::new();
        self.title_builder = StringBuilder::new();
    }

    fn close(&mut self) -> Maybe<()> {
        self.flush_batch();
        self.writer.take().unwrap().close()?;
        Ok(())
    }
}
