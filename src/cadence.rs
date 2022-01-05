use rusqlite::{
    types::{FromSql, FromSqlError, ToSqlOutput, Value, ValueRef},
    ToSql,
};

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
