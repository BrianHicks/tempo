use crate::cadence::Cadence;
use crate::format::Format;
use crate::item::Item;
use crate::pid::Pid;
use anyhow::{Context, Result};
use chrono::{DateTime, TimeZone, Utc};
use clap::Parser;
use rusqlite::{params, Connection};

#[derive(Parser, Debug)]
pub struct AddCommand {
    /// Text of the item to add
    #[clap(required(true))]
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

        let item = conn
            .query_row(
                "INSERT INTO items (text, cadence, next) VALUES (?, ?, ?) RETURNING id, text, cadence, next, proportional_factor, integral, integral_factor, last_error, derivative_factor",
                params![
                    self.text.join(" "),
                    self.get_cadence(now),
                    self.get_next(now)
                ],
                |row| Ok(Item{
                    id: row.get(0)?,
                    text: row.get(1)?,
                    cadence: row.get(2)?,
                    next: row.get(3)?,
                    pid: Pid {
                        proportional_factor: row.get(4)?,
                        integral: row.get(5)?,
                        integral_factor: row.get(6)?,
                        last_error: row.get(7)?,
                        derivative_factor: row.get(8)?,
                    }
                }),
            )
            .context("could not insert the new row into the database")?;

        match format {
            Format::Human => println!("Added \"{}\" with ID {}", item.text, item.id),
            Format::Json => println!(
                "{}",
                serde_json::to_string(&item).context("could not convert this item to JSON")?
            ),
        }

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
            text: vec!["Text".into()],
            tags: Vec::default(),
            cadence: None,
            next: None,
        }
    }

    fn conn() -> Connection {
        let mut conn = Connection::open_in_memory().expect("couldn't open an in-memory database");
        crate::db::migrations::runner()
            .run(&mut conn)
            .expect("couldn't migrate database");

        conn
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

    #[test]
    fn adds_specified_text() {
        let mut command = default();
        command.text = vec!["Hello,".into(), "world!".into()];

        let conn = conn();

        command
            .run(&conn, Format::Human)
            .expect("command should not fail");

        assert_eq!(
            "Hello, world!".to_string(),
            conn.query_row("SELECT text FROM items LIMIT 1", [], |row| row
                .get::<_, String>(0))
                .expect("failed to query the database")
        )
    }

    #[test]
    fn adds_specified_cadence() {
        let mut command = default();

        let cadence = Cadence::weeks(1);
        command.cadence = Some(cadence);

        let conn = conn();

        command
            .run(&conn, Format::Human)
            .expect("command should not fail");

        assert_eq!(
            cadence,
            conn.query_row("SELECT cadence FROM items LIMIT 1", [], |row| row.get(0))
                .expect("failed to query the database")
        )
    }

    #[test]
    fn adds_specified_next() {
        let mut command = default();

        let next = Utc.ymd(2022, 03, 01).and_hms(9, 0, 0);
        command.next = Some(next);

        let conn = conn();

        command
            .run(&conn, Format::Human)
            .expect("command should not fail");

        assert_eq!(
            next,
            conn.query_row("SELECT next FROM items LIMIT 1", [], |row| row
                .get::<_, DateTime<Utc>>(0))
                .expect("failed to query the database")
        )
    }
}
