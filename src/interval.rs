extern crate serde;
extern crate serde_json;

use chrono::{Duration, DateTime, Local};

use std::collections::BTreeSet;

#[derive(Debug, Serialize, Deserialize)]
pub struct Interval {
    pub start: DateTime<Local>,
    pub end: DateTime<Local>,
    pub tags: BTreeSet<String>,
}

impl Interval {
    pub fn to_duration(&self) -> Duration {
        self.end.signed_duration_since(self.start)
    }
}
