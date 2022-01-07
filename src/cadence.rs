use chrono::{DateTime, Duration, TimeZone};
use core::convert::From;
use core::fmt::{self, Display, Formatter};
use core::ops::Add;
use core::str::FromStr;
use rusqlite::{
    types::{FromSql, FromSqlError, ToSqlOutput, Value, ValueRef},
    ToSql,
};
use thiserror::Error;

static HOURS: i64 = 60;
static DAYS: i64 = HOURS * 24;
static WEEKS: i64 = DAYS * 7;
static MONTHS: i64 = DAYS * 30;
static YEARS: i64 = DAYS * 365;

#[derive(Debug, PartialEq, Eq, Clone, Copy, serde::Serialize)]
pub struct Cadence {
    minutes: i64,
}

impl Cadence {
    pub fn minutes(minutes: i64) -> Cadence {
        Cadence { minutes }
    }

    pub fn hours(hours: i64) -> Cadence {
        Self::minutes(hours * HOURS)
    }

    pub fn days(days: i64) -> Cadence {
        Self::minutes(days * DAYS)
    }

    pub fn weeks(weeks: i64) -> Cadence {
        Self::minutes(weeks * WEEKS)
    }

    pub fn months(months: i64) -> Cadence {
        Self::minutes(months * MONTHS)
    }

    pub fn years(years: i64) -> Cadence {
        Self::minutes(years * YEARS)
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
            // meaning: we haven't completed scanning the digits yet.
            else if c.is_numeric() {
                digits_offset += 1;
            }
            // meaning: we're done with the digits but haven't assigned a tag yet.
            else {
                tag = Some(c);
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

impl Display for Cadence {
    fn fmt(&self, out: &mut Formatter<'_>) -> fmt::Result {
        if self.minutes >= YEARS * 2 {
            write!(out, "~{}y", (self.minutes as f64 / YEARS as f64).round())
        } else if self.minutes >= MONTHS * 3 {
            write!(out, "~{}m", (self.minutes as f64 / MONTHS as f64).round())
        } else if self.minutes >= WEEKS {
            if self.minutes % WEEKS == 0 {
                write!(out, "{}w", self.minutes / WEEKS)
            } else {
                write!(out, "~{}w", (self.minutes as f64 / WEEKS as f64).round())
            }
        } else if self.minutes >= DAYS {
            if self.minutes % DAYS == 0 {
                write!(out, "{}d", self.minutes / DAYS)
            } else {
                write!(out, "~{}d", (self.minutes as f64 / DAYS as f64).round())
            }
        } else if self.minutes >= HOURS {
            if self.minutes % HOURS == 0 {
                write!(out, "{}h", self.minutes / HOURS)
            } else {
                write!(out, "~{}h", (self.minutes as f64 / HOURS as f64).round())
            }
        } else {
            write!(out, "<1h")
        }
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

impl Add<Cadence> for Cadence {
    type Output = Cadence;

    fn add(self, other: Self) -> Self {
        Self::minutes(self.minutes + other.minutes)
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

    mod display {
        use super::*;

        #[test]
        fn minutes() {
            assert_eq!("<1h", Cadence::minutes(1).to_string())
        }

        #[test]
        fn exact_hours() {
            assert_eq!("1h", Cadence::hours(1).to_string())
        }

        #[test]
        fn partial_hours() {
            assert_eq!("~2h", Cadence::minutes(90).to_string())
        }

        #[test]
        fn exact_days() {
            assert_eq!("1d", Cadence::days(1).to_string())
        }

        #[test]
        fn partial_days() {
            assert_eq!("~2d", Cadence::hours(36).to_string())
        }

        #[test]
        fn weeks() {
            assert_eq!("1w", Cadence::weeks(1).to_string())
        }

        #[test]
        fn partial_weeks() {
            assert_eq!("~2w", Cadence::days(11).to_string())
        }

        #[test]
        fn months() {
            // Note transition here from ~1m. That's just a little too
            // rough-grained. It may change in the future.
            assert_eq!("~4w", Cadence::months(1).to_string())
        }

        #[test]
        fn partial_months() {
            // Note transition here from ~2m. That's just a little too
            // rough-grained. It may change in the future.
            assert_eq!("~6w", Cadence::days(45).to_string())
        }

        #[test]
        fn quarters() {
            assert_eq!("~3m", Cadence::months(3).to_string())
        }

        #[test]
        fn years() {
            assert_eq!("~12m", Cadence::years(1).to_string())
        }

        #[test]
        fn multiple_years() {
            assert_eq!("~2y", Cadence::years(2).to_string())
        }
    }
}
