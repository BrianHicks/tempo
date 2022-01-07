use barrel::backend::Sqlite;
use barrel::{types, Migration};

pub fn migration() -> String {
    let mut m = Migration::new();

    m.change_table("items", |t| {
        t.add_column("last", types::datetime().nullable(true));
    });

    m.make::<Sqlite>()
}
