use barrel::backend::Sqlite;
use barrel::{types, Migration};

pub fn migration() -> String {
    let mut m = Migration::new();

    m.change_table("items", |t| {
        t.inject_custom("ADD COLUMN \"tag_id\" INTEGER REFERENCES tags (id)");
    });

    log::debug!("{}", m.make::<Sqlite>());
    m.make::<Sqlite>()
}
