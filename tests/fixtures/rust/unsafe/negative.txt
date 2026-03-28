fn safe_access(values: &[u8], index: usize) -> Option<u8> {
    values.get(index).copied()
}

fn initialized_value() -> String {
    String::from("ready")
}