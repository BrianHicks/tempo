use crate::format::Format;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use clap::Parser;
use rusqlite::Connection;

#[derive(Debug, Parser)]
pub struct Command {
    /// Only get up to this many items
    #[clap(long, short)]
    limit: Option<usize>,

    /// Only get items with these tags
    #[clap(long, short)]
    tag: Vec<String>,
}

#[derive(Debug)]
struct Pulled {
    id: u64,
    text: String,
    next: DateTime<Utc>,
    tag: Option<String>,
}

impl Command {
    pub fn run(&self, conn: &Connection, _format: Format) -> Result<()> {
        // note to future explorers: the sqlite API doesn't let you use an array
        // in a parameter, so we can't do like `WHERE tags.name IN ?`. So,
        // we just pull everything and do the filtering locally. We're not
        // dealing with enough data here for it to be a problem, though!
        let mut statement = conn
            .prepare(
                "SELECT items.id, text, next, tags.name FROM items LEFT JOIN tags ON items.tag_id = tags.id ORDER BY next ASC",
            )
            .context("could not prepare query to pull items")?;

        let unlimited = statement
            .query_map([], |row| {
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

        let pulled: Vec<Pulled> = match self.limit {
            Some(limit) => unlimited.take(limit).collect(),
            None => unlimited.collect(),
        };

        println!("{:#?}", pulled);

        Ok(())
    }
}
