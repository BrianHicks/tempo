use crate::format::Format;
use crate::item::Item;
use crate::tag::Tag;
use anyhow::{Context, Result};
use chrono::Local;
use clap::Parser;
use rusqlite::Connection;

#[derive(Debug, Parser)]
pub struct Command {
    /// Tag to filter by
    #[clap(long)]
    tag: Option<String>,
}

impl Command {
    pub fn run(&self, conn: &Connection, format: Format) -> Result<()> {
        let pulled = Item::all(conn).context("could not pull items")?;

        let tag_id = match &self.tag {
            Some(tag_name) => Some(
                Tag::get_by_name(conn, tag_name)
                    .context("could not get tag with that name")?
                    .id,
            ),
            None => None,
        };

        let filtered: Vec<Item> = pulled
            .filter(|item| tag_id.map_or(true, |id| item.id == id))
            .collect();

        match format {
            Format::Human => {
                for item in filtered {
                    println!(
                        "{}: {} (due {})",
                        item.id,
                        item.text,
                        item.next.with_timezone(&Local).format("%A, %B %d, %Y")
                    );
                }
            }
            Format::Json => println!(
                "{}",
                serde_json::to_string(&filtered).context("could not dump pulled items to JSON")?
            ),
        };

        Ok(())
    }
}
