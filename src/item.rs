use crate::pid::Pid;
use chrono::{DateTime, Duration, Utc};

#[derive(Debug)]
pub struct Item {
    pub id: usize,
    pub name: String,
    pub tags: Vec<String>,

    // scheduling
    pub cadence: Duration,
    pub next: DateTime<Utc>,
    pub pid: Pid,
}
