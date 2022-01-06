use crate::format::Format;
use anyhow::Result;
use chrono::{DateTime, Utc};

#[derive(Debug, clap::Parser)]
pub struct FinishCommand {
    /// ID of the item to finish
    id: usize,

    /// Explicitly set the next repeat
    #[clap(long, conflicts_with_all(&["early", "very-early", "late", "very-late"]), parse(try_from_str = super::parse_utc_datetime))]
    next: Option<DateTime<Utc>>,

    /// Mark this item as having been scheduled a little early
    #[clap(long, short('e'), conflicts_with_all(&["next", "very-early", "late", "very-late"]))]
    early: bool,

    /// Mark this item as having been scheduled very early
    #[clap(long, short('E'), conflicts_with_all(&["next", "early", "late", "very-late"]))]
    very_early: bool,

    /// Mark this item as having been scheduled a little late
    #[clap(long, short('l'), conflicts_with_all(&["next", "early", "very-early", "very-late"]))]
    late: bool,

    /// Mark this item as having been scheduled very late
    #[clap(long, short('L'), conflicts_with_all(&["next", "early", "very-early", "late"]))]
    very_late: bool,
}

impl FinishCommand {
    pub fn run(&self, _format: Format) -> Result<()> {
        todo!("reimplement FinishCommand.run")
    }
}
