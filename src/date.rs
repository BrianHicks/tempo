use chrono::{Duration, Local, TimeZone, Utc};
use core::fmt::{self, Display, Formatter};
use core::ops::{Add, Sub};
use rusqlite::{
    types::{FromSql, FromSqlError, ToSqlOutput, Value, ValueRef},
    ToSql,
};
use serde::ser::{Serialize, SerializeStruct, Serializer};

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub struct Date {
    date: chrono::Date<Utc>,
}

impl Date {
    pub fn today() -> Self {
        Utc::today().into()
    }

    pub fn ymd(year: i32, month: u32, day: u32) -> Self {
        Utc.ymd(year, month, day).into()
    }
}

impl Display for Date {
    fn fmt(&self, out: &mut Formatter<'_>) -> fmt::Result {
        write!(
            out,
            "{}",
            self.date.with_timezone(&Local).format("%A, %B %d, %Y")
        )
    }
}

impl ToSql for Date {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::Owned(Value::Text(
            self.date.and_hms(0, 0, 0).to_rfc3339(),
        )))
    }
}

impl FromSql for Date {
    fn column_result(value: ValueRef<'_>) -> Result<Self, FromSqlError> {
        match value {
            ValueRef::Text(rfc3339_bytes) => {
                let rfc3339_str = match std::str::from_utf8(rfc3339_bytes) {
                    Ok(s) => s,
                    Err(err) => return Err(FromSqlError::Other(Box::new(err))),
                };

                match chrono::DateTime::parse_from_str(rfc3339_str, "%+") {
                    Ok(datetime) => Ok(datetime.date().with_timezone(&Utc).into()),
                    Err(err) => Err(FromSqlError::Other(Box::new(err))),
                }
            }
            _ => Err(FromSqlError::InvalidType),
        }
    }
}

impl Serialize for Date {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("Date", 2)?;
        s.serialize_field("date", &self.date.and_hms(0, 0, 0).to_rfc2822())?;
        s.serialize_field("human_date", &self.to_string())?;
        s.end()
    }
}

impl Add<Duration> for Date {
    type Output = Self;

    fn add(self, duration: Duration) -> Self::Output {
        Self::from(self.date + duration)
    }
}

impl Sub<Date> for Date {
    type Output = Duration;

    fn sub(self, other: Self) -> Self::Output {
        self.date - other.date
    }
}

impl Sub<Duration> for Date {
    type Output = Self;

    fn sub(self, duration: Duration) -> Self::Output {
        Self::from(self.date - duration)
    }
}

impl From<chrono::Date<Utc>> for Date {
    fn from(date: chrono::Date<Utc>) -> Self {
        Date { date }
    }
}
