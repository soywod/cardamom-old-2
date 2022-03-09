use chrono::{DateTime, Local};

pub type Etag = Option<String>;

#[derive(Debug, PartialEq, Eq)]
pub struct Card {
    pub id: String,
    pub etag: Etag,
    pub date: DateTime<Local>,
    pub raw: String,
}
