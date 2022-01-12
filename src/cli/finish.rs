use crate::format::Format;
use crate::item::{Bump, Item};
use anyhow::Result;
use clap::Parser;
use rusqlite::Connection;

#[derive(Parser, Debug)]
pub struct Command {
    /// ID of the item to finish
    #[clap(required(true))]
    text: u64,

    /// Whether this item was scheduled too early, too late, etc. We will
    /// use this feedback to schedule the next repetition of the item.
    #[clap(arg_enum)]
    bump: Bump,
}

impl Command {
    pub fn run(&self, _conn: &Connection, _format: Format) -> Result<()> {
        println!("{:#?}", self);
        todo!()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::TimeZone;
}
