use similar::TextDiff;

pub fn unified_diff(from: &str, to: &str, from_name: &str, to_name: &str) -> String {
    TextDiff::from_lines(from, to)
        .unified_diff()
        .header(from_name, to_name)
        .to_string()
}
