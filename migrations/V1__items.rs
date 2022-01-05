use barrel::backend::Sqlite;
use barrel::{types, Migration};

pub fn migration() -> String {
    let mut m = Migration::new();

    m.create_table("items", |t| {
        t.add_column("id", types::primary());
        t.add_column("text", types::text());
        t.add_column("cadence", types::integer());
        t.add_column("next", types::datetime());
    });

    m.make::<Sqlite>()
}
