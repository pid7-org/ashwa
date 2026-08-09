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

