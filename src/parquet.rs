use std::fs::File;
use std::path::Path;
use std::sync::Arc;

use arrow::array::{ArrayBuilder, BooleanBuilder, Float64Builder, Int64Builder, StringBuilder};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use parquet::arrow::ArrowWriter;
use parquet::file::metadata::KeyValue;
use parquet::file::properties::WriterProperties;

use crate::io::{foreach_line, open_file_or_zstd};
use crate::json_to_parquet::{ParquetRow, json_entry_to_parquet_row};
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

            if let Some(row) = json_entry_to_parquet_row(line) {
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

    // Field builders (maintain alphabetical order).
    author_builder: StringBuilder,
    created_utc_builder: Int64Builder,
    crosspost_parent_builder: StringBuilder,
    downs_builder: Int64Builder,
    id_builder: StringBuilder,
    is_gallery_builder: BooleanBuilder,
    is_self_builder: BooleanBuilder,
    is_video_builder: BooleanBuilder,
    media_builder: StringBuilder,
    num_comments_builder: Int64Builder,
    over_18_builder: BooleanBuilder,
    permalink_builder: StringBuilder,
    score_builder: Int64Builder,
    secure_media_builder: StringBuilder,
    selftext_builder: StringBuilder,
    title_builder: StringBuilder,
    upvote_ratio_builder: Float64Builder,
    ups_builder: Int64Builder,
    url_builder: StringBuilder,
}

impl ParquetWriterWrapper {
    fn new(path: &str, metadata: Vec<(String, String)>) -> Maybe<Self> {
        let file = File::create(path)?;
        let schema = Schema::new(vec![
            Field::new("author", DataType::Utf8, true),
            Field::new("created_utc", DataType::Int64, true),
            Field::new("crosspost_parent", DataType::Utf8, true),
            Field::new("downs", DataType::Int64, true),
            Field::new("id", DataType::Utf8, true),
            Field::new("is_gallery", DataType::Boolean, true),
            Field::new("is_self", DataType::Boolean, true),
            Field::new("is_video", DataType::Boolean, true),
            Field::new("media", DataType::Utf8, true),
            Field::new("num_comments", DataType::Int64, true),
            Field::new("over_18", DataType::Boolean, true),
            Field::new("permalink", DataType::Utf8, true),
            Field::new("score", DataType::Int64, true),
            Field::new("secure_media", DataType::Utf8, true),
            Field::new("selftext", DataType::Utf8, true),
            Field::new("title", DataType::Utf8, true),
            Field::new("upvote_ratio", DataType::Float64, true),
            Field::new("ups", DataType::Int64, true),
            Field::new("url", DataType::Utf8, true),
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
            author_builder: StringBuilder::new(),
            created_utc_builder: Int64Builder::new(),
            crosspost_parent_builder: StringBuilder::new(),
            downs_builder: Int64Builder::new(),
            id_builder: StringBuilder::new(),
            is_gallery_builder: BooleanBuilder::new(),
            is_self_builder: BooleanBuilder::new(),
            is_video_builder: BooleanBuilder::new(),
            media_builder: StringBuilder::new(),
            num_comments_builder: Int64Builder::new(),
            over_18_builder: BooleanBuilder::new(),
            permalink_builder: StringBuilder::new(),
            score_builder: Int64Builder::new(),
            secure_media_builder: StringBuilder::new(),
            selftext_builder: StringBuilder::new(),
            title_builder: StringBuilder::new(),
            upvote_ratio_builder: Float64Builder::new(),
            ups_builder: Int64Builder::new(),
            url_builder: StringBuilder::new(),
        })
    }

    fn write_row(&mut self, row: ParquetRow) {
        self.author_builder.append_option(row.author.as_deref());
        self.created_utc_builder.append_option(row.created_utc);
        self.crosspost_parent_builder
            .append_option(row.crosspost_parent.as_deref());
        self.downs_builder.append_option(row.downs);
        self.id_builder.append_option(row.id.as_deref());
        self.is_gallery_builder.append_option(row.is_gallery);
        self.is_self_builder.append_option(row.is_self);
        self.is_video_builder.append_option(row.is_video);
        self.media_builder.append_option(row.media.as_deref());
        self.num_comments_builder.append_option(row.num_comments);
        self.over_18_builder.append_option(row.over_18);
        self.permalink_builder
            .append_option(row.permalink.as_deref());
        self.score_builder.append_option(row.score);
        self.secure_media_builder
            .append_option(row.secure_media.as_deref());
        self.selftext_builder.append_option(row.selftext.as_deref());
        self.title_builder.append_option(row.title.as_deref());
        self.upvote_ratio_builder.append_option(row.upvote_ratio);
        self.ups_builder.append_option(row.ups);
        self.url_builder.append_option(row.url.as_deref());

        // Flush when we have enough rows (using score_builder length as a proxy)
        if self.score_builder.len() >= BATCH_SIZE {
            self.flush_batch();
        }
    }

    fn flush_batch(&mut self) {
        if self.score_builder.is_empty() {
            return;
        }

        // Build arrays in the same order as the schema.
        let arrays: Vec<Arc<dyn arrow::array::Array>> = vec![
            Arc::new(self.author_builder.finish()),
            Arc::new(self.created_utc_builder.finish()),
            Arc::new(self.crosspost_parent_builder.finish()),
            Arc::new(self.downs_builder.finish()),
            Arc::new(self.id_builder.finish()),
            Arc::new(self.is_gallery_builder.finish()),
            Arc::new(self.is_self_builder.finish()),
            Arc::new(self.is_video_builder.finish()),
            Arc::new(self.media_builder.finish()),
            Arc::new(self.num_comments_builder.finish()),
            Arc::new(self.over_18_builder.finish()),
            Arc::new(self.permalink_builder.finish()),
            Arc::new(self.score_builder.finish()),
            Arc::new(self.secure_media_builder.finish()),
            Arc::new(self.selftext_builder.finish()),
            Arc::new(self.title_builder.finish()),
            Arc::new(self.upvote_ratio_builder.finish()),
            Arc::new(self.ups_builder.finish()),
            Arc::new(self.url_builder.finish()),
        ];

        let batch = RecordBatch::try_new(Arc::new(self.schema.clone()), arrays).unwrap();
        self.writer.as_mut().unwrap().write(&batch).unwrap();

        // Reset all builders for the next batch.
        self.author_builder = StringBuilder::new();
        self.created_utc_builder = Int64Builder::new();
        self.crosspost_parent_builder = StringBuilder::new();
        self.downs_builder = Int64Builder::new();
        self.id_builder = StringBuilder::new();
        self.is_gallery_builder = BooleanBuilder::new();
        self.is_self_builder = BooleanBuilder::new();
        self.is_video_builder = BooleanBuilder::new();
        self.media_builder = StringBuilder::new();
        self.num_comments_builder = Int64Builder::new();
        self.over_18_builder = BooleanBuilder::new();
        self.permalink_builder = StringBuilder::new();
        self.score_builder = Int64Builder::new();
        self.secure_media_builder = StringBuilder::new();
        self.selftext_builder = StringBuilder::new();
        self.title_builder = StringBuilder::new();
        self.upvote_ratio_builder = Float64Builder::new();
        self.ups_builder = Int64Builder::new();
        self.url_builder = StringBuilder::new();
    }

    fn close(&mut self) -> Maybe<()> {
        self.flush_batch();
        self.writer.take().unwrap().close()?;
        Ok(())
    }
}
