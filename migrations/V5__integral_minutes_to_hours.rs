pub fn migration() -> String {
    "UPDATE items SET integral = integral / 1440;".to_string()
}
