pub mod add;
pub mod all;
pub mod delete;
pub mod edit;
pub mod finish;
pub mod ready;

use crate::cadence::Cadence;
use crate::date::Date;
use anyhow::{Context, Result};
use chrono::{Local, TimeZone, Utc};
use std::str::FromStr;

fn parse_utc_datetime(input: &str) -> Result<Date> {
    let base = if input == "today" {
        Local::now()
    } else {
        Local
            .datetime_from_str(&format!("{}T00:00:00", input), "%Y-%m-%dT%H:%M:%S")
            .or_else(|_| Cadence::from_str(input).map(|cadence| Local::now() + cadence))
            .context("couldn't parse a date")?
    };

    Ok(base.with_timezone(&Utc).date().into())
}

#[cfg(test)]
mod test {
    use super::*;

    #[cfg(test)]
    mod parse_utc_datetime {
        use super::*;

        #[test]
        fn date() {
            assert_eq!(
                Date::from(Local.ymd(2022, 1, 1).with_timezone(&Utc)),
                parse_utc_datetime("2022-01-01").unwrap()
            );
        }

        #[test]
        fn today() {
            assert_eq!(
                Date::from(Local::today().with_timezone(&Utc)),
                parse_utc_datetime("today").unwrap()
            );
        }

        #[test]
        fn cadence() {
            assert_eq!(
                Date::from(Utc::today() + chrono::Duration::days(7)),
                parse_utc_datetime("1w").unwrap()
            );
        }

        #[test]
        fn blank_fails() {
            assert_eq!(
                "couldn't parse a date",
                parse_utc_datetime("").unwrap_err().to_string(),
            );
        }

        #[test]
        fn nonsense_fails() {
            assert_eq!(
                "couldn't parse a date",
                parse_utc_datetime("not a date").unwrap_err().to_string(),
            );
        }

        #[test]
        fn out_of_range_month() {
            assert!(parse_utc_datetime("2022-13-01").is_err());
        }

        #[test]
        fn out_of_range_day() {
            assert!(parse_utc_datetime("2022-02-30").is_err());
        }
    }
}
