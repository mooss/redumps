use sonic_rs::{JsonType, JsonValueTrait, to_object_iter};

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

trait AsString {
    fn as_string(&self) -> Option<String>;
}

impl AsString for sonic_rs::LazyValue<'_> {
    fn as_string(&self) -> Option<String> {
        self.as_str().map(|s| s.to_string())
    }
}

// Media and secure_media can be null or an object; we store a string representation.
fn get_media(value: &sonic_rs::LazyValue) -> Option<String> {
    match value.get_type() {
        JsonType::Null => Some("null".to_string()),
        JsonType::Object => Some(value.to_string()),
        _ => None,
    }
}

pub fn json_entry_to_parquet_row(line: &str) -> Option<ParquetRow> {
    let iter = to_object_iter(line);
    let mut row = ParquetRow::default();

    for (key, value) in iter.filter_map(|res| res.ok()) {
        match &*key {
            "author" => {
                row.author = value.as_string();
            }
            "created_utc" => {
                row.created_utc = match value.get_type() {
                    JsonType::Number => value.as_i64(),
                    JsonType::String => value.as_str().and_then(|s| s.parse::<i64>().ok()),
                    _ => None,
                };
            }
            "crosspost_parent" => {
                row.crosspost_parent = value.as_string();
            }
            "downs" => {
                row.downs = value.as_i64();
            }
            "id" => {
                row.id = value.as_string();
            }
            "is_gallery" => {
                row.is_gallery = value.as_bool();
            }
            "is_self" => {
                row.is_self = value.as_bool();
            }
            "is_video" => {
                row.is_video = value.as_bool();
            }
            "media" => {
                row.media = get_media(&value);
            }
            "num_comments" => {
                row.num_comments = value.as_i64();
            }
            "over_18" => {
                row.over_18 = value.as_bool();
            }
            "permalink" => {
                row.permalink = value.as_string();
            }
            "score" => {
                row.score = value.as_i64();
            }
            "secure_media" => {
                row.secure_media = get_media(&value);
            }
            "selftext" => {
                if let Some(s) = value.as_str() {
                    if s == "[deleted]" {
                        // Skip rows where the selftext is deleted.
                        return None;
                    }
                    row.selftext = Some(s.to_string());
                }
            }
            "subreddit" => {
                row.subreddit = value.as_string();
            }
            "subreddit_id" => {
                row.subreddit_id = value.as_string();
            }
            "subreddit_name_prefixed" => {
                row.subreddit_name_prefixed = value.as_string();
            }
            "title" => {
                row.title = value.as_string();
            }
            "upvote_ratio" => {
                row.upvote_ratio = value.as_f64();
            }
            "ups" => {
                row.ups = value.as_i64();
            }
            "url" => {
                row.url = value.as_string();
            }
            _ => {}
        }
    }

    Some(row)
}
