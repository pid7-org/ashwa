//! `ashwa_node` - NAPI-RS bindings for `ashwa`.
//!
//! Provides high-performance, native C++ addon bindings exposing hardware-accelerated
//! search routines to Node.js, Bun, and Deno.

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

