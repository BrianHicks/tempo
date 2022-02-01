use crate::date::Date;
use crate::format::Format;
use crate::item::Item;
use crate::tag::Tag;
use anyhow::{Context, Result};
use clap::Parser;
use rusqlite::Connection;
use std::collections::HashSet;

#[derive(Debug, Parser)]
pub struct Command {
    /// Only get up to this many items
    #[clap(long, short)]
    limit: Option<usize>,

    /// Only get items with these tags
    #[clap(long, short)]
    tag: Option<Vec<String>>,
}

impl Command {
    pub fn run(&self, conn: &Connection, format: Format) -> Result<()> {
        let pulled = self.items(conn)?;

        match format {
            Format::Human => {
                for item in pulled {
                    println!("{}: {} (due {})", item.id, item.text, item.next);
                }
            }
            Format::Json => println!(
                "{}",
                serde_json::to_string(&pulled).context("could not dump pulled items to JSON")?
            ),
        };

        Ok(())
    }

    fn items(&self, conn: &Connection) -> Result<Vec<Item>> {
        let tag_ids: Option<HashSet<u64>> = match &self.tag {
            Some(tag_names) => Some(
                Tag::all(conn)
                    .context("couldn't get tags")?
                    .filter(|tag| tag_names.contains(&tag.name))
                    .map(|tag| tag.id)
                    .collect(),
            ),

            None => None,
        };

        // note to future explorers: seems like we could do this with a SELECT,
        // right? Well, how many items are we ever gonna have? It's probably
        // under 1,000, so we're not going to see a huge speed benefit (computers
        // are fast!) and we'd probably have to introduce some query builder
        // dependency as well. Let's see how far we can take the naive pattern!
        let items = Item::due(conn)
            .context("couldn't get items from the database")?
            .filter(|item| match &tag_ids {
                Some(ids) => item.tag_id.as_ref().map_or(false, |id| ids.contains(id)),
                None => true,
            })
            .take(self.limit.unwrap_or(usize::MAX))
            .collect();

        Ok(items)
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
            params!["X", Date::today() + Cadence::days(1), Cadence::days(1)],
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
        let next = Date::today() - cadence;
        conn.execute(
            "INSERT INTO items (text, next, cadence) VALUES (?, ?, ?)",
            params!["X", next, cadence],
        )
        .unwrap();

        let command = Command::try_parse_from(&["pull"]).unwrap();
        let items = command.items(&conn).unwrap();

        assert_eq!(vec![Item::get(1, &conn).unwrap()], items);
    }

    #[test]
    fn not_matching_tag() {
        let conn = conn();

        let cadence = Cadence::days(1);
        let next = Date::today() - cadence;
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
    fn tagged_item_with_no_tag_specified() {
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
        let next = Date::today() - cadence;
        conn.execute(
            "INSERT INTO items (text, next, cadence, tag_id) VALUES (?, ?, ?, ?)",
            params!["X", next, cadence, tag_id],
        )
        .unwrap();

        let command = Command::try_parse_from(&["pull"]).unwrap();
        let items = command.items(&conn).unwrap();

        assert_eq!(vec![Item::get(1, &conn).unwrap()], items);
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
        let next = Date::today() - cadence;
        conn.execute(
            "INSERT INTO items (text, next, cadence, tag_id) VALUES (?, ?, ?, ?)",
            params!["X", next, cadence, tag_id],
        )
        .unwrap();

        let command = Command::try_parse_from(&["pull", "--tag", tag_name]).unwrap();
        let items = command.items(&conn).unwrap();

        assert_eq!(vec![Item::get(1, &conn).unwrap()], items);
    }

    #[test]
    fn limit() {
        let conn = conn();

        let later = Cadence::days(1);
        let earlier = Cadence::days(2);
        let today = Date::today();

        let due_later = today - later;
        conn.execute(
            "INSERT INTO items (text, next, cadence) VALUES (?, ?, ?)",
            params!["Due Later", due_later, later],
        )
        .unwrap();

        let due_earlier = today - earlier;
        conn.execute(
            "INSERT INTO items (text, next, cadence) VALUES (?, ?, ?)",
            params!["Due Earlier", due_earlier, earlier],
        )
        .unwrap();

        let command = Command::try_parse_from(&["pull", "--limit", "1"]).unwrap();
        let items = command.items(&conn).unwrap();

        assert_eq!(vec![Item::get(2, &conn).unwrap()], items);
    }

    #[test]
    fn sorted_by_next_ascending() {
        let conn = conn();

        let later = Cadence::days(1);
        let earlier = Cadence::days(2);
        let today = Date::today();

        let due_earlier = today - earlier;
        conn.execute(
            "INSERT INTO items (text, next, cadence) VALUES (?, ?, ?)",
            params!["Due Earlier", due_earlier, earlier],
        )
        .unwrap();

        let due_later = today - later;
        conn.execute(
            "INSERT INTO items (text, next, cadence) VALUES (?, ?, ?)",
            params!["Due Later", due_later, later],
        )
        .unwrap();

        let command = Command::try_parse_from(&["pull"]).unwrap();
        let items = command.items(&conn).unwrap();

        assert_eq!(
            vec![Item::get(1, &conn).unwrap(), Item::get(2, &conn).unwrap()],
            items
        );
    }
}
