use super::duration::parse_duration;
use anyhow::Result;
use chrono::Duration;
use chrono::{DateTime, Utc};
use clap::Parser;

#[derive(Parser, Debug)]
pub struct AddCommand {
    /// Text of the item to add
    name: Vec<String>,

    /// What category does this item belong to?
    #[clap(short, long)]
    tags: Vec<String>,

    /// Initial guess on cadence. Don't worry about this being incorrect; we'll
    /// find the right value over time! Supported units: hours (h), days (d),
    /// weeks (w), 30-day months (m), 365-day years (y)
    #[clap(short, long, default_value = "1d", parse(try_from_str = parse_duration))]
    cadence: Duration,

    /// When should this next be scheduled?
    #[clap(short, long)] // TODO: should parse a chrono date
    next: Option<DateTime<Utc>>,
}

impl AddCommand {
    pub fn run(&self, mut store: crate::store::Store) -> Result<()> {
        let id = store.add(
            self.name.join(" "),
            &self.tags,
            self.cadence,
            self.get_next(Utc::now()),
        );
        println!("{:#?}", store.get(id));

        Ok(())
    }

    fn get_next(&self, now: DateTime<Utc>) -> DateTime<Utc> {
        self.next.unwrap_or_else(|| now + self.cadence)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn uses_next_if_present() {
        let next = Utc::now() + Duration::weeks(1);

        let command = AddCommand {
            name: vec![],
            tags: vec![],
            cadence: Duration::days(1),
            next: Some(next),
        };

        assert_eq!(next, command.get_next(Utc::now()))
    }

    #[test]
    fn calculates_next_based_on_cadence() {
        let now = Utc::now();
        let duration = Duration::days(1);

        let command = AddCommand {
            name: vec![],
            tags: vec![],
            cadence: duration,
            next: None,
        };

        assert_eq!(now + duration, command.get_next(now));
    }
}
