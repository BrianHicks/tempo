use chrono::{DateTime, Duration, TimeZone};
use core::convert::From;
use core::ops::Add;
use core::str::FromStr;
use rusqlite::{
    types::{FromSql, FromSqlError, ToSqlOutput, Value, ValueRef},
    ToSql,
};
use thiserror::Error;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Cadence {
    minutes: i64,
}

impl Cadence {
    pub fn minutes(minutes: i64) -> Cadence {
        Cadence { minutes }
    }

    pub fn hours(hours: i64) -> Cadence {
        Self::minutes(hours * 60)
    }

    pub fn days(days: i64) -> Cadence {
        Self::hours(days * 24)
    }

    pub fn weeks(weeks: i64) -> Cadence {
        Self::days(weeks * 7)
    }

    pub fn months(months: i64) -> Cadence {
        Self::days(months * 30)
    }

    pub fn years(years: i64) -> Cadence {
        Self::days(years * 365)
    }
}

impl Default for Cadence {
    fn default() -> Cadence {
        Cadence::hours(1)
    }
}

impl ToSql for Cadence {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::Owned(Value::Integer(self.minutes)))
    }
}

impl FromSql for Cadence {
    fn column_result(value: ValueRef<'_>) -> Result<Self, FromSqlError> {
        match value {
            ValueRef::Integer(minutes) => Ok(Self::minutes(minutes)),
            _ => Err(FromSqlError::InvalidType),
        }
    }
}

impl FromStr for Cadence {
    type Err = ParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut digits_offset = 0;
        let mut tag = None;

        for c in input.chars() {
            // meaning: we've already assigned a value to tag, which means we're
            // somewhere after it in the input string. There shouldn't be anything
            // here, so we can just bail.
            if tag != None {
                return Err(Self::Err::ExtraStuff);
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
            return Err(Self::Err::MissingDigits);
        }

        let amount: i64 = match digits.parse() {
            Ok(num) => num,
            Err(err) => return Err(Self::Err::CouldntParseNumber(digits.to_string(), err)),
        };

        let out = match tag {
            Some('h') => Self::hours(amount),
            Some('d') => Self::days(amount),
            Some('w') => Self::weeks(amount),
            Some('m') => Self::months(amount),
            Some('y') => Self::years(amount),
            _ => return Err(Self::Err::MissingTag),
        };

        Ok(out)
    }
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("I expected to see some numbers, then a tag (like 1d), but I didn't see any numbers")]
    MissingDigits,
    #[error("I couldn't parse a number from {0}: {1}")]
    CouldntParseNumber(String, core::num::ParseIntError),
    #[error("I expected to see a tag (h, d, w, m, y) after the amount")]
    MissingTag,
    #[error(
        "I got extra stuff after the number and tag. Reduce it to something like 30d and try again"
    )]
    ExtraStuff,
}

impl<TZ: TimeZone> Add<DateTime<TZ>> for Cadence {
    type Output = DateTime<TZ>;

    fn add(self, dt: Self::Output) -> Self::Output {
        dt + Duration::minutes(self.minutes)
    }
}

impl<TZ: TimeZone> Add<Cadence> for DateTime<TZ> {
    type Output = DateTime<TZ>;

    fn add(self, cadence: Cadence) -> Self::Output {
        cadence + self
    }
}

impl From<Duration> for Cadence {
    fn from(duration: Duration) -> Cadence {
        Cadence::minutes(duration.num_minutes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod from_str {
        use super::*;

        #[test]
        fn parse_duration_hours() {
            assert_eq!(Cadence::hours(1), Cadence::from_str("1h").unwrap());
        }

        #[test]
        fn parse_duration_multiple() {
            assert_eq!(Cadence::hours(24), Cadence::from_str("24h").unwrap());
        }

        #[test]
        fn parse_duration_days() {
            assert_eq!(Cadence::days(1), Cadence::from_str("1d").unwrap());
        }

        #[test]
        fn parse_duration_weeks() {
            assert_eq!(Cadence::weeks(1), Cadence::from_str("1w").unwrap());
        }

        #[test]
        fn parse_duration_months() {
            assert_eq!(Cadence::months(1), Cadence::from_str("1m").unwrap());
        }

        #[test]
        fn parse_duration_years() {
            assert_eq!(Cadence::years(1), Cadence::from_str("1y").unwrap());
        }

        #[test]
        fn parse_duration_extra() {
            assert!(Cadence::from_str("1dd").is_err());
        }

        #[test]
        fn parse_duration_leading() {
            assert!(Cadence::from_str("d").is_err());
        }
    }
}
