use crate::cadence::Cadence;
use crate::format::Format;
use anyhow::{Context, Result};
use chrono::{DateTime, TimeZone, Utc};
use clap::Parser;
use rusqlite::{params, Connection};

#[derive(Parser, Debug)]
pub struct AddCommand {
    /// Text of the item to add
    text: Vec<String>,

    /// What category does this item belong to?
    #[clap(short, long)]
    tags: Vec<String>,

    /// Initial guess on cadence. Don't worry about this being incorrect; we'll
    /// find the right value over time! Supported units: hours (h), days (d),
    /// weeks (w), 30-day months (m), 365-day years (y)
    #[clap(short, long)]
    cadence: Option<Cadence>,

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
    pub fn run(&self, conn: &Connection, format: Format) -> Result<()> {
        let now = Utc::now();
        let (id, text, cadence, next): (usize, String, Cadence, DateTime<Utc>) = conn
            .query_row(
                "INSERT INTO items (text, cadence, next) VALUES (?, ?, ?) RETURNING id, text, cadence, next",
                params![
                    self.text.join(" "),
                    self.get_cadence(now),
                    self.get_next(now)
                ],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .context("could not insert the new row into the database")?;

        println!("{} {} {:?} {}", id, text, cadence, next);

        Ok(())
    }

    fn get_cadence(&self, now: DateTime<Utc>) -> Cadence {
        match (self.cadence, self.next) {
            (Some(cadence), _) => cadence,
            (None, Some(_)) => (self.get_next(now) - now).into(),
            (None, None) => Cadence::days(1),
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
            text: Vec::default(),
            tags: Vec::default(),
            cadence: None,
            next: None,
        }
    }

    #[test]
    fn next_is_used() {
        let next = Utc::now() + Cadence::weeks(1);

        let mut command = default();
        command.next = Some(next);

        assert_eq!(next, command.get_next(Utc::now()))
    }

    #[test]
    fn next_is_calculated_based_on_cadence() {
        let now = Utc::now();
        let cadence = Cadence::days(1);

        let mut command = default();
        command.cadence = Some(cadence);
        command.next = None; // just to be explicit

        assert_eq!(now + cadence, command.get_next(now));
    }

    #[test]
    fn cadence_is_used() {
        let cadence = Cadence::weeks(1);

        let mut command = default();
        command.cadence = Some(cadence);

        assert_eq!(cadence, command.get_cadence(Utc::now()))
    }

    #[test]
    fn cadence_is_calculated_based_on_next() {
        let now = Utc::now();
        let next = now + Cadence::weeks(1);

        let mut command = default();
        command.cadence = None; // just to be explicit
        command.next = Some(next);

        assert_eq!(Cadence::from(next - now), command.get_cadence(now));
    }

    #[test]
    fn cadence_is_one_day_if_neither_is_present() {
        let mut command = default();
        command.cadence = None; // just to be explicit
        command.next = None; // just to be explicit

        assert_eq!(Cadence::days(1), command.get_cadence(Utc::now()))
    }
}
