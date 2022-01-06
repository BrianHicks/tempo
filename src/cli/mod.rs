pub mod add;
pub mod finish;

use anyhow::{Context, Result};
use chrono::{DateTime, TimeZone, Utc};

fn parse_utc_datetime(input: &str) -> Result<DateTime<Utc>> {
    Utc.datetime_from_str(input, "%Y-%m-%dT%H:%M:%S")
        .or_else(|_| Utc.datetime_from_str(&format!("{}T00:00:00", input), "%Y-%m-%dT%H:%M:%S"))
        .context("couldn't parse a date")
}
