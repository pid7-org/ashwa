use crate::{search_n, search_one, search_three, search_two};

/// Searches for the first occurrence of a needle in a byte slice haystack.
///
/// Automatically dispatches to hardware-accelerated specialized search routines
/// based on the length of the needle:
/// - Needle length 0: Returns `Some(0)`
/// - Needle length 1: Dispatches to [`search_one`]
/// - Needle length 2: Dispatches to [`search_two`]
/// - Needle length 3: Dispatches to [`search_three`]
/// - Needle length $\ge 4$: Dispatches to [`search_n`]
///
/// ## Example
///
/// ```
/// use ashwa::search;
///
/// let haystack = b"The quick brown fox jumps over the lazy dog";
/// assert_eq!(search(haystack, b""), Some(0));
/// assert_eq!(search(haystack, b"f"), Some(0x10));
/// assert_eq!(search(haystack, b"qu"), Some(0x04));
/// assert_eq!(search(haystack, b"fox"), Some(0x10));
/// assert_eq!(search(haystack, b"lazy dog"), Some(0x23));
/// assert_eq!(search(haystack, b"not found"), None);
/// ```
#[inline(always)]
pub fn search(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    let n = needle.len();
    if n == 0 {
        return Some(0);
    }
    if n > haystack.len() {
        return None;
    }
    match n {
        1 => search_one(haystack, needle[0]),
        2 => search_two(haystack, [needle[0], needle[1]]),
        3 => search_three(haystack, [needle[0], needle[1], needle[2]]),
        _ => search_n(haystack, needle),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_unified() {
        let haystack = b"the quick brown fox jumps over the lazy dog";

        // Empty needle
        assert_eq!(search(haystack, b""), Some(0));
        assert_eq!(search(b"", b""), Some(0));
        assert_eq!(search(b"", b"a"), None);

        // Single byte
        assert_eq!(search(haystack, b"t"), Some(0));
        assert_eq!(search(haystack, b"h"), Some(1));
        assert_eq!(search(haystack, b"o"), Some(12));
        assert_eq!(search(haystack, b"g"), Some(42));
        assert_eq!(search(haystack, b"z"), Some(37));
        assert_eq!(search(haystack, b"!"), None);

        // Two bytes
        assert_eq!(search(haystack, b"th"), Some(0));
        assert_eq!(search(haystack, b"he"), Some(1));
        assert_eq!(search(haystack, b"qu"), Some(4));
        assert_eq!(search(haystack, b"ox"), Some(17));
        assert_eq!(search(haystack, b"og"), Some(41));
        assert_eq!(search(haystack, b"!!"), None);

        // Three bytes
        assert_eq!(search(haystack, b"the"), Some(0));
        assert_eq!(search(haystack, b"qui"), Some(4));
        assert_eq!(search(haystack, b"fox"), Some(16));
        assert_eq!(search(haystack, b"dog"), Some(40));
        assert_eq!(search(haystack, b"!!!"), None);

        // N bytes (>= 4)
        assert_eq!(search(haystack, b"quick"), Some(4));
        assert_eq!(search(haystack, b"brown fox"), Some(10));
        assert_eq!(search(haystack, b"lazy dog"), Some(35));
        assert_eq!(search(haystack, b"the quick brown fox jumps over the lazy dog"), Some(0));
        assert_eq!(search(haystack, b"the quick brown fox jumps over the lazy dog!"), None);
        assert_eq!(search(haystack, b"not present"), None);
    }
}
