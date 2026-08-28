#![cfg(target_arch = "wasm32")]

use ashwa_wasm::{search_one, search_two};
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_search_one_browser_simd128() {
    let haystack = b"Hello, World! WebAssembly SIMD128 Headless Browser Test.";
    assert_eq!(search_one(haystack, b'W'), Some(7));
    assert_eq!(search_one(haystack, b'Z'), None);
}

#[wasm_bindgen_test]
fn test_search_two_browser_simd128() {
    let haystack = b"Hello, World! WebAssembly SIMD128 Headless Browser Test.";
    assert_eq!(search_two(haystack, b"Wo"), Some(7));
    assert_eq!(search_two(haystack, b"ZZ"), None);
    assert_eq!(search_two(haystack, b"H"), None);
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

#[wasm_bindgen_test]
fn test_search_two_simd128_boundaries_browser() {
    let sizes = [2, 3, 7, 8, 15, 16, 17, 31, 32, 33, 63, 64, 65, 127, 128, 255, 256];

    for size in sizes {
        let mut buf = vec![0xaa; size];

        buf[0] = 0xbb;
        buf[1] = 0xcc;
        assert_eq!(search_two(&buf, &[0xbb, 0xcc]), Some(0));

        buf[0] = 0xaa;
        buf[1] = 0xaa;
        buf[size - 2] = 0xbb;
        buf[size - 1] = 0xcc;
        assert_eq!(search_two(&buf, &[0xbb, 0xcc]), Some((size - 2) as i64));

        if size > 3 {
            let mid = size / 2;
            buf[size - 2] = 0xaa;
            buf[size - 1] = 0xaa;
            buf[mid] = 0xbb;
            buf[mid + 1] = 0xcc;
            assert_eq!(search_two(&buf, &[0xbb, 0xcc]), Some(mid as i64));
        }
    }
}
