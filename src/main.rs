mod cadence;
mod cli;
mod format;
// mod serde_duration;

use crate::cadence::Cadence;
use crate::format::Format;
use anyhow::{Context, Result};
use clap::Parser;
use rusqlite::{params, Connection};
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

    /// Where to store the Tempo database. If absent, we'll figure out the
    /// right place for this based on the platform you're running (e.g. Linux
    /// will use the XDG specification, macOS will put stuff in `~/Application
    /// Support`, etc.)
    #[clap(long, short, env = "TEMPO_DB_PATH")]
    db_path: Option<PathBuf>,
}

#[derive(Parser, Debug)]
enum Command {
    /// Add a new item to the store
    Add(cli::add::AddCommand),

    /// Finish a scheduled item
    Finish(cli::finish::FinishCommand),
}

impl Opts {
    fn try_main(&self) -> Result<()> {
        let mut conn = self.get_store()?;
        embedded::migrations::runner()
            .run(&mut conn)
            .context("couldn't migrate the database's data!")?;

        println!(
            "{:#?}",
            conn.execute(
                "INSERT INTO items (text, cadence) VALUES (?, ?)",
                params!["Test", Cadence::hours(1)],
            )
        );

        match &self.command {
            Some(Command::Add(add)) => add.run(self.format),

            Some(Command::Finish(finish)) => finish.run(self.format),

            None => {
                println!("{:#?}", self);
                todo!("implement no-subcommand case in Opts.try_main");
            }
        }
    }

    fn get_store(&self) -> Result<Connection> {
        let path = self
            .get_db_path()
            .context("couldn't get the database path")?;
        Connection::open(path).context("couldn't open the database")
    }

    fn get_db_path(&self) -> Result<PathBuf> {
        if let Some(explicit) = &self.db_path {
            return Ok(explicit.to_path_buf());
        }

        let dirs = directories::ProjectDirs::from("zone", "bytes", "tempo")
            .context("couldn't load HOME (set --db-path explicitly to get around this.)")?;

        Ok(dirs.data_dir().join("tempo.sqlite3"))
    }
}

mod embedded {
    refinery::embed_migrations!("migrations");
}

fn main() {
    let opts = Opts::parse();

    env_logger::init();

    if let Err(err) = opts.try_main() {
        eprintln!("{:?}", err);
        std::process::exit(1);
    }
}
