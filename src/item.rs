use crate::cadence::Cadence;
use crate::pid::Pid;
use chrono::{DateTime, Utc};

#[derive(Debug, serde::Serialize)]
pub struct Item {
    pub id: u64,
    pub text: String,

    // scheduling
    pub cadence: Cadence,
    pub next: DateTime<Utc>,

    #[serde(flatten)]
    pub pid: Pid,
}
