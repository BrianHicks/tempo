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

fn main() {
    if let Err(err) = try_main() {
        eprintln!("{:#?}", err);
        std::process::exit(1);
    }
}

fn try_main() -> Result<()> {
    let opts = Opts::parse();

    let store = store::Store::default(); // TODO: load from disk

    match opts.command {
        Some(Command::Add(add)) => add.run(store, opts.format)?,

        None => {
            println!("{:#?}", opts)
        }
    };

    Ok(())
}
