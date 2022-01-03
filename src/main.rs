use anyhow::Result;
use clap::Parser;

mod cli;
mod item;
mod pid;
mod store;

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Opts {
    #[clap(subcommand)]
    command: Option<Command>,
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
        Some(Command::Add(add)) => add.run(store)?,

        None => {
            println!("{:#?}", opts)
        }
    };

    Ok(())
}
