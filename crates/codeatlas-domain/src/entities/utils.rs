/// Allocation-free case-insensitive substring matcher for ASCII strings.
/// Useful for avoiding `.to_lowercase().contains()` which allocates memory.
/// If strings are non-ASCII, falls back to `to_lowercase()`.
#[inline]
pub fn contains_ignore_case(haystack: &str, needle: &str) -> bool {
    if needle.is_ascii() && haystack.is_ascii() {
        if needle.len() > haystack.len() {
            return false;
        }
        haystack
            .as_bytes()
            .windows(needle.len())
            .any(|w| w.eq_ignore_ascii_case(needle.as_bytes()))
    } else {
        haystack.to_lowercase().contains(&needle.to_lowercase())
    }
}
