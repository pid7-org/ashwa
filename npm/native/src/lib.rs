//! NAPI-RS bindings for `ashwa`

use napi_derive::napi;

/// Searches for the first occurrence of `needle` in `haystack`.
///
/// # Arguments
/// * `haystack` - A byte slice (`&[u8]`) to search within.
/// * `needle` - The byte (`u8`) to find.
///
/// # Returns
/// * `Some(index)` - The 0-based byte index of the first match.
/// * `None` - If `needle` is not found.
#[napi(js_name = "searchOne")]
pub fn search_one(haystack: &[u8], needle: u8) -> Option<i64> {
    ashwa::search_one(haystack, needle).map(|i| i as i64)
}

/// Searches for the first occurrence of a two-byte `needle` in `haystack`.
///
/// # Arguments
/// * `haystack` - A byte slice (`&[u8]`) to search within.
/// * `needle` - A 2-byte slice (`&[u8]`) to locate.
///
/// # Returns
/// * `Some(index)` - The 0-based byte index of the first match.
/// * `None` - If `needle` is not found.
#[napi(js_name = "searchTwo")]
pub fn search_two(haystack: &[u8], needle: &[u8]) -> Option<i64> {
    if needle.len() != 2 {
        return None;
    }

    ashwa::search_two(haystack, [needle[0], needle[1]]).map(|i| i as i64)
}
