use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = "searchOne")]
pub fn search_one(haystack: &[u8], needle: u8) -> Option<i64> {
    ashwa::search_one(haystack, needle).map(|i| i as i64)
}
