use crate::format::Format;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use clap::Parser;
use rusqlite::Connection;
use serde::Serialize;
use tabled::{Table, Tabled};

#[derive(Debug, Parser)]
pub struct Command {
    /// Only get up to this many items
    #[clap(long, short)]
    limit: Option<usize>,

    /// Only get items with these tags
    #[clap(long, short)]
    tag: Vec<String>,
}

#[derive(Debug, Serialize, Tabled, PartialEq)]
struct Pulled {
    #[header("ID")]
    id: u64,

    #[header("Text")]
    text: String,

    #[header("Scheduled")]
    next: DateTime<Utc>,

    #[header("Tag")]
    #[field(display_with = "display_tag")]
    tag: Option<String>,
}

fn display_tag(tag_opt: &Option<String>) -> String {
    match tag_opt {
        Some(tag) => tag.to_string(),
        None => "-".to_string(),
    }
}

impl Command {
    pub fn run(&self, conn: &Connection, format: Format) -> Result<()> {
        let pulled = self.items(conn)?;

        match format {
            Format::Human => println!(
                "{}",
                Table::new(pulled).with(tabled::Style::PSQL).to_string()
            ),
            Format::Json => println!(
                "{}",
                serde_json::to_string(&pulled).context("could not dump pulled items to JSON")?
            ),
        };

        Ok(())
    }

    fn items(&self, conn: &Connection) -> Result<Vec<Pulled>> {
        // note to future explorers: the sqlite API doesn't let you use an array
        // in a parameter, so we can't do like `WHERE tags.name IN ?`. So,
        // we just pull everything and do the filtering locally. We're not
        // dealing with enough data here for it to be a problem, though!
        let mut statement = conn
            .prepare(
                "SELECT items.id, text, next, tags.name FROM items LEFT JOIN tags ON items.tag_id = tags.id WHERE next <= ? ORDER BY next ASC",
            )
            .context("could not prepare query to pull items")?;

        let unlimited = statement
            .query_map([Utc::now()], |row| {
                Ok(Pulled {
                    id: row.get(0)?,
                    text: row.get(1)?,
                    next: row.get(2)?,
                    tag: row.get(3)?,
                })
            })
            .context("could not pull next items")?
            .flatten()
            .filter(|item| match &item.tag {
                Some(filter_tag) => self.tag.contains(filter_tag),
                None => self.tag.is_empty(),
            });

        Ok(match self.limit {
            Some(limit) => unlimited.take(limit).collect(),
            None => unlimited.collect(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cadence::Cadence;
    use rusqlite::params;

    fn conn() -> Connection {
        let mut conn = Connection::open_in_memory().expect("couldn't open an in-memory database");
        crate::db::migrations::runner()
            .run(&mut conn)
            .expect("couldn't migrate database");

        conn
    }

    #[test]
    fn not_due() {
        let conn = conn();

        conn.execute(
            "INSERT INTO items (text, next, cadence) VALUES (?, ?, ?)",
            params!["X", Utc::now() + Cadence::days(1), Cadence::days(1)],
        )
        .unwrap();

        let command = Command::try_parse_from(&["pull"]).unwrap();
        let items = command.items(&conn).unwrap();

        assert!(items.is_empty());
    }

    #[test]
    fn due() {
        let conn = conn();

        let cadence = Cadence::days(1);
        let next = Utc::now() - cadence;
        conn.execute(
            "INSERT INTO items (text, next, cadence) VALUES (?, ?, ?)",
            params!["X", next, cadence],
        )
        .unwrap();

        let command = Command::try_parse_from(&["pull"]).unwrap();
        let items = command.items(&conn).unwrap();

        assert_eq!(
            vec![Pulled {
                id: 1,
                text: "X".into(),
                next,
                tag: None
            }],
            items
        );
    }

    #[test]
    fn not_matching_tag() {
        let conn = conn();

        let cadence = Cadence::days(1);
        let next = Utc::now() - cadence;
        conn.execute(
            "INSERT INTO items (text, next, cadence) VALUES (?, ?, ?)",
            params!["X", next, cadence],
        )
        .unwrap();

        let command = Command::try_parse_from(&["pull", "--tag", "x"]).unwrap();
        let items = command.items(&conn).unwrap();

        assert!(items.is_empty());
    }

    #[test]
    fn matching_tag() {
        let conn = conn();

        let tag_name = "tag";
        let tag_id: u64 = conn
            .query_row(
                "INSERT INTO tags (name) VALUES (?) RETURNING id",
                [tag_name],
                |row| row.get(0),
            )
            .unwrap();

        let cadence = Cadence::days(1);
        let next = Utc::now() - cadence;
        conn.execute(
            "INSERT INTO items (text, next, cadence, tag_id) VALUES (?, ?, ?, ?)",
            params!["X", next, cadence, tag_id],
        )
        .unwrap();

        let command = Command::try_parse_from(&["pull", "--tag", tag_name]).unwrap();
        let items = command.items(&conn).unwrap();

        assert_eq!(
            vec![Pulled {
                id: 1,
                text: "X".into(),
                next,
                tag: Some(tag_name.into()),
            }],
            items
        );
    }

    #[test]
    fn limit() {
        let conn = conn();

        let later = Cadence::days(1);
        let earlier = Cadence::days(2);
        let now = Utc::now();

        let due_earlier = now - earlier;
        conn.execute(
            "INSERT INTO items (text, next, cadence) VALUES (?, ?, ?)",
            params!["Due Earlier", due_earlier, earlier],
        )
        .unwrap();

        let due_later = now - later;
        conn.execute(
            "INSERT INTO items (text, next, cadence) VALUES (?, ?, ?)",
            params!["Due Later", due_later, later],
        )
        .unwrap();

        let command = Command::try_parse_from(&["pull", "--limit", "1"]).unwrap();
        let items = command.items(&conn).unwrap();

        assert_eq!(
            vec![Pulled {
                id: 1,
                text: "Due Earlier".into(),
                next: due_earlier,
                tag: None,
            }],
            items
        );
    }
}
