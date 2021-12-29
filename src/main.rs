use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Opts {
    #[clap(long)]
    command: String,
}

fn main() {
    if let Err(err) = try_main() {
        eprintln!("{:#?}", err);
        std::process::exit(1);
    }
}

fn try_main() -> Result<()> {
    let opts = Opts::parse();

    println!("{:#?}", opts);

    Ok(())
}
