use crate::format::Format;
use anyhow::{bail, Context, Result};
use clap::Parser;
use rusqlite::Connection;

#[derive(Debug, Parser)]
pub struct Command {
    /// ID of the item to drop
    id: u64,
}

impl Command {
    pub fn run(&self, conn: &Connection, format: Format) -> Result<()> {
        match conn.execute("DELETE FROM items WHERE id = ?", [self.id]) {
            Ok(0) => bail!("Could not find item with ID {}", self.id),
            Ok(1) => match format {
                Format::Human => println!("Deleted item with ID {}", self.id),
                Format::Json => println!("{}", serde_json::to_string(&true)?),
            },
            Ok(more_than_one) => {
                bail!(
                    "Deleted {} rows for ID {}. Please report this as a bug!",
                    more_than_one,
                    self.id
                )
            }
            Err(err) => {
                return Err(err).with_context(|| format!("could not drop item with ID {}", self.id))
            }
        }

        Ok(())
    }
}
