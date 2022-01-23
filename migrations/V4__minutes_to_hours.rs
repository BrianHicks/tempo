pub fn migration() -> String {
    "UPDATE items SET cadence = cadence / 1440;".to_string()
}
