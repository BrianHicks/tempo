use super::duration::parse_duration;
use anyhow::Result;
use chrono::Duration;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct AddCommand {
    /// Text of the item to add
    name: Vec<String>,

    /// What category does this item belong to?
    #[clap(short, long)]
    tags: Vec<String>,

    /// Initial guess on cadence. Don't worry about this being incorrect; we'll
    /// find the right value over time! Supported units: hours (h), days (d),
    /// weeks (w), 30-day months (m), 365-day years (y)
    #[clap(short, long, default_value = "1d", parse(try_from_str = parse_duration))]
    cadence: Duration,

    /// When should this next be scheduled?
    #[clap(short, long)]
    next: Option<String>, // TODO: should be a chrono date or something
}

impl AddCommand {
    pub fn run(&self, mut store: crate::store::Store) -> Result<()> {
        let id = store.add(self.name.join(" "), &self.tags, self.cadence);
        println!("{:#?}", store.get(id));

        Ok(())
    }
}
