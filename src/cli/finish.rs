use crate::format::Format;
use anyhow::Result;

#[derive(Debug, clap::Parser)]
pub struct FinishCommand {
    /// ID of the item to finish
    id: String,
}

impl FinishCommand {
    pub fn run(&self, _format: Format) -> Result<()> {
        todo!("reimplement FinishCommand.run")
    }
}
