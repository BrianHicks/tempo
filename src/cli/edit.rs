use crate::format::Format;
use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::Connection;

#[derive(Debug, clap::Parser)]
pub struct EditCommand {
    /// ID of the item to edit
    id: usize,

    /// New text to add to the item
    text: Vec<String>,

    /// Change this item's tag
    #[clap(short, long)]
    tag: Option<String>,

    /// Change when this item will be scheduled next
    #[clap(long, conflicts_with_all(&["earlier", "much-earlier", "later", "much-later"]), parse(try_from_str = super::parse_utc_datetime))]
    next: Option<DateTime<Utc>>,

    /// Tweak this item's next schedule to be a little earlier
    #[clap(long, short('e'), conflicts_with_all(&["next", "much-earlier", "later", "much-later"]))]
    earlier: bool,

    /// Tweak this item's next schedule to be much earlier
    #[clap(long, short('E'), conflicts_with_all(&["next", "earlier", "later", "much-later"]))]
    much_earlier: bool,

    /// Tweak this item's next schedule to be a little later
    #[clap(long, short('l'), conflicts_with_all(&["next", "earlier", "much-earlier", "much-later"]))]
    later: bool,

    /// Tweak this item's next schedule to be much later
    #[clap(long, short('L'), conflicts_with_all(&["next", "earlier", "much-earlier", "later"]))]
    much_later: bool,
}

impl EditCommand {
    pub fn run(&self, _conn: &Connection, _format: Format) -> Result<()> {
        println!("{:#?}", self);

        todo!("implement EditCommand.run")
    }
}
