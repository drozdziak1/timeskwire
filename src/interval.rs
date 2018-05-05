extern crate chrono;
extern crate serde;
extern crate serde_json;

use chrono::{Duration, DateTime, Utc};
use std::collections::HashSet;

#[derive(Debug, Serialize, Deserialize)]
pub struct Interval {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub tags: HashSet<String>,
}

impl Interval {
    pub fn duration(&self) -> Duration {
        self.end.signed_duration_since(self.start)
    }
}
