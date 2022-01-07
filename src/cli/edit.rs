use crate::cadence::Cadence;
use crate::format::Format;
use crate::item::{Bump, Item};
use crate::tag::Tag;
use anyhow::{Context, Result};
use chrono::{DateTime, Local, Utc};
use clap::Parser;
use rusqlite::{params, Connection};

#[derive(Debug, Parser)]
pub struct Command {
    /// ID of the item to edit
    id: u64,

    /// New text to for the item. New text is required if there are no
    /// other edits in the flags.
    #[clap(required_unless_present_any(&["tag", "next", "cadence", "bump"]))]
    text: Vec<String>,

    /// Change this item's tag
    #[clap(long, short)]
    tag: Option<String>,

    /// Change when this item will be scheduled next
    #[clap(long, short, conflicts_with_all(&["cadence", "bump"]), parse(try_from_str = super::parse_utc_datetime))]
    next: Option<DateTime<Utc>>,

    /// Set the cadence manually (see add --help for docs on this.)
    #[clap(long, short, conflicts_with_all(&["next", "bump"]))]
    cadence: Option<Cadence>,

    /// Tweak this item's schedule a little earlier or later
    #[clap(long, short, arg_enum, conflicts_with_all(&["cadence", "next"]))]
    bump: Option<Bump>,
}

impl Command {
    pub fn run(&self, conn: &Connection, format: Format) -> Result<()> {
        if !self.text.is_empty() {
            self.update_text(conn)?;
            if format == Format::Human {
                println!("Updated text to {}", self.text.join(" "));
            }
        }

        if let Some(new_tag) = &self.tag {
            self.update_tag(new_tag, conn)?;
            if format == Format::Human {
                println!("Updated tag to {}", new_tag);
            }
        }

        if let Some(new_next) = &self.next {
            self.update_next(new_next, conn)?;
            if format == Format::Human {
                println!(
                    "Updated next to {}",
                    new_next.with_timezone(&Local).to_rfc2822()
                );
            }
        }

        if let Some(new_cadence) = &self.cadence {
            self.update_cadence(*new_cadence, conn)?;
            if format == Format::Human {
                println!("Updated cadence to {}", new_cadence);
            }
        }

        if let Some(bump) = &self.bump {
            let (adjustment, item) = self.update_bump(bump, conn)?;
            if format == Format::Human {
                println!(
                    "Bumped schedule by {} to {}",
                    adjustment,
                    item.next.with_timezone(&Local).to_rfc2822()
                );
            }
        }

        if format == Format::Json {
            let item = Item::get(self.id, conn).context("couldn't get item for JSON formatting")?;
            println!(
                "{}",
                serde_json::to_string(&item).context("couldn't convert item to JSON")?
            );
        }

        Ok(())
    }

    fn update_text(&self, conn: &Connection) -> Result<()> {
        self.handle_update(
            conn.execute(
                "UPDATE items SET text = ? WHERE id = ?",
                params![self.text.join(" "), self.id],
            ),
            "there was a problem updating item text",
        )
    }

    fn update_tag(&self, new_tag: &str, conn: &Connection) -> Result<()> {
        let tag = Tag::get_or_create(conn, new_tag).context("couldn't get the new tag")?;

        self.handle_update(
            conn.execute(
                "UPDATE items SET tag_id = ? WHERE id = ?",
                params![tag.id, self.id],
            ),
            "couldn't update the tag",
        )
    }

    fn update_next(&self, new_next: &DateTime<Utc>, conn: &Connection) -> Result<()> {
        self.handle_update(
            conn.execute(
                "UPDATE items SET next = ? WHERE id = ?",
                params![new_next, self.id],
            ),
            "couldn't update next",
        )
    }

    fn update_cadence(&self, new_cadence: Cadence, conn: &Connection) -> Result<()> {
        self.handle_update(
            conn.execute(
                "UPDATE items SET cadence = ? WHERE id = ?",
                params![new_cadence, self.id],
            ),
            "couldn't update cadence",
        )
    }

    fn update_bump(&self, bump: &Bump, conn: &Connection) -> Result<(Cadence, Item)> {
        let mut item = Item::get(self.id, conn).context("couldn't load item to bump")?;
        let adjustment = item.bump_cadence(bump);
        item.next = item.next + adjustment;

        item.save(conn)
            .context("could not save item after bumping")?;

        Ok((adjustment, item))
    }

    fn handle_update(&self, count: rusqlite::Result<usize>, context: &'static str) -> Result<()> {
        match count {
            Ok(0) => anyhow::bail!("could not update item with ID {} (does it exist?)", self.id),
            Ok(1) => Ok(()),
            Ok(_) => anyhow::bail!(
                "there were somehow multiple rows with the same ID. Please report this as a bug!"
            ),
            Err(problem) => Err(problem).context(context),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::TimeZone;

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
                1,
            ],
        )
        .unwrap();

        conn
    }

    #[test]
    fn fails_for_invalid_id() {
        let conn = setup();
        let command = Command::try_parse_from(&["edit", "0", "new text"]).unwrap();

        assert!(command.run(&conn, Format::Human).is_err());
    }

    #[test]
    fn updates_text() {
        let conn = setup();
        let command = Command::try_parse_from(&["edit", "1", "new", "text"]).unwrap();
        command.run(&conn, Format::Human).unwrap();

        assert_eq!(
            "new text".to_string(),
            conn.query_row("SELECT text FROM items WHERE id = 1", [], |row| row
                .get::<_, String>(0))
                .unwrap()
        );
    }

    #[test]
    fn updates_tag() {
        let conn = setup();
        let command = Command::try_parse_from(&["edit", "1", "--tag", "newtag"]).unwrap();
        command.run(&conn, Format::Human).unwrap();

        assert_eq!(
            "newtag".to_string(),
            conn.query_row("SELECT name FROM tags WHERE id = 2", [], |row| row
                .get::<_, String>(0))
                .unwrap()
        );

        assert_eq!(
            2,
            conn.query_row("SELECT tag_id FROM items WHERE id = 1", [], |row| row
                .get::<_, u64>(0))
                .unwrap()
        );
    }

    #[test]
    fn updates_next() {
        let conn = setup();
        let command = Command::try_parse_from(&["edit", "1", "--next", "2022-03-01"]).unwrap();
        command.run(&conn, Format::Human).unwrap();

        assert_eq!(
            Utc.ymd(2022, 3, 1).and_hms(0, 0, 0),
            conn.query_row("SELECT next FROM items WHERE id = 1", [], |row| row
                .get::<_, DateTime<Utc>>(0))
                .unwrap()
        );
    }

    #[test]
    fn updates_cadence() {
        let conn = setup();
        let command = Command::try_parse_from(&["edit", "1", "--cadence", "1w"]).unwrap();
        command.run(&conn, Format::Human).unwrap();

        assert_eq!(
            Cadence::weeks(1),
            conn.query_row("SELECT cadence FROM items WHERE id = 1", [], |row| row
                .get::<_, Cadence>(
                0
            ))
            .unwrap()
        );
    }

    #[test]
    fn bumps_schedule() {
        let conn = setup();
        let before = Item::get(1, &conn).unwrap();

        let command = Command::try_parse_from(&["edit", "1", "--bump", "later"]).unwrap();
        command.run(&conn, Format::Human).unwrap();

        let after = Item::get(1, &conn).unwrap();

        println!(
            "before.cadence: {}, after.cadence: {}",
            before.cadence, after.cadence
        );
        assert!(before.cadence < after.cadence);

        println!("before.next: {}, after.next: {}", before.next, after.next);
        assert!(before.next < after.next);
    }
}
