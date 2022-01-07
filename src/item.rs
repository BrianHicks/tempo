use crate::cadence::Cadence;
use crate::pid::Pid;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::Connection;

#[derive(Debug, serde::Serialize)]
pub struct Item {
    pub id: u64,
    pub text: String,

    // scheduling
    pub cadence: Cadence,
    pub next: DateTime<Utc>,
    pub last: Option<DateTime<Utc>>,

    #[serde(flatten)]
    pub pid: Pid,
}

impl Item {
    pub fn get(id: u64, conn: &Connection) -> Result<Item> {
        conn.query_row(
                "SELECT id, text, cadence, next, last, proportional_factor, integral, integral_factor, last_error, derivative_factor FROM items WHERE id = ?",
                [id],
                |row| Ok(Item{
                    id: row.get(0)?,
                    text: row.get(1)?,
                    cadence: row.get(2)?,
                    next: row.get(3)?,
                    last: row.get(4)?,
                    pid: Pid {
                        proportional_factor: row.get(5)?,
                        integral: row.get(6)?,
                        integral_factor: row.get(7)?,
                        last_error: row.get(8)?,
                        derivative_factor: row.get(9)?,
                    }
                }),
            ).with_context(|| format!("could not retrieve item with ID {}", id))
    }
}
