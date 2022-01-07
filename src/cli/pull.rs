use crate::format::Format;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use clap::Parser;
use rusqlite::Connection;

#[derive(Debug, Parser)]
pub struct Command {
    /// Only get up to this many items
    #[clap(long, short)]
    limit: Option<u32>,

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
        let mut statement = conn
            .prepare(
                "SELECT items.id, text, next, tags.name FROM items LEFT JOIN tags ON items.tag_id = tags.id ORDER BY next ASC",
            )
            .context("could not prepare query to pull items")?;

        let pulled: Vec<Pulled> = statement
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
            .collect();

        println!("{:#?}", pulled);

        Ok(())
    }
}
