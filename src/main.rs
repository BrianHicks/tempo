mod cli;
mod format;
mod item;
mod pid;
mod serde_duration;
mod store;

use crate::format::Format;
use anyhow::Result;
use clap::Parser;

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
}

#[derive(Parser, Debug)]
enum Command {
    /// Add a new item to the store
    Add(cli::add::AddCommand),
}

impl Opts {
    fn try_main(&self) -> Result<()> {
        let store = store::Store::default(); // TODO: load from disk

        match &self.command {
            Some(Command::Add(add)) => add.run(store, self.format)?,

            None => {
                println!("{:#?}", self)
            }
        };

        Ok(())
    }
}

fn main() {
    let opts = Opts::parse();

    if let Err(err) = opts.try_main() {
        eprintln!("{:#?}", err);
        std::process::exit(1);
    }
}
