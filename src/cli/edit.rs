use crate::cadence::Cadence;
use crate::format::Format;
use anyhow::{Context, Result};
use chrono::{DateTime, TimeZone, Utc};
use clap::Parser;
use rusqlite::{params, Connection};

#[derive(Debug, Parser)]
pub struct EditCommand {
    /// ID of the item to edit
    id: usize,

    /// New text to for the item. New text is required if there are no
    /// other edits in the flags.
    #[clap(required_unless_present_any(&["tag", "next", "cadence", "earlier", "much-earlier", "later", "much-later"]))]
    text: Vec<String>,

    /// Change this item's tag
    #[clap(short, long)]
    tag: Option<String>,

    /// Change when this item will be scheduled next
    #[clap(long, conflicts_with_all(&["cadence", "earlier", "much-earlier", "later", "much-later"]), parse(try_from_str = super::parse_utc_datetime))]
    next: Option<DateTime<Utc>>,

    /// Set the cadence manually (see add --help for docs on this.)
    #[clap(long, conflicts_with_all(&["next", "earlier", "much-earlier", "later", "much-later"]))]
    cadence: Option<Cadence>,

    /// Tweak this item's next schedule to be a little earlier
    #[clap(long, short('e'), conflicts_with_all(&["cadence", "next", "much-earlier", "later", "much-later"]))]
    earlier: bool,

    /// Tweak this item's next schedule to be much earlier
    #[clap(long, short('E'), conflicts_with_all(&["cadence", "next", "earlier", "later", "much-later"]))]
    much_earlier: bool,

    /// Tweak this item's next schedule to be a little later
    #[clap(long, short('l'), conflicts_with_all(&["cadence", "next", "earlier", "much-earlier", "much-later"]))]
    later: bool,

    /// Tweak this item's next schedule to be much later
    #[clap(long, short('L'), conflicts_with_all(&["cadence", "next", "earlier", "much-earlier", "later"]))]
    much_later: bool,
}

impl EditCommand {
    pub fn run(&self, conn: &Connection, format: Format) -> Result<()> {
        if !self.text.is_empty() {
            self.update_text(conn)?;
            if format == Format::Human {
                println!("Updated text to {}", self.text.join(" "))
            }
        }

        Ok(())
    }

    fn update_text(&self, conn: &Connection) -> Result<()> {
        match conn.execute(
            "UPDATE items SET text = ? WHERE id = ?",
            params![self.text.join(" "), self.id],
        ) {
            Ok(0) => anyhow::bail!("could not update text for ID {} (does it exist?)", self.id),
            Ok(1) => Ok(()),
            Ok(_) => anyhow::bail!(
                "there were somehow multiple rows with the same ID. Please report this as a bug!"
            ),
            Err(problem) => Err(problem).context("there was a problem updating item text"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn setup() -> Connection {
        let mut conn = Connection::open_in_memory().unwrap();
        crate::db::migrations::runner().run(&mut conn).unwrap();

        conn.execute("INSERT INTO tags (id, name) VALUES (1, \"test\")", [])
            .unwrap();
        conn.execute(
            "INSERT INTO items (text, cadence, next, tag_id) VALUES (?, ?, ?, ?)",
            params![
                "test",
                Cadence::days(1),
                Utc.ymd(2022, 1, 1).and_hms(0, 0, 0),
                1
            ],
        )
        .unwrap();

        conn
    }

    #[test]
    fn fails_for_invalid_id() {
        let conn = setup();
        let command = EditCommand::try_parse_from(&["edit", "0"]).unwrap();

        assert!(command.run(&conn, Format::Human).is_err());
    }

    #[test]
    fn updates_text() {
        let conn = setup();
        let command = EditCommand::try_parse_from(&["edit", "1", "new", "text"]).unwrap();
        command.run(&conn, Format::Human).unwrap();

        assert_eq!(
            "new text".to_string(),
            conn.query_row("SELECT text FROM items WHERE id = 1", [], |row| row
                .get::<_, String>(0))
                .unwrap()
        )
    }
}
