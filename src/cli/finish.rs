use crate::format::Format;
use crate::store::Store;
use anyhow::Result;

#[derive(Debug, clap::Parser)]
pub struct FinishCommand {
    /// ID of the item to finish
    id: String,
}

impl FinishCommand {
    pub fn run(&self, store: &mut Store, format: Format) -> Result<()> {
        Ok(())
    }
}
