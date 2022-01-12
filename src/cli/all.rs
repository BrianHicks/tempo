use crate::format::Format;
use crate::item::Item;
use anyhow::{Context, Result};
use chrono::Local;
use clap::Parser;
use rusqlite::Connection;

#[derive(Debug, Parser)]
pub struct Command {}

impl Command {
    pub fn run(conn: &Connection, format: Format) -> Result<()> {
        let pulled: Vec<Item> = Item::all(conn).context("could not pull items")?.collect();

        match format {
            Format::Human => {
                for item in pulled {
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
                serde_json::to_string(&pulled).context("could not dump pulled items to JSON")?
            ),
        };

        Ok(())
    }
}
