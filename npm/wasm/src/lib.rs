//! `ashwa_wasm` - WebAssembly bindings for `ashwa`.
//!
//! Provides WebAssembly (`simd128`) SIMD-accelerated search routines for browsers,
//! Web Workers, and JS runtimes via `wasm-bindgen`.

use wasm_bindgen::prelude::*;

/// Searches for the first occurrence of `needle` in `haystack`.
///
/// # Arguments
/// * `haystack` - A byte slice (`&[u8]`) to search within.
/// * `needle` - The byte (`u8`) to locate.
///
/// # Returns
/// * `Some(index)` - The 0-based byte index of the first match.
/// * `None` - If `needle` is not found.
#[wasm_bindgen(js_name = "searchOne")]
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
#[wasm_bindgen(js_name = "searchTwo")]
pub fn search_two(haystack: &[u8], needle: &[u8]) -> Option<i64> {
    if needle.len() != 2 {
        return None;
    }
    ashwa::search_two(haystack, [needle[0], needle[1]]).map(|i| i as i64)
}

/// Searches for the first occurrence of a three-byte `needle` in `haystack`.
///
/// # Arguments
/// * `haystack` - A byte slice (`&[u8]`) to search within.
/// * `needle` - A 3-byte slice (`&[u8]`) to locate.
///
/// # Returns
/// * `Some(index)` - The 0-based byte index of the first match.
/// * `None` - If `needle` is not found.
#[wasm_bindgen(js_name = "searchThree")]
pub fn search_three(haystack: &[u8], needle: &[u8]) -> Option<i64> {
    if needle.len() != 3 {
        return None;
    }
    ashwa::search_three(haystack, [needle[0], needle[1], needle[2]]).map(|i| i as i64)
}


