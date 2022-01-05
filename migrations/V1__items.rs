use barrel::backend::Sqlite;
use barrel::{types, Migration};

pub fn migration() -> String {
    let mut m = Migration::new();

    m.create_table("items", |t| {
        t.add_column("id", types::primary());
        t.add_column("text", types::text());

        // scheduling
        t.add_column("cadence", types::integer());
        t.add_column("next", types::datetime());

        // PID
        t.add_column("proportional_factor", types::float().default(1.5));
        t.add_column("integral", types::float().default(0.0));
        t.add_column("integral_factor", types::float().default(0.3));
        t.add_column("last_error", types::float().default(0.0));
        t.add_column("derivative_factor", types::float().default(0.1));
    });

    m.make::<Sqlite>()
}
