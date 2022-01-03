use super::duration::parse_duration;
use anyhow::{anyhow, bail, Context, Result};
use chrono::{DateTime, Duration, TimeZone, Utc, Weekday};
use clap::Parser;

#[derive(Parser, Debug)]
pub struct AddCommand {
    /// Text of the item to add
    name: Vec<String>,

    /// What category does this item belong to?
    #[clap(short, long)]
    tags: Vec<String>,

    /// Initial guess on cadence. Don't worry about this being incorrect; we'll
    /// find the right value over time! Supported units: hours (h), days (d),
    /// weeks (w), 30-day months (m), 365-day years (y)
    #[clap(short, long, parse(try_from_str = parse_duration))]
    cadence: Option<Duration>,

    /// When should this next be scheduled?
    #[clap(short, long, parse(try_from_str = parse_utc_datetime))]
    next: Option<DateTime<Utc>>,
}

fn parse_utc_datetime(input: &str) -> Result<DateTime<Utc>> {
    let parsed = iso8601::date(input).map_err(|err| anyhow!(err)).context(
        "failed to parse a date and time in IS08601 format (YYYY-MM-DD, YYYY-MM-DDTHH:MM:SS, etc)",
    )?;

    // TODO: use the X_opt version of each constructor
    let date = match parsed {
        iso8601::Date::YMD { year, month, day } => Utc.ymd(year, month, day),
        iso8601::Date::Week { year, ww, d } => Utc.isoywd(
            year,
            ww,
            match d {
                1 => Weekday::Mon,
                2 => Weekday::Tue,
                3 => Weekday::Wed,
                4 => Weekday::Thu,
                5 => Weekday::Fri,
                6 => Weekday::Sat,
                7 => Weekday::Sun,
                _ => bail!("the only valid weekdays are 1-7"),
            },
        ),
        iso8601::Date::Ordinal { year, ddd } => Utc.yo(year, ddd),
    };

    Ok(date.and_hms(0, 0, 0))
}

impl AddCommand {
    pub fn run(&self, mut store: crate::store::Store) -> Result<()> {
        let now = Utc::now();

        let id = store.add(
            self.name.join(" "),
            &self.tags,
            self.get_cadence(now),
            self.get_next(now),
        );
        println!("{:#?}", store.get(id));

        Ok(())
    }

    fn get_cadence(&self, now: DateTime<Utc>) -> Duration {
        match (self.cadence, self.next) {
            (Some(cadence), _) => cadence,
            (None, Some(_)) => self.get_next(now) - now,
            (None, None) => Duration::days(1),
        }
    }

    fn get_next(&self, now: DateTime<Utc>) -> DateTime<Utc> {
        self.next.unwrap_or_else(|| now + self.get_cadence(now))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn default() -> AddCommand {
        AddCommand {
            name: Vec::default(),
            tags: Vec::default(),
            cadence: None,
            next: None,
        }
    }

    #[test]
    fn next_is_used() {
        let next = Utc::now() + Duration::weeks(1);

        let mut command = default();
        command.next = Some(next);

        assert_eq!(next, command.get_next(Utc::now()))
    }

    #[test]
    fn next_is_calculated_based_on_cadence() {
        let now = Utc::now();
        let duration = Duration::days(1);

        let mut command = default();
        command.cadence = Some(duration);
        command.next = None; // just to be explicit

        assert_eq!(now + duration, command.get_next(now));
    }

    #[test]
    fn cadence_is_used() {
        let cadence = Duration::weeks(1);

        let mut command = default();
        command.cadence = Some(cadence);

        assert_eq!(cadence, command.get_cadence(Utc::now()))
    }

    #[test]
    fn cadence_is_calculated_based_on_next() {
        let now = Utc::now();
        let next = now + Duration::weeks(1);

        let mut command = default();
        command.cadence = None; // just to be explicit
        command.next = Some(next);

        assert_eq!(next - now, command.get_cadence(now));
    }

    #[test]
    fn cadence_is_one_day_if_neither_is_present() {
        let mut command = default();
        command.cadence = None; // just to be explicit
        command.next = None; // just to be explicit

        assert_eq!(Duration::days(1), command.get_cadence(Utc::now()))
    }
}
