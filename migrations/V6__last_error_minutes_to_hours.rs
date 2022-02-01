pub fn migration() -> String {
    "UPDATE items SET last_error = last_error / 1440;".to_string()
}
