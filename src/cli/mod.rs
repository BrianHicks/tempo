pub mod add;
pub mod edit;
pub mod finish;
pub mod pull;

use anyhow::{Context, Result};
use chrono::{DateTime, TimeZone, Utc};

fn parse_utc_datetime(input: &str) -> Result<DateTime<Utc>> {
    Utc.datetime_from_str(input, "%Y-%m-%dT%H:%M:%S")
        .or_else(|_| Utc.datetime_from_str(&format!("{}T00:00:00", input), "%Y-%m-%dT%H:%M:%S"))
        .context("couldn't parse a date")
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
                Utc.ymd(2022, 1, 1).and_hms(0, 0, 0),
                parse_utc_datetime("2022-01-01").unwrap()
            );
        }

        #[test]
        fn datetime() {
            assert_eq!(
                Utc.ymd(2022, 1, 1).and_hms(3, 2, 1),
                parse_utc_datetime("2022-01-01T03:02:01").unwrap()
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
            assert_eq!(
                "input is out of range",
                parse_utc_datetime("2022-13-01")
                    .unwrap_err()
                    .root_cause()
                    .to_string(),
            );
        }

        #[test]
        fn out_of_range_day() {
            assert_eq!(
                "input is out of range",
                parse_utc_datetime("2022-02-30")
                    .unwrap_err()
                    .root_cause()
                    .to_string(),
            );
        }
    }
}
