use sonic_rs::{JsonType, JsonValueTrait, ObjectJsonIter, to_object_iter};
use std::borrow::Cow;
use std::collections::HashMap;
use std::io::BufRead;

use crate::io::foreach_line;
use crate::utils::Maybe;

pub type CountMap = HashMap<Cow<'static, str>, usize>;

fn json_type_to_str(typ: JsonType) -> &'static str {
    match typ {
        JsonType::Null => "Null",
        JsonType::Boolean => "Boolean",
        JsonType::Number => "Number",
        JsonType::String => "String",
        JsonType::Object => "Object",
        JsonType::Array => "Array",
    }
}

fn count_fields(object: ObjectJsonIter, counts: &mut CountMap) {
    // We ignore errors and only count valid fields.
    for (key, value) in object.filter_map(|res| res.ok()) {
        let key = Cow::<'static, str>::Owned(key.into_owned());
        let key = key + "/" + json_type_to_str(value.get_type());

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

#[derive(Debug, Default)]
pub struct ParquetRow {
    pub author: Option<String>,
    pub created_utc: Option<i64>,
    pub crosspost_parent: Option<String>,
    pub downs: Option<i64>,
    pub id: Option<String>,
    pub is_gallery: Option<bool>,
    pub is_self: Option<bool>,
    pub is_video: Option<bool>,
    pub media: Option<String>,
    pub num_comments: Option<i64>,
    pub over_18: Option<bool>,
    pub permalink: Option<String>,
    pub score: Option<i64>,
    pub secure_media: Option<String>,
    pub selftext: Option<String>,
    pub subreddit: Option<String>,
    pub subreddit_id: Option<String>,
    pub subreddit_name_prefixed: Option<String>,
    pub title: Option<String>,
    pub upvote_ratio: Option<f64>,
    pub ups: Option<i64>,
    pub url: Option<String>,
}

pub fn extract_parquet_row(line: &str) -> Option<ParquetRow> {
    let iter = to_object_iter(line);
    let mut row = ParquetRow::default();
    let mut skip = false;

    for (key, value) in iter.filter_map(|res| res.ok()) {
        match &*key {
            "author" => {
                if value.get_type() == JsonType::String {
                    row.author = value.as_str().map(|s| s.to_string());
                }
            }
            "created_utc" => match value.get_type() {
                JsonType::Number => {
                    row.created_utc = value.as_i64();
                }
                JsonType::String => {
                    if let Some(s) = value.as_str()
                        && let Ok(num) = s.parse::<i64>()
                    {
                        row.created_utc = Some(num);
                    }
                }
                _ => {}
            },
            "crosspost_parent" => {
                if value.get_type() == JsonType::String {
                    row.crosspost_parent = value.as_str().map(|s| s.to_string());
                }
            }
            "downs" => {
                if value.get_type() == JsonType::Number {
                    row.downs = value.as_i64();
                }
            }
            "id" => {
                if value.get_type() == JsonType::String {
                    row.id = value.as_str().map(|s| s.to_string());
                }
            }
            "is_gallery" => {
                if value.get_type() == JsonType::Boolean {
                    row.is_gallery = value.as_bool();
                }
            }
            "is_self" => {
                if value.get_type() == JsonType::Boolean {
                    row.is_self = value.as_bool();
                }
            }
            "is_video" => {
                if value.get_type() == JsonType::Boolean {
                    row.is_video = value.as_bool();
                }
            }
            "media" => {
                match value.get_type() {
                    JsonType::Null => {
                        row.media = Some("null".to_string());
                    }
                    JsonType::Object => {
                        // Store the raw JSON representation as a string
                        row.media = Some(value.to_string());
                    }
                    _ => {}
                }
            }
            "num_comments" => {
                if value.get_type() == JsonType::Number {
                    row.num_comments = value.as_i64();
                }
            }
            "over_18" => {
                if value.get_type() == JsonType::Boolean {
                    row.over_18 = value.as_bool();
                }
            }
            "permalink" => {
                if value.get_type() == JsonType::String {
                    row.permalink = value.as_str().map(|s| s.to_string());
                }
            }
            "score" => {
                if value.get_type() == JsonType::Number {
                    row.score = value.as_i64();
                }
            }
            "secure_media" => match value.get_type() {
                JsonType::Null => {
                    row.secure_media = Some("null".to_string());
                }
                JsonType::Object => {
                    row.secure_media = Some(value.to_string());
                }
                _ => {}
            },
            "selftext" => {
                if let Some(s) = value.as_str() {
                    if s == "[deleted]" {
                        skip = true;
                        break;
                    }
                    row.selftext = Some(s.to_string());
                }
            }
            "subreddit" => {
                if value.get_type() == JsonType::String {
                    row.subreddit = value.as_str().map(|s| s.to_string());
                }
            }
            "subreddit_id" => {
                if value.get_type() == JsonType::String {
                    row.subreddit_id = value.as_str().map(|s| s.to_string());
                }
            }
            "subreddit_name_prefixed" => {
                if value.get_type() == JsonType::String {
                    row.subreddit_name_prefixed = value.as_str().map(|s| s.to_string());
                }
            }
            "title" => {
                if value.get_type() == JsonType::String {
                    row.title = value.as_str().map(|s| s.to_string());
                }
            }
            "upvote_ratio" => {
                if value.get_type() == JsonType::Number {
                    row.upvote_ratio = value.as_f64();
                }
            }
            "ups" => {
                if value.get_type() == JsonType::Number {
                    row.ups = value.as_i64();
                }
            }
            "url" => {
                if value.get_type() == JsonType::String {
                    row.url = value.as_str().map(|s| s.to_string());
                }
            }
            _ => {}
        }
    }

    if skip { None } else { Some(row) }
}
