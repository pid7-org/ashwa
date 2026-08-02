use ashwa::search_one;
use std::hint::black_box;

fn main() {
    let needle_not_found = b'z';
    let mut haystack = vec![b'a'; 0x10 * 0x400];

    haystack[8 * 0x400] = b'b';
    let needle_middle = b'b';

    haystack[0x10 * 0x400 - 1] = b'c';
    let needle_end = b'c';

    let iterations = 0x100_000;
    for _ in 0..iterations {
        black_box(search_one(black_box(&haystack), black_box(needle_not_found)));
        black_box(search_one(black_box(&haystack), black_box(needle_middle)));
        black_box(search_one(black_box(&haystack), black_box(needle_end)));
    }
}
