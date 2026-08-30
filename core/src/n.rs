#![allow(unused)]

#[cfg(any(target_pointer_width = "64", test))]
use crate::common::search_n_swar64;

#[cfg(any(target_pointer_width = "32", test))]
use crate::common::search_n_swar32;

/// Searches for the first occurrence of a needle in a byte slice haystack
///
/// ## Example
///
/// ```
/// use ashwa::search_n;
///
/// let haystack = b"hello world";
/// assert_eq!(search_n(haystack, b"ello"), Some(1));
/// assert_eq!(search_n(haystack, b"world"), Some(6));
/// assert_eq!(search_n(haystack, b"zzz"), None);
/// ```
#[inline(always)]
#[cfg(target_pointer_width = "64")]
pub fn search_n(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    search_n_swar64(haystack, needle)
}

/// Searches for the first occurrence of a needle in a byte slice haystack
///
/// ## Example
///
/// ```
/// use ashwa::search_n;
///
/// let haystack = b"hello world";
/// assert_eq!(search_n(haystack, b"ello"), Some(1));
/// assert_eq!(search_n(haystack, b"world"), Some(6));
/// assert_eq!(search_n(haystack, b"zzz"), None);
/// ```
#[inline(always)]
#[cfg(target_pointer_width = "32")]
pub fn search_n(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    search_n_swar32(haystack, needle)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_standard_suite(search_fn: impl Fn(&[u8], &[u8]) -> Option<usize>) {
        assert_eq!(search_fn(b"", b""), Some(0));
        assert_eq!(search_fn(b"abc", b""), Some(0));

        assert_eq!(search_fn(b"", b"a"), None);
        assert_eq!(search_fn(b"", b"hello"), None);

        assert_eq!(search_fn(b"abc", b"abcd"), None);

        assert_eq!(search_fn(b"abcd", b"abcd"), Some(0));
        assert_eq!(search_fn(b"helloworld", b"helloworld"), Some(0));

        let haystack = b"the quick brown fox jumps over the lazy dog";
        assert_eq!(search_fn(haystack, b"Z"), None);
        assert_eq!(search_fn(haystack, b"!"), None);
        assert_eq!(search_fn(haystack, b"the"), Some(0));
        assert_eq!(search_fn(haystack, b"quick"), Some(4));
        assert_eq!(search_fn(haystack, b"brown"), Some(0x0A));
        assert_eq!(search_fn(haystack, b"fox"), Some(0x10));
        assert_eq!(search_fn(haystack, b"jumps over"), Some(0x14));
        assert_eq!(search_fn(haystack, b"lazy dog"), Some(0x23));
        assert_eq!(search_fn(haystack, b"dog"), Some(0x28));
        assert_eq!(search_fn(haystack, b"dogs"), None);

        for len in 1..8 {
            let mut h = vec![b'x'; len];
            h[len - 1] = b'a';
            assert_eq!(search_fn(&h, b"a"), Some(len - 1));
        }

        for n_len in 1..=0x10 {
            let needle: Vec<u8> = (0..n_len).map(|b| b'A' + (b as u8 % 0x1A)).collect();
            for h_len in n_len..=0x80 {
                let mut h = vec![b'.'; h_len];
                let pos = h_len - n_len;
                h[pos..pos + n_len].copy_from_slice(&needle);
                assert_eq!(
                    search_fn(&h, &needle),
                    Some(pos),
                    "Failed searching needle of len {} in haystack of len {} at pos {}",
                    n_len,
                    h_len,
                    pos
                );
            }
        }

        for n_len in [4, 5, 8, 0x0C, 0x10, 0x20] {
            let needle: Vec<u8> = (0..n_len).map(|b| b'0' + (b as u8 % 0x0A)).collect();
            let mut haystack = vec![b'-'; 0x200];

            for i in 0..=haystack.len() - n_len {
                haystack[i..i + n_len].copy_from_slice(&needle);
                assert_eq!(
                    search_fn(&haystack, &needle),
                    Some(i),
                    "Failed finding needle (len {}) at index {}",
                    n_len,
                    i
                );

                haystack[i..i + n_len].fill(b'-');
            }
        }

        let mut fp_haystack = vec![b'x'; 0x100];
        let needle = b"A___B";
        fp_haystack[0x20] = b'A';
        fp_haystack[0x21] = b'x';
        fp_haystack[0x22] = b'x';
        fp_haystack[0x23] = b'x';
        fp_haystack[0x24] = b'B';
        assert_eq!(search_fn(&fp_haystack, needle), None);

        fp_haystack[0x40..0x45].copy_from_slice(needle);
        assert_eq!(search_fn(&fp_haystack, needle), Some(0x40));

        let mut multi_fp = vec![b'.'; 0x80];
        multi_fp[0x10] = b'A';
        multi_fp[0x11] = b'B';
        multi_fp[0x12] = b'x';
        multi_fp[0x13] = b'Z';

        multi_fp[0x14] = b'A';
        multi_fp[0x15] = b'B';
        multi_fp[0x16] = b'C';
        multi_fp[0x17] = b'Z';

        assert_eq!(search_fn(&multi_fp, b"ABCZ"), Some(0x14));
        assert_eq!(search_fn(b"aaaaaaaa", b"aaaa"), Some(0));
        assert_eq!(search_fn(b"baaaaaaa", b"aaaa"), Some(1));
        assert_eq!(search_fn(b"bbaaaaaa", b"aaaa"), Some(2));
        assert_eq!(search_fn(b"abababab", b"abab"), Some(0));
        assert_eq!(search_fn(b"babababa", b"abab"), Some(1));
        assert_eq!(search_fn(b"bbababab", b"abab"), Some(2));

        let mut haystack_high = vec![0x80; 0x40];
        haystack_high[0x3C] = 0xFE;
        haystack_high[0x3D] = 0xFD;
        haystack_high[0x3E] = 0xFC;
        haystack_high[0x3F] = 0xFB;
        assert_eq!(search_fn(&haystack_high, &[0x7F, 0x80, 0x80, 0x80]), None);
        assert_eq!(search_fn(&haystack_high, &[0xFE, 0xFD, 0xFC, 0xFB]), Some(0x3C));

        let mut haystack_null = vec![0xFF; 0x50];
        haystack_null[0x2A] = 0x00;
        haystack_null[0x2B] = 0x00;
        haystack_null[0x2C] = 0x00;
        haystack_null[0x2D] = 0x00;
        assert_eq!(search_fn(&haystack_null, &[0x00, 0x00, 0x00, 0x00]), Some(0x2A));

        let buffer = [b'-'; 0x80];
        for offset in 1..8 {
            let mut h = buffer[offset..].to_vec();
            h[0x19..0x1D].copy_from_slice(b"WXYZ");
            assert_eq!(search_fn(&h, b"WXYZ"), Some(0x19));

            let end_idx = h.len() - 4;
            h[end_idx..end_idx + 4].copy_from_slice(b"KLMN");
            assert_eq!(search_fn(&h, b"KLMN"), Some(end_idx));
        }

        let tail_lengths = [
            4, 5, 6, 7, 8, 9, 0x0A, 0x0E, 0x0F, 0x10, 0x11, 0x12, 0x1F, 0x20, 0x21, 0x22, 0x2F,
            0x30, 0x31, 0x3F, 0x40, 0x41, 0x7F, 0x80, 0x81, 0xFF, 0x100, 0x101,
        ];
        for &len in &tail_lengths {
            let mut h = vec![b'-'; len];
            h[len - 4..len].copy_from_slice(b"ABCD");
            assert_eq!(
                search_fn(&h, b"ABCD"),
                Some(len - 4),
                "Failed tail chunk fallback for length {}",
                len
            );
        }

        let mut huge_haystack = vec![b'x'; 0x64 * 0x400];
        assert_eq!(search_fn(&huge_haystack, b"WXYZ1234"), None);

        let last_pos = 0x64 * 0x400 - 8;
        huge_haystack[last_pos..last_pos + 8].copy_from_slice(b"WXYZ1234");
        assert_eq!(search_fn(&huge_haystack, b"WXYZ1234"), Some(last_pos));
    }

    #[test]
    fn test_public_api() {
        run_standard_suite(search_n);
    }

    #[test]
    #[cfg(any(target_pointer_width = "64", test))]
    fn test_swar64_directly() {
        run_standard_suite(search_n_swar64);
    }

    #[test]
    #[cfg(any(target_pointer_width = "32", test))]
    fn test_swar32_directly() {
        run_standard_suite(search_n_swar32);
    }

    #[test]
    fn test_differential_randomized() {
        let mut rng = 0x123456789ABCDEF0u64;
        let mut next_rand = || {
            rng ^= rng << 0x0D;
            rng ^= rng >> 7;
            rng ^= rng << 0x11;
            rng
        };

        let search_fns: [fn(&[u8], &[u8]) -> Option<usize>; 2] = [search_n_swar64, search_n_swar32];

        for s_fn in search_fns {
            for _ in 0..0x3E8 {
                let h_len = (next_rand() % 0x12C) as usize;
                let n_len =
                    if h_len == 0 { 0 } else { (next_rand() % (h_len as u64 + 0x0A)) as usize };

                let alphabet_size = match next_rand() % 4 {
                    0 => 2,
                    1 => 4,
                    2 => 0x1A,
                    _ => 0x100,
                };

                let mut haystack = vec![0u8; h_len];
                for b in haystack.iter_mut() {
                    *b = (next_rand() % alphabet_size) as u8;
                }

                let mut needle = vec![0u8; n_len];
                for b in needle.iter_mut() {
                    *b = (next_rand() % alphabet_size) as u8;
                }

                if n_len > 0 && n_len <= h_len && (next_rand() % 2 == 0) {
                    let insert_pos = (next_rand() % (h_len - n_len + 1) as u64) as usize;
                    haystack[insert_pos..insert_pos + n_len].copy_from_slice(&needle);
                }

                let expected = if needle.is_empty() {
                    Some(0)
                } else if needle.len() > haystack.len() {
                    None
                } else {
                    haystack.windows(needle.len()).position(|w| w == needle.as_slice())
                };

                let actual = s_fn(&haystack, &needle);
                assert_eq!(
                    actual, expected,
                    "Differential mismatch: expected {:?}, got {:?}, h_len={}, n_len={}, alphabet_size={}",
                    expected, actual, h_len, n_len, alphabet_size
                );
            }
        }
    }
}
