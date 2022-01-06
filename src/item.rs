use crate::cadence::Cadence;
use crate::pid::Pid;
use chrono::{DateTime, Utc};

#[derive(Debug, serde::Serialize)]
pub struct Item {
    // TODO: make these private again by reading field indexes by name from the query plan
    pub id: u64,
    pub text: String,

    // scheduling
    pub cadence: Cadence,
    pub next: DateTime<Utc>,

    #[serde(flatten)]
    pub pid: Pid,
}
