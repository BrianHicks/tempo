use super::duration::parse_duration;
use crate::format::Format;
use anyhow::{Context, Result};
use chrono::{DateTime, Duration, TimeZone, Utc};
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
    #[clap(short, long, parse(try_from_str = parse_duration))]
    cadence: Option<Duration>,

    /// When should this next be scheduled?
    #[clap(short, long, parse(try_from_str = parse_utc_datetime))]
    next: Option<DateTime<Utc>>,
}

fn parse_utc_datetime(input: &str) -> Result<DateTime<Utc>> {
    Utc.datetime_from_str(input, "%Y-%m-%dT%H:%M:%S")
        .or_else(|_| Utc.datetime_from_str(&format!("{}T00:00:00", input), "%Y-%m-%dT%H:%M:%S"))
        .context("couldn't parse a date")
}

impl AddCommand {
    pub fn run(&self, _format: Format) -> Result<()> {
        println!("tmp: {}", self.get_cadence(Utc::now()));
        todo!("reimplement AddCommand.run")
    }

    fn get_cadence(&self, now: DateTime<Utc>) -> Duration {
        match (self.cadence, self.next) {
            (Some(cadence), _) => cadence,
            (None, Some(_)) => self.get_next(now) - now,
            (None, None) => Duration::days(1),
        }
    }

    fn get_next(&self, now: DateTime<Utc>) -> DateTime<Utc> {
        self.next.unwrap_or_else(|| now + self.get_cadence(now))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn default() -> AddCommand {
        AddCommand {
            name: Vec::default(),
            tags: Vec::default(),
            cadence: None,
            next: None,
        }
    }

    #[test]
    fn next_is_used() {
        let next = Utc::now() + Duration::weeks(1);

        let mut command = default();
        command.next = Some(next);

        assert_eq!(next, command.get_next(Utc::now()))
    }

    #[test]
    fn next_is_calculated_based_on_cadence() {
        let now = Utc::now();
        let duration = Duration::days(1);

        let mut command = default();
        command.cadence = Some(duration);
        command.next = None; // just to be explicit

        assert_eq!(now + duration, command.get_next(now));
    }

    #[test]
    fn cadence_is_used() {
        let cadence = Duration::weeks(1);

        let mut command = default();
        command.cadence = Some(cadence);

        assert_eq!(cadence, command.get_cadence(Utc::now()))
    }

    #[test]
    fn cadence_is_calculated_based_on_next() {
        let now = Utc::now();
        let next = now + Duration::weeks(1);

        let mut command = default();
        command.cadence = None; // just to be explicit
        command.next = Some(next);

        assert_eq!(next - now, command.get_cadence(now));
    }

    #[test]
    fn cadence_is_one_day_if_neither_is_present() {
        let mut command = default();
        command.cadence = None; // just to be explicit
        command.next = None; // just to be explicit

        assert_eq!(Duration::days(1), command.get_cadence(Utc::now()))
    }
}
