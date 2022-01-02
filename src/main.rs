use anyhow::{bail, Context, Result};
use chrono::Duration;
use clap::Parser;

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
    Add(AddCommand),
}

#[derive(Parser, Debug)]
struct AddCommand {
    /// Text of the item to add
    name: Vec<String>,

    /// What category does this item belong to?
    #[clap(short, long)]
    tags: Vec<String>,

    /// Initial guess on cadence. Don't worry about this being incorrect; we'll
    /// find the right value over time! Supported units: hours (h), days (d),
    /// weeks (w), 30-day months (m), 365-day years (y)
    #[clap(short, long, default_value = "1d", parse(try_from_str = parse_duration))]
    cadence: Duration,

    /// When should this next be scheduled?
    #[clap(short, long)]
    next: Option<String>, // TODO: should be a chrono date or something
}

fn main() {
    if let Err(err) = try_main() {
        eprintln!("{:#?}", err);
        std::process::exit(1);
    }
}

fn try_main() -> Result<()> {
    let opts = Opts::parse();

    let mut store = store::Store::default(); // TODO: load from disk

    match opts.command {
        Some(Command::Add(add)) => {
            let id = store.add(add.name.join(" "), add.tags, add.cadence);
            println!("{:#?}", store.get(id));
        }

        None => {
            println!("{:#?}", opts)
        }
    };

    Ok(())
}

/// Parse a simple duration string: a number followed by a unit of hours (h),
/// days (d), or weeks (w). Months and years are supported but are rougher:
/// we'll assume a 30-day month and a 365-day year.
fn parse_duration(input: &str) -> Result<Duration> {
    let mut digits_offset = 0;
    let mut tag = None;

    for c in input.chars() {
        // meaning: we've already assigned a value to tag, which means we're
        // somewhere after it in the input string. There shouldn't be anything
        // here, so we can just bail.
        if tag != None {
            bail!("got more input than I expected. I wanted to see a number and tag, like 10d.")
        }
        // meaning: we're done with the digits but haven't assigned a tag yet.
        else if !c.is_numeric() {
            tag = Some(c);
        }
        // meaning: we haven't completed scanning the digits yet.
        else {
            digits_offset += 1;
        }
    }

    let digits = &input[0..digits_offset];
    if digits.is_empty() {
        bail!("I expected to see some numbers then a tag (like 1d), but I didn't see any numbers!");
    }

    let amount: i64 = digits
        .parse()
        .with_context(|| format!("could not parse a number from `{}`", digits))?;

    let out = match tag {
        Some('h') => Duration::hours(amount),
        Some('d') => Duration::days(amount),
        Some('w') => Duration::weeks(amount),
        Some('m') => Duration::days(amount * 30),
        Some('y') => Duration::days(amount * 365),
        _ => bail!("expected to see a tag (h, d, w, m, y) after the amount"),
    };

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_duration_hours() {
        assert_eq!(Duration::hours(1), parse_duration("1h").unwrap());
    }

    #[test]
    fn parse_duration_multiple() {
        assert_eq!(Duration::hours(24), parse_duration("24h").unwrap());
    }

    #[test]
    fn parse_duration_days() {
        assert_eq!(Duration::days(1), parse_duration("1d").unwrap());
    }

    #[test]
    fn parse_duration_weeks() {
        assert_eq!(Duration::weeks(1), parse_duration("1w").unwrap());
    }

    #[test]
    fn parse_duration_months() {
        assert_eq!(Duration::days(30), parse_duration("1m").unwrap());
    }

    #[test]
    fn parse_duration_years() {
        assert_eq!(Duration::days(365), parse_duration("1y").unwrap());
    }

    #[test]
    fn parse_duration_extra() {
        assert!(parse_duration("1dd").is_err());
    }

    #[test]
    fn parse_duration_leading() {
        assert!(parse_duration("d").is_err());
    }
}
