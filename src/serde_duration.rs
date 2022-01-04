use chrono::Duration;
use serde::de::{self, Visitor};
use serde::{Deserializer, Serializer};
use std::fmt;

pub fn serialize<S>(d: &Duration, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_i64(d.num_milliseconds())
}

struct DurationVisitor {}

impl<'de> Visitor<'de> for DurationVisitor {
    type Value = Duration;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "expecting a number of milliseconds")
    }

    fn visit_i64<E>(self, ms: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Duration::milliseconds(ms))
    }
}

pub fn deserialize<'de, D>(d: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    d.deserialize_i64(DurationVisitor {})
}
