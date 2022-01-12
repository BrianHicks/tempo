use crate::format::Format;
use crate::item::{Bump, Item};
use anyhow::{Context, Result};
use chrono::Local;
use clap::Parser;
use rusqlite::Connection;

#[derive(Parser, Debug)]
pub struct Command {
    /// ID of the item to finish
    #[clap(required(true))]
    id: u64,

    /// Whether this item was scheduled too early, too late, etc. We will
    /// use this feedback to schedule the next repetition of the item.
    #[clap(arg_enum)]
    bump: Bump,
}

impl Command {
    pub fn run(&self, conn: &Connection, format: Format) -> Result<()> {
        let mut item = Item::get(self.id, conn)
            .with_context(|| format!("couldn't load item with ID {}", self.id))?;

        let adjustment = item
            .finish(&self.bump)
            .with_context(|| format!("couldn't finish item with ID {}", self.id))?;

        match format {
            Format::Human =>
                println!(
                    "Finished! For next time, I bumped the schedule by {} so the next time you'll see this will be {}",
                    adjustment,
                    item.next.with_timezone(&Local).to_rfc2822()
                ),
            Format::Json => println!("{}", serde_json::to_string(&item).context("couldn't convert item to JSON")?),
        }

        Ok(())
    }
}
