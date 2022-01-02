use chrono::Duration;

pub struct Item {
    pub id: usize,
    pub name: String,
    pub tags: Vec<String>,
    pub cadence: Duration,
}
