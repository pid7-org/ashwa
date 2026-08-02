use core::ptr;

const LSB: u64 = 0x0101_0101_0101_0101;
const MSB: u64 = 0x8080_8080_8080_8080;

#[inline(always)]
#[cfg(all(target_pointer_width = "64", target_endian = "little"))]
pub fn search_one(haystack: &[u8], needle: u8) -> Option<usize> {
    let needle_word = (needle as u64).wrapping_mul(LSB);

    let mut i = 0;
    let len = haystack.len();
    let ptr = haystack.as_ptr();

    while i + 0x20 <= len {
        let w1 = unsafe { ptr::read_unaligned(ptr.add(i) as *const u64) };
        let m1 = match_64(w1, needle_word);

        let w2 = unsafe { ptr::read_unaligned(ptr.add(i + 8) as *const u64) };
        let m2 = match_64(w2, needle_word);

        let w3 = unsafe { ptr::read_unaligned(ptr.add(i + 0x10) as *const u64) };
        let m3 = match_64(w3, needle_word);

        let w4 = unsafe { ptr::read_unaligned(ptr.add(i + 0x18) as *const u64) };
        let m4 = match_64(w4, needle_word);

        if (m1 | m2 | m3 | m4) != 0 {
            if m1 != 0 {
                return Some(i + (m1.trailing_zeros() / 8) as usize);
            }

            if m2 != 0 {
                return Some(i + 8 + (m2.trailing_zeros() / 8) as usize);
            }

            if m3 != 0 {
                return Some(i + 0x10 + (m3.trailing_zeros() / 8) as usize);
            }

            return Some(i + 0x18 + (m4.trailing_zeros() / 8) as usize);
        }

        i += 0x20;
    }

    if i + 0x10 <= len {
        let w1 = unsafe { ptr::read_unaligned(ptr.add(i) as *const u64) };
        let m1 = match_64(w1, needle_word);

        let w2 = unsafe { ptr::read_unaligned(ptr.add(i + 8) as *const u64) };
        let m2 = match_64(w2, needle_word);

        if (m1 | m2) != 0 {
            if m1 != 0 {
                return Some(i + (m1.trailing_zeros() / 8) as usize);
            }

            return Some(i + 8 + (m2.trailing_zeros() / 8) as usize);
        }

        i += 0x10;
    }

    if i + 8 <= len {
        let w1 = unsafe { ptr::read_unaligned(ptr.add(i) as *const u64) };
        let m1 = match_64(w1, needle_word);

        if m1 != 0 {
            return Some(i + (m1.trailing_zeros() / 8) as usize);
        }

        i += 8;
    }

    haystack[i..].iter().position(|&b| b == needle).map(|pos| pos + i)
}

#[inline]
fn match_64(haystack_word: u64, needle_word: u64) -> u64 {
    let x = haystack_word ^ needle_word;
    let m = x.wrapping_sub(LSB) & !x & MSB;

    m
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ok_empty_haystack() {
        assert_eq!(search_one(b"", b'a'), None);
    }

    #[test]
    fn ok_needle_not_found() {
        let haystack = b"the quick brown fox jumps over the lazy dog";

        assert_eq!(search_one(haystack, b'Z'), None);
        assert_eq!(search_one(haystack, b'!'), None);
    }

    #[test]
    fn ok_iter_fallback_for_len_1to8() {
        for len in 1..8 {
            let mut haystack = vec![b'x'; len];
            haystack[len - 1] = b'a';

            assert_eq!(search_one(&haystack, b'a'), Some(len - 1));
        }
    }

    #[test]
    fn ok_exact_chunk_boundaries() {
        let mut h8 = [b'-'; 8];
        let mut h16 = [b'-'; 0x10];
        let mut h24 = [b'-'; 0x18];
        let mut h32 = [b'-'; 0x20];

        h8[7] = b'A';
        assert_eq!(search_one(&h8, b'A'), Some(7));

        h16[0x0F] = b'B';
        assert_eq!(search_one(&h16, b'B'), Some(0x0F));

        h24[0x17] = b'C';
        assert_eq!(search_one(&h24, b'C'), Some(0x17));

        h32[0x1F] = b'D';
        assert_eq!(search_one(&h32, b'D'), Some(0x1F));
    }

    #[test]
    fn ok_exhaustive_positions() {
        let mut haystack = vec![b'-'; 0x200];

        for i in 0..haystack.len() {
            haystack[i] = b'A';

            assert_eq!(
                search_one(&haystack, b'A'),
                Some(i),
                "Failed finding needle at index {}",
                i
            );

            haystack[i] = b'-';
        }
    }

    #[test]
    fn ok_multiple_occurrences() {
        let haystack = b"hello world, hello rust";
        assert_eq!(search_one(haystack, b'o'), Some(4));
    }

    #[test]
    fn ok_high_bit_characters() {
        let mut haystack = vec![0x80; 0x40];
        haystack[0x3F] = 0xFF;

        assert_eq!(search_one(&haystack, 0x7F), None);
        assert_eq!(search_one(&haystack, 0xFF), Some(0x3F));
    }

    #[test]
    fn ok_null_byte_search() {
        let mut haystack = vec![0xFF; 0x50];
        haystack[0x2A] = 0x00;

        assert_eq!(search_one(&haystack, 0x00), Some(0x2A));
    }

    #[test]
    fn ok_unaligned_slice_offsets() {
        let buffer = vec![b'-'; 0x60];

        for offset in 1..8 {
            let mut haystack = buffer[offset..].to_vec();

            haystack[0x19] = b'Z';
            assert_eq!(
                search_one(&haystack, b'Z'),
                Some(0x19),
                "Failed at slice offset {}",
                offset
            );

            let end_idx = haystack.len() - 2;
            haystack[end_idx] = b'Y';
            assert_eq!(
                search_one(&haystack, b'Y'),
                Some(end_idx),
                "Failed at slice offset {} near the end",
                offset
            );
        }
    }

    #[test]
    fn ok_needle_is_lsb_msb_masks() {
        let haystack_lsb = b"\x01\x01\x01\x01\x01\x01\x01\x01";
        assert_eq!(search_one(haystack_lsb, 0x01), Some(0));

        let haystack_msb = b"\x80\x80\x80\x80\x80\x80\x80\x80";
        assert_eq!(search_one(haystack_msb, 0x80), Some(0));
    }
}
