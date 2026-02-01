pub mod json_to_parquet;

use std::fs::File;
use std::path::Path;
use std::sync::Arc;

use arrow::array::{ArrayBuilder, BooleanBuilder, Float64Builder, Int64Builder, StringBuilder};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;

use parquet::arrow::ArrowWriter;
use parquet::basic::{Compression, ZstdLevel};
use parquet::file::metadata::KeyValue;
use parquet::file::properties::WriterProperties;

use crate::io::{foreach_line, open_file_or_zstd};
use crate::parquet::json_to_parquet::{ParquetRow, json_entry_to_parquet_row};
use crate::utils::Maybe;

const BATCH_SIZE: usize = 64_000;

/// Column encapsulates a column's builder, schema field, and value extractor.
enum Column {
    String(StringBuilder, Field, fn(&ParquetRow) -> Option<&str>),
    Int64(Int64Builder, Field, fn(&ParquetRow) -> Option<i64>),
    Boolean(BooleanBuilder, Field, fn(&ParquetRow) -> Option<bool>),
    Float64(Float64Builder, Field, fn(&ParquetRow) -> Option<f64>),
}

impl Column {
    fn new_string(name: &str, f: fn(&ParquetRow) -> Option<&str>) -> Self {
        Self::String(
            StringBuilder::new(),
            Field::new(name, DataType::Utf8, true),
            f,
        )
    }

    fn new_int64(name: &str, f: fn(&ParquetRow) -> Option<i64>) -> Self {
        Self::Int64(
            Int64Builder::new(),
            Field::new(name, DataType::Int64, true),
            f,
        )
    }

    fn new_boolean(name: &str, f: fn(&ParquetRow) -> Option<bool>) -> Self {
        Self::Boolean(
            BooleanBuilder::new(),
            Field::new(name, DataType::Boolean, true),
            f,
        )
    }

    fn new_float64(name: &str, f: fn(&ParquetRow) -> Option<f64>) -> Self {
        Self::Float64(
            Float64Builder::new(),
            Field::new(name, DataType::Float64, true),
            f,
        )
    }

    fn field(&self) -> Field {
        match self {
            Self::String(_, field, _) => field.clone(),
            Self::Int64(_, field, _) => field.clone(),
            Self::Boolean(_, field, _) => field.clone(),
            Self::Float64(_, field, _) => field.clone(),
        }
    }

    fn append(&mut self, row: &ParquetRow) {
        match self {
            Self::String(builder, _, extractor) => builder.append_option(extractor(row)),
            Self::Int64(builder, _, extractor) => builder.append_option(extractor(row)),
            Self::Boolean(builder, _, extractor) => builder.append_option(extractor(row)),
            Self::Float64(builder, _, extractor) => builder.append_option(extractor(row)),
        }
    }

    fn finish(&mut self) -> Arc<dyn arrow::array::Array> {
        match self {
            Self::String(builder, _, _) => Arc::new(builder.finish()),
            Self::Int64(builder, _, _) => Arc::new(builder.finish()),
            Self::Boolean(builder, _, _) => Arc::new(builder.finish()),
            Self::Float64(builder, _, _) => Arc::new(builder.finish()),
        }
    }

    fn reset(&mut self) {
        match self {
            Self::String(builder, _, _) => *builder = StringBuilder::new(),
            Self::Int64(builder, _, _) => *builder = Int64Builder::new(),
            Self::Boolean(builder, _, _) => *builder = BooleanBuilder::new(),
            Self::Float64(builder, _, _) => *builder = Float64Builder::new(),
        }
    }

    fn len(&self) -> usize {
        match self {
            Self::String(builder, _, _) => builder.len(),
            Self::Int64(builder, _, _) => builder.len(),
            Self::Boolean(builder, _, _) => builder.len(),
            Self::Float64(builder, _, _) => builder.len(),
        }
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub fn run_to_parquet(input_files: Vec<String>, output_path: String) -> Maybe<usize> {
    let mut writer: Option<ParquetWriterWrapper> = None;
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
    columns: Vec<Column>,
}

impl ParquetWriterWrapper {
    fn new(path: &str, metadata: Vec<(String, String)>) -> Maybe<Self> {
        let file = File::create(path)?;

        let columns = vec![
            Column::new_string("author", |r| r.author.as_deref()),
            Column::new_int64("created_utc", |r| r.created_utc),
            Column::new_string("crosspost_parent", |r| r.crosspost_parent.as_deref()),
            Column::new_int64("downs", |r| r.downs),
            Column::new_string("id", |r| r.id.as_deref()),
            Column::new_boolean("is_gallery", |r| r.is_gallery),
            Column::new_boolean("is_self", |r| r.is_self),
            Column::new_boolean("is_video", |r| r.is_video),
            Column::new_string("media", |r| r.media.as_deref()),
            Column::new_int64("num_comments", |r| r.num_comments),
            Column::new_boolean("over_18", |r| r.over_18),
            Column::new_string("permalink", |r| r.permalink.as_deref()),
            Column::new_int64("score", |r| r.score),
            Column::new_string("secure_media", |r| r.secure_media.as_deref()),
            Column::new_string("selftext", |r| r.selftext.as_deref()),
            Column::new_string("title", |r| r.title.as_deref()),
            Column::new_float64("upvote_ratio", |r| r.upvote_ratio),
            Column::new_int64("ups", |r| r.ups),
            Column::new_string("url", |r| r.url.as_deref()),
        ];

        let schema = Schema::new(columns.iter().map(|c| c.field()).collect::<Vec<_>>());

        let kv_metadata: Vec<KeyValue> = metadata
            .into_iter()
            .map(|(k, v)| KeyValue {
                key: k,
                value: Some(v),
            })
            .collect();

        let props = WriterProperties::builder()
            .set_key_value_metadata(Some(kv_metadata))
            .set_compression(Compression::ZSTD(ZstdLevel::try_new(9).unwrap()))
            .build();

        let writer = ArrowWriter::try_new(file, Arc::new(schema.clone()), Some(props))?;

        Ok(Self {
            writer: Some(writer),
            schema,
            columns,
        })
    }

    fn write_row(&mut self, row: ParquetRow) {
        for col in &mut self.columns {
            col.append(&row);
        }

        if self.columns.first().is_some_and(|c| c.len() >= BATCH_SIZE) {
            self.flush_batch();
        }
    }

    fn flush_batch(&mut self) {
        if self.columns.first().is_none_or(|c| c.is_empty()) {
            return;
        }

        let arrays: Vec<Arc<dyn arrow::array::Array>> =
            self.columns.iter_mut().map(|c| c.finish()).collect();

        let batch = RecordBatch::try_new(Arc::new(self.schema.clone()), arrays).unwrap();
        self.writer.as_mut().unwrap().write(&batch).unwrap();

        for col in &mut self.columns {
            col.reset();
        }
    }

    fn close(&mut self) -> Maybe<()> {
        self.flush_batch();
        self.writer.take().unwrap().close()?;
        Ok(())
    }
}
