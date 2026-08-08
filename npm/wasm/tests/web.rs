#![cfg(target_arch = "wasm32")]

use ashwa_wasm::search_one;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_search_one_browser_simd128() {
    let haystack = b"Hello, World! WebAssembly SIMD128 Headless Browser Test.";
    assert_eq!(search_one(haystack, b'W'), Some(7));
    assert_eq!(search_one(haystack, b'Z'), None);
}

#[wasm_bindgen_test]
fn test_simd128_boundaries_browser() {
    let sizes = [1, 2, 7, 8, 15, 16, 17, 31, 32, 33, 63, 64, 65, 127, 128, 255, 256];

    for size in sizes {
        let mut buf = vec![0xaa; size];

        buf[0] = 0xbb;
        assert_eq!(search_one(&buf, 0xbb), Some(0));

        buf[0] = 0xaa;
        buf[size - 1] = 0xbb;
        assert_eq!(search_one(&buf, 0xbb), Some((size - 1) as i64));

        if size > 2 {
            let mid = size / 2;
            buf[size - 1] = 0xaa;
            buf[mid] = 0xbb;
            assert_eq!(search_one(&buf, 0xbb), Some(mid as i64));
        }
    }
}
