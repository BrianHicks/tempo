use crate::format::Format;
use crate::store::Store;
use anyhow::{Context, Result};

#[derive(Debug, clap::Parser)]
pub struct FinishCommand {
    /// ID of the item to finish
    id: String,
}

impl FinishCommand {
    pub fn run(&self, store: &mut Store, _format: Format) -> Result<()> {
        let item = store
            .get_mut(&self.id)
            .with_context(|| format!("couldn't get item with ID {}", &self.id))?;

        item.finish()
            .with_context(|| format!("couldn't finish item with ID {}", &self.id))?;

        Ok(())
    }
}
