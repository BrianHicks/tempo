use crate::pid::Pid;
use chrono::{DateTime, Duration, Utc};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Item {
    pub name: String,
    pub tags: Vec<String>,

    // scheduling
    #[serde(with = "crate::serde_duration")]
    pub cadence: Duration,
    pub next: DateTime<Utc>,
    #[serde(flatten)]
    pub pid: Pid,
}
