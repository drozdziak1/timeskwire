extern crate chrono;
extern crate serde;
extern crate serde_json;

use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct Interval {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub tags: Vec<String>,
}
