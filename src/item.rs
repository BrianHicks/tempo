use crate::pid::Pid;
use chrono::Duration;

#[derive(Debug)]
pub struct Item {
    pub id: usize,
    pub name: String,
    pub tags: Vec<String>,

    // scheduling
    pub cadence: Duration,
    pub pid: Pid,
}
