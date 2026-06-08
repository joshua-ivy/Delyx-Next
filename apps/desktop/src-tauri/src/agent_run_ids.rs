pub(crate) fn numeric_suffix(value: &str, prefix: &str) -> Option<usize> {
    value.strip_prefix(prefix)?.parse().ok()
}
