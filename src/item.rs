use crate::pid::Pid;
use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use thiserror::Error;

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

impl Item {
    pub fn finish(&mut self) -> Result<(), FinishError> {
        self.finish_from(Utc::now())
    }

    fn finish_from(&mut self, base: DateTime<Utc>) -> Result<(), FinishError> {
        if base < self.next {
            log::debug!("item not due yet. base: {}, next: {}", &base, self.next);
            return Err(FinishError::NotDueYet(self.next));
        }

        self.next = base + self.cadence;

        Ok(())
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum FinishError {
    #[error("this item is not due yet, and won't be until {}", .0.with_timezone(&chrono::Local).to_rfc2822())]
    NotDueYet(DateTime<Utc>),
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn default() -> Item {
        Item {
            name: String::default(),
            tags: Vec::default(),
            cadence: Duration::days(1),
            next: Utc.ymd(2022, 01, 01).and_hms(0, 0, 0),
            pid: Pid::default(),
        }
    }
    #[test]
    fn finish_moves_by_cadence() {
        let now = Utc::now();

        let mut item = default();
        item.finish_from(now);

        assert_eq!(now + item.cadence, item.next);
    }

    #[test]
    fn finish_errors_for_non_due_task() {
        let now = Utc::now();

        let mut item = default();
        item.next = now + Duration::days(1);

        assert_eq!(
            Err(FinishError::NotDueYet(item.next)),
            item.finish_from(now)
        )
    }
}
