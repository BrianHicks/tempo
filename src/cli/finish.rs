use crate::format::Format;
use crate::store::Store;
use anyhow::Result;

#[derive(Debug, clap::Parser)]
pub struct FinishCommand {
    id: String,
}

impl FinishCommand {
    pub fn run(&self, store: &mut Store, format: Format) -> Result<()> {
        Ok(())
    }
}
