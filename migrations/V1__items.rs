use barrel::backend::Sqlite;
use barrel::{types, Migration};

pub fn migration() -> String {
    let mut m = Migration::new();

    m.create_table("items", |t| {
        t.add_column("id", types::primary());
        t.add_column("text", types::text());

        // scheduling
        t.add_column("cadence", types::integer().default(1440));
        t.add_column("next", types::datetime());

        // PID
        t.add_column("integral", types::float().default(0.0));
        t.add_column("last_error", types::float().default(0.0));
    });

    m.make::<Sqlite>()
}
