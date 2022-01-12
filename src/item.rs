use crate::cadence::Cadence;
use crate::pid::Pid;
use anyhow::{bail, Context, Result};
use chrono::{DateTime, Local, Utc};
use rusqlite::{params, Connection};

#[derive(Debug, serde::Serialize, PartialEq)]
pub struct Item {
    pub id: u64,
    pub text: String,
    pub tag_id: Option<u64>,

    // scheduling
    pub cadence: Cadence,
    pub next: DateTime<Utc>,

    #[serde(flatten)]
    pub pid: Pid,
}

#[derive(clap::ArgEnum, Clone, Debug)]
pub enum Bump {
    MuchEarlier,
    Earlier,
    JustRight,
    Later,
    MuchLater,
}

impl Item {
    pub fn get(id: u64, conn: &Connection) -> Result<Item> {
        conn.query_row(
            "SELECT id, text, tag_id, cadence, next, integral, last_error FROM items WHERE id = ?",
            [id],
            |row| {
                Ok(Item {
                    id: row.get(0)?,
                    text: row.get(1)?,
                    tag_id: row.get(2)?,
                    cadence: row.get(3)?,
                    next: row.get(4)?,
                    pid: Pid {
                        integral: row.get(5)?,
                        last_error: row.get(6)?,
                    },
                })
            },
        )
        .with_context(|| format!("could not retrieve item with ID {}", id))
    }

    pub fn all(conn: &Connection) -> Result<impl Iterator<Item = Item>> {
        let mut statement = conn
            .prepare("SELECT id, text, tag_id, cadence, next, integral, last_error FROM items ORDER BY id ASC")
            .context("could not prepare query to get all items")?;

        let items = statement
            .query_map([], |row| {
                Ok(Item {
                    id: row.get(0)?,
                    text: row.get(1)?,
                    tag_id: row.get(2)?,
                    cadence: row.get(3)?,
                    next: row.get(4)?,
                    pid: Pid {
                        integral: row.get(5)?,
                        last_error: row.get(6)?,
                    },
                })
            })?
            .collect::<rusqlite::Result<Vec<Item>>>()
            .context("could not pull rows")?;

        Ok(items.into_iter())
    }

    pub fn due(conn: &Connection) -> Result<impl Iterator<Item = Item>> {
        let mut statement = conn.prepare("SELECT id, text, tag_id, cadence, next, integral, last_error FROM items WHERE next <= ? ORDER BY next ASC").context("could not prepare query to get items")?;

        let items = statement
            .query_map([Utc::now()], |row| {
                Ok(Item {
                    id: row.get(0)?,
                    text: row.get(1)?,
                    tag_id: row.get(2)?,
                    cadence: row.get(3)?,
                    next: row.get(4)?,
                    pid: Pid {
                        integral: row.get(5)?,
                        last_error: row.get(6)?,
                    },
                })
            })?
            .collect::<rusqlite::Result<Vec<Item>>>()
            .context("could not pull rows")?;

        Ok(items.into_iter())
    }

    pub fn save(&self, conn: &Connection) -> Result<()> {
        conn.execute(
            "UPDATE items SET text = ?, cadence = ?, next = ?, tag_id = ?, integral = ?, last_error = ? WHERE id = ?",
            params![
                self.text,
                self.cadence,
                self.next,
                self.tag_id,
                self.pid.integral,
                self.pid.last_error,
                self.id,
            ]
        ).with_context(|| format!("could not item with ID {}", self.id))?;

        Ok(())
    }

    // as in Cadence, doing a conversion back and forth here is fine because
    // we're not going to be anywhere near the danger zone (52 bits)
    #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
    pub fn bump_cadence(&mut self, bump: &Bump) -> Cadence {
        let adjustment = Cadence::minutes(match bump {
            Bump::MuchEarlier => self.pid.next(Cadence::days(-4).minutes as f64),
            Bump::Earlier => self.pid.next(Cadence::days(-1).minutes as f64),
            Bump::JustRight => self.pid.next(0.0),
            Bump::Later => self.pid.next(Cadence::days(1).minutes as f64),
            Bump::MuchLater => self.pid.next(Cadence::days(4).minutes as f64),
        } as i64);

        log::debug!("adjusting cadence by {:?}", adjustment);
        self.cadence += adjustment;

        adjustment
    }

    pub fn finish(&mut self, bump: &Bump) -> Result<Cadence> {
        let now = Utc::now();

        log::debug!("next: {}, now: {}", self.next, now);
        if self.next > now {
            bail!(
                "can't finish an item before it's due ({})",
                self.next.with_timezone(&Local).format("%A, %B %d, %Y")
            )
        }

        let adjustment = self.bump_cadence(bump);

        self.next = now + self.cadence;

        Ok(adjustment)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn default() -> Item {
        Item {
            id: 1,
            text: "Test".into(),
            tag_id: None,
            cadence: Cadence::days(1),
            next: Utc.ymd(2022, 1, 1).and_hms(0, 0, 0),
            pid: Pid::default(),
        }
    }

    mod bump_cadence {
        use super::*;

        #[test]
        fn much_earlier() {
            let mut small = default();
            let mut large = default();

            let orig = Cadence::months(1);
            small.cadence = orig;
            large.cadence = orig;

            small.bump_cadence(&Bump::Earlier);
            large.bump_cadence(&Bump::MuchEarlier);

            assert!(large.cadence < small.cadence);
        }

        #[test]
        fn earlier() {
            let mut item = default();

            let orig = Cadence::months(1);
            item.cadence = orig;

            item.bump_cadence(&Bump::Earlier);

            assert!(item.cadence < orig);
        }

        #[test]
        fn later() {
            let mut item = default();

            let orig = Cadence::months(1);
            item.cadence = orig;

            item.bump_cadence(&Bump::Later);

            assert!(item.cadence > orig);
        }

        #[test]
        fn much_later() {
            let mut small = default();
            let mut large = default();

            let orig = Cadence::months(1);
            small.cadence = orig;
            large.cadence = orig;

            small.bump_cadence(&Bump::Later);
            large.bump_cadence(&Bump::MuchLater);

            assert!(large.cadence > small.cadence);
        }
    }

    mod finish {
        use super::*;

        #[test]
        fn disallows_tasks_before_next_date() {
            let mut item = default();
            item.next = Utc::now() + item.cadence;

            assert_eq!(
                format!(
                    "can't finish an item before it's due ({})",
                    item.next.with_timezone(&Local).format("%A, %B %d, %Y")
                ),
                item.finish(&Bump::JustRight).unwrap_err().to_string()
            )
        }

        #[test]
        fn moves_into_the_future() {
            let mut item = default();
            item.next = Utc::now() - item.cadence;
            let old_next = item.next;

            item.finish(&Bump::JustRight).unwrap();

            assert!(old_next < item.next)
        }
    }
}
