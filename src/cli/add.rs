use crate::cadence::Cadence;
use crate::date::Date;
use crate::format::Format;
use crate::item::Item;
use crate::tag::Tag;
use anyhow::{Context, Result};
use clap::Parser;
use rusqlite::{params, Connection};

#[derive(Parser, Debug)]
pub struct Command {
    /// Text of the item to add
    #[clap(required(true))]
    text: Vec<String>,

    /// What category does this item belong to?
    #[clap(short, long)]
    tag: Option<String>,

    /// Initial guess on cadence. Don't worry about this being incorrect; we'll
    /// find the right value over time! Supported units: days (d), weeks (w),
    /// 30-day months (m), 365-day years (y)
    #[clap(short, long)]
    cadence: Option<Cadence>,

    /// When should this next be scheduled?
    #[clap(short, long, parse(try_from_str = super::parse_utc_datetime))]
    next: Option<Date>,
}

impl Command {
    pub fn run(&self, conn: &Connection, format: Format) -> Result<()> {
        let today = Date::today();

        let tag_id: Option<u64> = match &self.tag {
            Some(tag_name) => Some(Tag::get_or_create_by_name(conn, tag_name)?.id),
            None => None,
        };

        let id: u64 = conn
            .query_row(
                // This *could* be a RETURNING for all the columns instead of
                // just ID, but making more queries in SQLite is super fast and
                // it lets us use the shared "get an item" infrastructure here,
                // which is better overall.
                "INSERT INTO items (text, cadence, next, tag_id) VALUES (?, ?, ?, ?) RETURNING id",
                params![
                    self.text.join(" "),
                    self.get_cadence(today),
                    self.get_next(today),
                    tag_id,
                ],
                |row| row.get(0),
            )
            .context("could not insert the new row into the database")?;

        let item = Item::get(id, conn)?;

        match format {
            Format::Human => println!(
                "Added \"{}\" with ID {}. Currently scheduled on {} and every {} thereafter",
                item.text, item.id, item.next, item.cadence,
            ),
            Format::Json => println!(
                "{}",
                serde_json::to_string(&item).context("could not convert this item to JSON")?
            ),
        }

        Ok(())
    }

    fn get_cadence(&self, today: Date) -> Cadence {
        match (self.cadence, self.next) {
            (Some(cadence), _) => cadence,
            (None, Some(_)) => (self.get_next(today) - today).into(),
            (None, None) => Cadence::days(1),
        }
    }

    fn get_next(&self, today: Date) -> Date {
        self.next.unwrap_or_else(|| today + self.get_cadence(today))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn default() -> Command {
        Command {
            text: vec!["Text".into()],
            tag: None,
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
        let next = Date::today() + Cadence::weeks(1);

        let mut command = default();
        command.next = Some(next);

        assert_eq!(next, command.get_next(Date::today()));
    }

    #[test]
    fn next_is_calculated_based_on_cadence() {
        let today = Date::today();
        let cadence = Cadence::days(1);

        let mut command = default();
        command.cadence = Some(cadence);
        command.next = None; // just to be explicit

        assert_eq!(today + cadence, command.get_next(today));
    }

    #[test]
    fn cadence_is_used() {
        let cadence = Cadence::weeks(1);

        let mut command = default();
        command.cadence = Some(cadence);

        assert_eq!(cadence, command.get_cadence(Date::today()));
    }

    #[test]
    fn cadence_is_calculated_based_on_next() {
        let today = Date::today();
        let next = today + Cadence::weeks(1);

        let mut command = default();
        command.cadence = None; // just to be explicit
        command.next = Some(next);

        assert_eq!(Cadence::from(next - today), command.get_cadence(today));
    }

    #[test]
    fn cadence_is_one_day_if_neither_is_present() {
        let mut command = default();
        command.cadence = None; // just to be explicit
        command.next = None; // just to be explicit

        assert_eq!(Cadence::days(1), command.get_cadence(Date::today()));
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
        );
    }

    #[test]
    fn adds_specified_tag() {
        let mut command = default();

        let tag: String = "tag".into();
        command.tag = Some(tag.clone());

        let conn = conn();

        command
            .run(&conn, Format::Human)
            .expect("command should not fail");

        let (tag_id, db_tag): (u64, String) = conn
            .query_row("SELECT id, name FROM tags LIMIT 1", [], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })
            .expect("failed to find a new tag");

        assert_eq!(tag, db_tag);

        conn.query_row("SELECT * FROM items WHERE tag_id = ?", [tag_id], |_| Ok(()))
            .expect("expected at least one row with the new tag");
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
        );
    }

    #[test]
    fn adds_specified_next() {
        let mut command = default();

        let next = Date::ymd(2022, 3, 1);
        command.next = Some(next);

        let conn = conn();

        command
            .run(&conn, Format::Human)
            .expect("command should not fail");

        assert_eq!(
            next,
            conn.query_row("SELECT next FROM items LIMIT 1", [], |row| row
                .get::<_, Date>(0))
                .expect("failed to query the database")
        );
    }
}
