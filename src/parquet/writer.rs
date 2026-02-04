use std::fs::File;
use std::sync::Arc;

use arrow::datatypes::Schema;
use arrow::record_batch::RecordBatch;
use parquet::arrow::ArrowWriter;
use parquet::basic::{Compression, ZstdLevel};
use parquet::file::metadata::KeyValue;
use parquet::file::properties::WriterProperties;

use crate::parquet::column::Column;
use crate::parquet::json::ParquetRow;
use crate::utils::Maybe;

/// Number of rows to accumulate before flushing a batch to disk.
const BATCH_SIZE: usize = 64_000;

pub struct ParquetBatchWriter {
    writer: Option<ArrowWriter<File>>,
    schema: Schema,
    columns: Vec<Column>,
}

impl ParquetBatchWriter {
    /// Create a new writer for `path`, embedding the supplied key-value `metadata`.
    pub fn new(path: &str, metadata: Vec<(String, String)>) -> Maybe<Self> {
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

        let schema = Schema::new(
            columns
                .iter()
                .map(|c| c.field().clone())
                .collect::<Vec<_>>(),
        );

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

    /// Append a `ParquetRow` to the current batch.
    pub fn write_row(&mut self, row: ParquetRow) {
        for col in &mut self.columns {
            col.append(&row);
        }

        if self.columns.first().is_some_and(|c| c.len() >= BATCH_SIZE) {
            self.flush_batch();
        }
    }

    /// Write the accumulated batch to the output file.
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

    /// Write any remaining row and close the output file.
    pub fn close(&mut self) -> Maybe<()> {
        self.flush_batch();
        self.writer.take().unwrap().close()?;
        Ok(())
    }
}
