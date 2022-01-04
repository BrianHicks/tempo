mod cli;
mod format;
mod item;
mod pid;
mod serde_duration;
mod store;

use crate::format::Format;
use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Opts {
    #[clap(subcommand)]
    command: Option<Command>,

    /// How to format the output. If you're just using this on the command line,
    /// you'll probably be fine with never touching this. If you're integrating
    /// with another system, however, you might want to use the JSON output.
    #[clap(long, short, arg_enum, default_value = "human")]
    format: Format,

    /// Where to store config and data about items and their repeats. If absent,
    /// we'll figure out the right place for this based on the platform you're
    /// running (e.g. Linux will use the XDG specification, macOS will put stuff
    /// in `~/Application Support`, etc.)
    #[clap(long, short, env = "TEMPO_STORE_PATH")]
    store_path: Option<PathBuf>,
}

#[derive(Parser, Debug)]
enum Command {
    /// Add a new item to the store
    Add(cli::add::AddCommand),
}

impl Opts {
    fn try_main(&self) -> Result<()> {
        let store = self.get_store().context("couldn't get a store")?;

        match &self.command {
            Some(Command::Add(add)) => add.run(store, self.format)?,

            None => {
                println!("{:#?}", self)
            }
        };

        Ok(())
    }

    fn get_store(&self) -> Result<store::Store> {
        let path = self.get_store_path().context("couldn't get a store path")?;

        if !path.exists() {
            Ok(store::Store::default())
        } else {
            let source = std::fs::read_to_string(&path)
                .with_context(|| format!("couldn't read from {}", path.display()))?;

            toml::from_str(&source)
                .with_context(|| format!("couldn't decode a store from {}", path.display()))
        }
    }

    fn get_store_path(&self) -> Result<PathBuf> {
        if let Some(explicit) = &self.store_path {
            return Ok(explicit.to_path_buf());
        }

        let dirs = directories::ProjectDirs::from("zone", "bytes", "tempo")
            .context("couldn't load HOME (set --store-path explicitly to get around this.)")?;

        // Note: YAML might not be the final format. TOML also seems promising?
        Ok(dirs.data_dir().join("tempo.yaml"))
    }
}

fn main() {
    let opts = Opts::parse();

    if let Err(err) = opts.try_main() {
        eprintln!("{:?}", err);
        std::process::exit(1);
    }
}
