use anyhow::{Context, Result};
use rusqlite::Connection;

#[derive(Debug)]
pub struct Tag {
    pub id: u64,
    pub name: String,
}

impl Tag {
    pub fn get_or_create(conn: &Connection, name: &str) -> Result<Tag> {
        conn.query_row(
            // We use `DO UPDATE SET` for upsert here because `DO
            // NOTHING` makes the query fail to return the ID in the
            // RETURNING clause.
            "INSERT INTO tags (name) VALUES (?1) ON CONFLICT DO UPDATE SET name = ?1 RETURNING id, name",
            [name],
            |row| Ok(Tag { id: row.get(0)?, name: row.get(1)? }),
        )
        .with_context(|| format!("could not get or insert the \"{}\" tag", name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn conn() -> Connection {
        let mut conn = Connection::open_in_memory().expect("couldn't open an in-memory database");
        crate::db::migrations::runner()
            .run(&mut conn)
            .expect("couldn't migrate database");

        conn
    }

    #[test]
    fn inserts_new() {
        let conn = conn();
        let tag_name = "tag".to_string();

        let tag = Tag::get_or_create(&conn, &tag_name).unwrap();

        assert_eq!(tag_name, tag.name);
        assert_eq!(1, tag.id);
    }

    #[test]
    fn uses_existing() {
        let tag = "tag".to_string();

        let conn = conn();

        let tag_id: u64 = conn
            .query_row(
                "INSERT INTO tags (name) VALUES (?) RETURNING id",
                [&tag],
                |row| row.get(0),
            )
            .expect("failed to insert a new tag");

        assert_eq!(tag_id, Tag::get_or_create(&conn, &tag).unwrap().id);
    }
}
