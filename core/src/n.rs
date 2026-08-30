#![allow(unused)]

use core::ptr;

#[cfg(target_arch = "x86_64")]
use crate::{get_cpu_feature, ISA};

#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

#[cfg(target_arch = "x86")]
use core::arch::x86::*;

#[cfg(target_arch = "wasm32")]
use core::arch::wasm32::*;

#[cfg(target_arch = "aarch64")]
use core::arch::aarch64::*;

#[cfg(all(target_arch = "arm", target_feature = "neon"))]
use core::arch::arm::*;

#[cfg(any(target_pointer_width = "64", test))]
use crate::common::search_n_swar64;

#[cfg(any(target_pointer_width = "32", test))]
use crate::common::search_n_swar32;

#[cfg(any(
    target_arch = "aarch64",
    all(target_arch = "arm", target_feature = "neon")
))]
use crate::common::clear_lowest_match_64;
use crate::{search_one, search_three, search_two};

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
    #[cfg(target_arch = "x86_64")]
    match get_cpu_feature() {
        ISA::AVX512BW => unsafe { search_n_avx512(haystack, needle) },
        ISA::AVX2 => unsafe { search_n_avx2(haystack, needle) },
        ISA::SSE4_2 => unsafe { search_n_sse42(haystack, needle) },
        ISA::SSSE3 => unsafe { search_n_ssse3(haystack, needle) },
        ISA::SSE2 => unsafe { search_n_sse2(haystack, needle) },
        _ => search_n_swar64(haystack, needle),
    }

    #[cfg(target_arch = "aarch64")]
    {
        #[cfg(forced_swar_backend)]
        return search_n_swar64(haystack, needle);

        unsafe { search_n_neon(haystack, needle) }
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    {
        search_n_swar64(haystack, needle)
    }
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
    #[cfg(target_arch = "wasm32")]
    {
        #[cfg(target_feature = "simd128")]
        return unsafe { search_n_simd128(haystack, needle) };

        search_n_swar32(haystack, needle)
    }

    #[cfg(target_arch = "arm")]
    {
        #[cfg(target_feature = "neon")]
        return unsafe { search_n_neon(haystack, needle) };

        search_n_swar32(haystack, needle)
    }

    #[cfg(target_arch = "x86")]
    {
        #[cfg(target_feature = "sse4.2")]
        return unsafe { search_n_sse42(haystack, needle) };

        #[cfg(target_feature = "ssse3")]
        return unsafe { search_n_ssse3(haystack, needle) };

        #[cfg(target_feature = "sse2")]
        return unsafe { search_n_sse2(haystack, needle) };

        search_n_swar32(haystack, needle)
    }

    #[cfg(not(any(target_arch = "wasm32", target_arch = "arm", target_arch = "x86")))]
    {
        search_n_swar32(haystack, needle)
    }
}

#[target_feature(enable = "sse2")]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
unsafe fn search_n_sse2(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    let n = needle.len();
    if n == 0 {
        return Some(0);
    }

    if n > haystack.len() {
        return None;
    }

    if n == 1 {
        return search_one(haystack, needle[0]);
    }

    if n == 2 {
        return search_two(haystack, [needle[0], needle[1]]);
    }

    if n == 3 {
        return search_three(haystack, [needle[0], needle[1], needle[2]]);
    }

    let v_needle_a = _mm_set1_epi8(needle[0] as i8);
    let v_needle_b = _mm_set1_epi8(needle[1] as i8);
    let v_needle_c = _mm_set1_epi8(needle[n - 1] as i8);

    let len = haystack.len();
    let ptr = haystack.as_ptr();

    let mut i = 0;
    while i + 0x40 + n - 1 <= len {
        let v1_a = _mm_loadu_si128(ptr.add(i) as *const __m128i);
        let v1_b = _mm_loadu_si128(ptr.add(i + 1) as *const __m128i);
        let v1_c = _mm_loadu_si128(ptr.add(i + n - 1) as *const __m128i);

        let v2_a = _mm_loadu_si128(ptr.add(i + 0x10) as *const __m128i);
        let v2_b = _mm_loadu_si128(ptr.add(i + 0x11) as *const __m128i);
        let v2_c = _mm_loadu_si128(ptr.add(i + 0x10 + n - 1) as *const __m128i);

        let v3_a = _mm_loadu_si128(ptr.add(i + 0x20) as *const __m128i);
        let v3_b = _mm_loadu_si128(ptr.add(i + 0x21) as *const __m128i);
        let v3_c = _mm_loadu_si128(ptr.add(i + 0x20 + n - 1) as *const __m128i);

        let v4_a = _mm_loadu_si128(ptr.add(i + 0x30) as *const __m128i);
        let v4_b = _mm_loadu_si128(ptr.add(i + 0x31) as *const __m128i);
        let v4_c = _mm_loadu_si128(ptr.add(i + 0x30 + n - 1) as *const __m128i);

        let eq1 = _mm_and_si128(
            _mm_and_si128(_mm_cmpeq_epi8(v1_a, v_needle_a), _mm_cmpeq_epi8(v1_b, v_needle_b)),
            _mm_cmpeq_epi8(v1_c, v_needle_c),
        );
        let eq2 = _mm_and_si128(
            _mm_and_si128(_mm_cmpeq_epi8(v2_a, v_needle_a), _mm_cmpeq_epi8(v2_b, v_needle_b)),
            _mm_cmpeq_epi8(v2_c, v_needle_c),
        );
        let eq3 = _mm_and_si128(
            _mm_and_si128(_mm_cmpeq_epi8(v3_a, v_needle_a), _mm_cmpeq_epi8(v3_b, v_needle_b)),
            _mm_cmpeq_epi8(v3_c, v_needle_c),
        );
        let eq4 = _mm_and_si128(
            _mm_and_si128(_mm_cmpeq_epi8(v4_a, v_needle_a), _mm_cmpeq_epi8(v4_b, v_needle_b)),
            _mm_cmpeq_epi8(v4_c, v_needle_c),
        );

        let or1 = _mm_or_si128(eq1, eq2);
        let or2 = _mm_or_si128(eq3, eq4);
        let or_vec = _mm_or_si128(or1, or2);

        if _mm_movemask_epi8(or_vec) != 0 {
            let mut m1 = _mm_movemask_epi8(eq1) as u32;
            while m1 != 0 {
                let offset = m1.trailing_zeros() as usize;
                let cand = i + offset;
                if &haystack[cand..cand + n] == needle {
                    return Some(cand);
                }
                m1 &= m1 - 1;
            }

            let mut m2 = _mm_movemask_epi8(eq2) as u32;
            while m2 != 0 {
                let offset = m2.trailing_zeros() as usize;
                let cand = i + 0x10 + offset;
                if &haystack[cand..cand + n] == needle {
                    return Some(cand);
                }
                m2 &= m2 - 1;
            }

            let mut m3 = _mm_movemask_epi8(eq3) as u32;
            while m3 != 0 {
                let offset = m3.trailing_zeros() as usize;
                let cand = i + 0x20 + offset;
                if &haystack[cand..cand + n] == needle {
                    return Some(cand);
                }
                m3 &= m3 - 1;
            }

            let mut m4 = _mm_movemask_epi8(eq4) as u32;
            while m4 != 0 {
                let offset = m4.trailing_zeros() as usize;
                let cand = i + 0x30 + offset;
                if &haystack[cand..cand + n] == needle {
                    return Some(cand);
                }
                m4 &= m4 - 1;
            }
        }

        i += 0x40;
    }

    while i + 0x10 + n - 1 <= len {
        let v_a = _mm_loadu_si128(ptr.add(i) as *const __m128i);
        let v_b = _mm_loadu_si128(ptr.add(i + 1) as *const __m128i);
        let v_c = _mm_loadu_si128(ptr.add(i + n - 1) as *const __m128i);

        let eq = _mm_and_si128(
            _mm_and_si128(_mm_cmpeq_epi8(v_a, v_needle_a), _mm_cmpeq_epi8(v_b, v_needle_b)),
            _mm_cmpeq_epi8(v_c, v_needle_c),
        );
        let mut m = _mm_movemask_epi8(eq) as u32;

        while m != 0 {
            let offset = m.trailing_zeros() as usize;
            let cand = i + offset;
            if &haystack[cand..cand + n] == needle {
                return Some(cand);
            }
            m &= m - 1;
        }

        i += 0x10;
    }

    haystack[i..].windows(n).position(|w| w == needle).map(|pos| pos + i)
}

#[target_feature(enable = "ssse3")]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
unsafe fn search_n_ssse3(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    let n = needle.len();
    if n == 0 {
        return Some(0);
    }

    if n > haystack.len() {
        return None;
    }

    if n == 1 {
        return search_one(haystack, needle[0]);
    }

    if n == 2 {
        return search_two(haystack, [needle[0], needle[1]]);
    }

    if n == 3 {
        return search_three(haystack, [needle[0], needle[1], needle[2]]);
    }

    let v_needle_a = _mm_set1_epi8(needle[0] as i8);
    let v_needle_b = _mm_set1_epi8(needle[1] as i8);
    let v_needle_c = _mm_set1_epi8(needle[n - 1] as i8);

    let len = haystack.len();
    let ptr = haystack.as_ptr();

    let mut i = 0;
    while i + 0x40 + n - 1 <= len {
        let v1 = _mm_loadu_si128(ptr.add(i) as *const __m128i);
        let v2 = _mm_loadu_si128(ptr.add(i + 0x10) as *const __m128i);
        let v3 = _mm_loadu_si128(ptr.add(i + 0x20) as *const __m128i);
        let v4 = _mm_loadu_si128(ptr.add(i + 0x30) as *const __m128i);
        let v5 = _mm_loadu_si128(ptr.add(i + 0x40) as *const __m128i);

        let v1_b = _mm_alignr_epi8(v2, v1, 1);
        let v2_b = _mm_alignr_epi8(v3, v2, 1);
        let v3_b = _mm_alignr_epi8(v4, v3, 1);
        let v4_b = _mm_alignr_epi8(v5, v4, 1);

        let v1_c = _mm_loadu_si128(ptr.add(i + n - 1) as *const __m128i);
        let v2_c = _mm_loadu_si128(ptr.add(i + 0x10 + n - 1) as *const __m128i);
        let v3_c = _mm_loadu_si128(ptr.add(i + 0x20 + n - 1) as *const __m128i);
        let v4_c = _mm_loadu_si128(ptr.add(i + 0x30 + n - 1) as *const __m128i);

        let eq1 = _mm_and_si128(
            _mm_and_si128(_mm_cmpeq_epi8(v1, v_needle_a), _mm_cmpeq_epi8(v1_b, v_needle_b)),
            _mm_cmpeq_epi8(v1_c, v_needle_c),
        );
        let eq2 = _mm_and_si128(
            _mm_and_si128(_mm_cmpeq_epi8(v2, v_needle_a), _mm_cmpeq_epi8(v2_b, v_needle_b)),
            _mm_cmpeq_epi8(v2_c, v_needle_c),
        );
        let eq3 = _mm_and_si128(
            _mm_and_si128(_mm_cmpeq_epi8(v3, v_needle_a), _mm_cmpeq_epi8(v3_b, v_needle_b)),
            _mm_cmpeq_epi8(v3_c, v_needle_c),
        );
        let eq4 = _mm_and_si128(
            _mm_and_si128(_mm_cmpeq_epi8(v4, v_needle_a), _mm_cmpeq_epi8(v4_b, v_needle_b)),
            _mm_cmpeq_epi8(v4_c, v_needle_c),
        );

        let or1 = _mm_or_si128(eq1, eq2);
        let or2 = _mm_or_si128(eq3, eq4);
        let or_vec = _mm_or_si128(or1, or2);

        if _mm_movemask_epi8(or_vec) != 0 {
            let mut m1 = _mm_movemask_epi8(eq1) as u32;
            while m1 != 0 {
                let offset = m1.trailing_zeros() as usize;
                let cand = i + offset;
                if &haystack[cand..cand + n] == needle {
                    return Some(cand);
                }
                m1 &= m1 - 1;
            }

            let mut m2 = _mm_movemask_epi8(eq2) as u32;
            while m2 != 0 {
                let offset = m2.trailing_zeros() as usize;
                let cand = i + 0x10 + offset;
                if &haystack[cand..cand + n] == needle {
                    return Some(cand);
                }
                m2 &= m2 - 1;
            }

            let mut m3 = _mm_movemask_epi8(eq3) as u32;
            while m3 != 0 {
                let offset = m3.trailing_zeros() as usize;
                let cand = i + 0x20 + offset;
                if &haystack[cand..cand + n] == needle {
                    return Some(cand);
                }
                m3 &= m3 - 1;
            }

            let mut m4 = _mm_movemask_epi8(eq4) as u32;
            while m4 != 0 {
                let offset = m4.trailing_zeros() as usize;
                let cand = i + 0x30 + offset;
                if &haystack[cand..cand + n] == needle {
                    return Some(cand);
                }
                m4 &= m4 - 1;
            }
        }

        i += 0x40;
    }

    while i + 0x10 + n - 1 <= len {
        let v_a = _mm_loadu_si128(ptr.add(i) as *const __m128i);
        let v_b = _mm_loadu_si128(ptr.add(i + 1) as *const __m128i);
        let v_c = _mm_loadu_si128(ptr.add(i + n - 1) as *const __m128i);

        let eq = _mm_and_si128(
            _mm_and_si128(_mm_cmpeq_epi8(v_a, v_needle_a), _mm_cmpeq_epi8(v_b, v_needle_b)),
            _mm_cmpeq_epi8(v_c, v_needle_c),
        );
        let mut m = _mm_movemask_epi8(eq) as u32;

        while m != 0 {
            let offset = m.trailing_zeros() as usize;
            let cand = i + offset;
            if &haystack[cand..cand + n] == needle {
                return Some(cand);
            }
            m &= m - 1;
        }

        i += 0x10;
    }

    haystack[i..].windows(n).position(|w| w == needle).map(|pos| pos + i)
}

#[target_feature(enable = "sse4.2")]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
unsafe fn search_n_sse42(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    let n = needle.len();
    if n == 0 {
        return Some(0);
    }

    if n > haystack.len() {
        return None;
    }

    if n == 1 {
        return search_one(haystack, needle[0]);
    }

    if n == 2 {
        return search_two(haystack, [needle[0], needle[1]]);
    }

    if n == 3 {
        return search_three(haystack, [needle[0], needle[1], needle[2]]);
    }

    let v_needle_a = _mm_set1_epi8(needle[0] as i8);
    let v_needle_b = _mm_set1_epi8(needle[1] as i8);
    let v_needle_c = _mm_set1_epi8(needle[n - 1] as i8);

    let len = haystack.len();
    let ptr = haystack.as_ptr();

    let mut i = 0;
    while i + 0x40 + n - 1 <= len {
        let v1 = _mm_loadu_si128(ptr.add(i) as *const __m128i);
        let v2 = _mm_loadu_si128(ptr.add(i + 0x10) as *const __m128i);
        let v3 = _mm_loadu_si128(ptr.add(i + 0x20) as *const __m128i);
        let v4 = _mm_loadu_si128(ptr.add(i + 0x30) as *const __m128i);
        let v5 = _mm_loadu_si128(ptr.add(i + 0x40) as *const __m128i);

        let v1_b = _mm_alignr_epi8(v2, v1, 1);
        let v2_b = _mm_alignr_epi8(v3, v2, 1);
        let v3_b = _mm_alignr_epi8(v4, v3, 1);
        let v4_b = _mm_alignr_epi8(v5, v4, 1);

        let v1_c = _mm_loadu_si128(ptr.add(i + n - 1) as *const __m128i);
        let v2_c = _mm_loadu_si128(ptr.add(i + 0x10 + n - 1) as *const __m128i);
        let v3_c = _mm_loadu_si128(ptr.add(i + 0x20 + n - 1) as *const __m128i);
        let v4_c = _mm_loadu_si128(ptr.add(i + 0x30 + n - 1) as *const __m128i);

        let eq1 = _mm_and_si128(
            _mm_and_si128(_mm_cmpeq_epi8(v1, v_needle_a), _mm_cmpeq_epi8(v1_b, v_needle_b)),
            _mm_cmpeq_epi8(v1_c, v_needle_c),
        );
        let eq2 = _mm_and_si128(
            _mm_and_si128(_mm_cmpeq_epi8(v2, v_needle_a), _mm_cmpeq_epi8(v2_b, v_needle_b)),
            _mm_cmpeq_epi8(v2_c, v_needle_c),
        );
        let eq3 = _mm_and_si128(
            _mm_and_si128(_mm_cmpeq_epi8(v3, v_needle_a), _mm_cmpeq_epi8(v3_b, v_needle_b)),
            _mm_cmpeq_epi8(v3_c, v_needle_c),
        );
        let eq4 = _mm_and_si128(
            _mm_and_si128(_mm_cmpeq_epi8(v4, v_needle_a), _mm_cmpeq_epi8(v4_b, v_needle_b)),
            _mm_cmpeq_epi8(v4_c, v_needle_c),
        );

        let or1 = _mm_or_si128(eq1, eq2);
        let or2 = _mm_or_si128(eq3, eq4);
        let or_vec = _mm_or_si128(or1, or2);

        if _mm_testz_si128(or_vec, or_vec) == 0 {
            let mut m1 = _mm_movemask_epi8(eq1) as u32;
            while m1 != 0 {
                let offset = m1.trailing_zeros() as usize;
                let cand = i + offset;
                if &haystack[cand..cand + n] == needle {
                    return Some(cand);
                }
                m1 &= m1 - 1;
            }

            let mut m2 = _mm_movemask_epi8(eq2) as u32;
            while m2 != 0 {
                let offset = m2.trailing_zeros() as usize;
                let cand = i + 0x10 + offset;
                if &haystack[cand..cand + n] == needle {
                    return Some(cand);
                }
                m2 &= m2 - 1;
            }

            let mut m3 = _mm_movemask_epi8(eq3) as u32;
            while m3 != 0 {
                let offset = m3.trailing_zeros() as usize;
                let cand = i + 0x20 + offset;
                if &haystack[cand..cand + n] == needle {
                    return Some(cand);
                }
                m3 &= m3 - 1;
            }

            let mut m4 = _mm_movemask_epi8(eq4) as u32;
            while m4 != 0 {
                let offset = m4.trailing_zeros() as usize;
                let cand = i + 0x30 + offset;
                if &haystack[cand..cand + n] == needle {
                    return Some(cand);
                }
                m4 &= m4 - 1;
            }
        }

        i += 0x40;
    }

    while i + 0x10 + n - 1 <= len {
        let v_a = _mm_loadu_si128(ptr.add(i) as *const __m128i);
        let v_b = _mm_loadu_si128(ptr.add(i + 1) as *const __m128i);
        let v_c = _mm_loadu_si128(ptr.add(i + n - 1) as *const __m128i);

        let eq = _mm_and_si128(
            _mm_and_si128(_mm_cmpeq_epi8(v_a, v_needle_a), _mm_cmpeq_epi8(v_b, v_needle_b)),
            _mm_cmpeq_epi8(v_c, v_needle_c),
        );
        let mut m = _mm_movemask_epi8(eq) as u32;

        while m != 0 {
            let offset = m.trailing_zeros() as usize;
            let cand = i + offset;
            if &haystack[cand..cand + n] == needle {
                return Some(cand);
            }
            m &= m - 1;
        }

        i += 0x10;
    }

    haystack[i..].windows(n).position(|w| w == needle).map(|pos| pos + i)
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn search_n_avx2(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    let n = needle.len();
    if n == 0 {
        return Some(0);
    }

    if n > haystack.len() {
        return None;
    }

    if n == 1 {
        return search_one(haystack, needle[0]);
    }

    if n == 2 {
        return search_two(haystack, [needle[0], needle[1]]);
    }

    if n == 3 {
        return search_three(haystack, [needle[0], needle[1], needle[2]]);
    }

    let v_needle_a = _mm256_set1_epi8(needle[0] as i8);
    let v_needle_b = _mm256_set1_epi8(needle[1] as i8);
    let v_needle_c = _mm256_set1_epi8(needle[n - 1] as i8);

    let len = haystack.len();
    let ptr = haystack.as_ptr();

    let mut i = 0;
    while i + 0x40 + n - 1 <= len {
        let v1_a = _mm256_loadu_si256(ptr.add(i) as *const __m256i);
        let v1_b = _mm256_loadu_si256(ptr.add(i + 1) as *const __m256i);
        let v1_c = _mm256_loadu_si256(ptr.add(i + n - 1) as *const __m256i);

        let v2_a = _mm256_loadu_si256(ptr.add(i + 0x20) as *const __m256i);
        let v2_b = _mm256_loadu_si256(ptr.add(i + 0x21) as *const __m256i);
        let v2_c = _mm256_loadu_si256(ptr.add(i + 0x20 + n - 1) as *const __m256i);

        let eq1 = _mm256_and_si256(
            _mm256_and_si256(
                _mm256_cmpeq_epi8(v1_a, v_needle_a),
                _mm256_cmpeq_epi8(v1_b, v_needle_b),
            ),
            _mm256_cmpeq_epi8(v1_c, v_needle_c),
        );
        let eq2 = _mm256_and_si256(
            _mm256_and_si256(
                _mm256_cmpeq_epi8(v2_a, v_needle_a),
                _mm256_cmpeq_epi8(v2_b, v_needle_b),
            ),
            _mm256_cmpeq_epi8(v2_c, v_needle_c),
        );

        let or_vec = _mm256_or_si256(eq1, eq2);
        if _mm256_movemask_epi8(or_vec) != 0 {
            let mut m1 = _mm256_movemask_epi8(eq1) as u32;
            while m1 != 0 {
                let offset = m1.trailing_zeros() as usize;
                let cand = i + offset;
                if &haystack[cand..cand + n] == needle {
                    return Some(cand);
                }
                m1 &= m1 - 1;
            }

            let mut m2 = _mm256_movemask_epi8(eq2) as u32;
            while m2 != 0 {
                let offset = m2.trailing_zeros() as usize;
                let cand = i + 0x20 + offset;
                if &haystack[cand..cand + n] == needle {
                    return Some(cand);
                }
                m2 &= m2 - 1;
            }
        }

        i += 0x40;
    }

    if i + 0x20 + n - 1 <= len {
        let v_a = _mm256_loadu_si256(ptr.add(i) as *const __m256i);
        let v_b = _mm256_loadu_si256(ptr.add(i + 1) as *const __m256i);
        let v_c = _mm256_loadu_si256(ptr.add(i + n - 1) as *const __m256i);

        let eq = _mm256_and_si256(
            _mm256_and_si256(
                _mm256_cmpeq_epi8(v_a, v_needle_a),
                _mm256_cmpeq_epi8(v_b, v_needle_b),
            ),
            _mm256_cmpeq_epi8(v_c, v_needle_c),
        );

        let mut m = _mm256_movemask_epi8(eq) as u32;
        while m != 0 {
            let offset = m.trailing_zeros() as usize;
            let cand = i + offset;
            if &haystack[cand..cand + n] == needle {
                return Some(cand);
            }
            m &= m - 1;
        }

        i += 0x20;
    }

    haystack[i..].windows(n).position(|w| w == needle).map(|pos| pos + i)
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512bw")]
unsafe fn search_n_avx512(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    let n = needle.len();
    if n == 0 {
        return Some(0);
    }

    if n > haystack.len() {
        return None;
    }

    if n == 1 {
        return search_one(haystack, needle[0]);
    }

    if n == 2 {
        return search_two(haystack, [needle[0], needle[1]]);
    }

    if n == 3 {
        return search_three(haystack, [needle[0], needle[1], needle[2]]);
    }

    let v_needle_a = _mm512_set1_epi8(needle[0] as i8);
    let v_needle_b = _mm512_set1_epi8(needle[1] as i8);
    let v_needle_c = _mm512_set1_epi8(needle[n - 1] as i8);

    let len = haystack.len();
    let ptr = haystack.as_ptr();

    let mut i = 0;
    while i + 0x80 + n - 1 <= len {
        let v1_a = _mm512_loadu_si512(ptr.add(i) as *const _);
        let v1_b = _mm512_loadu_si512(ptr.add(i + 1) as *const _);
        let v1_c = _mm512_loadu_si512(ptr.add(i + n - 1) as *const _);

        let v2_a = _mm512_loadu_si512(ptr.add(i + 0x40) as *const _);
        let v2_b = _mm512_loadu_si512(ptr.add(i + 0x41) as *const _);
        let v2_c = _mm512_loadu_si512(ptr.add(i + 0x40 + n - 1) as *const _);

        let mut eq1 = _mm512_cmpeq_epi8_mask(v1_a, v_needle_a)
            & _mm512_cmpeq_epi8_mask(v1_b, v_needle_b)
            & _mm512_cmpeq_epi8_mask(v1_c, v_needle_c);

        let mut eq2 = _mm512_cmpeq_epi8_mask(v2_a, v_needle_a)
            & _mm512_cmpeq_epi8_mask(v2_b, v_needle_b)
            & _mm512_cmpeq_epi8_mask(v2_c, v_needle_c);

        if (eq1 | eq2) != 0 {
            while eq1 != 0 {
                let offset = eq1.trailing_zeros() as usize;
                let cand = i + offset;
                if &haystack[cand..cand + n] == needle {
                    return Some(cand);
                }
                eq1 &= eq1 - 1;
            }

            while eq2 != 0 {
                let offset = eq2.trailing_zeros() as usize;
                let cand = i + 0x40 + offset;
                if &haystack[cand..cand + n] == needle {
                    return Some(cand);
                }
                eq2 &= eq2 - 1;
            }
        }

        i += 0x80;
    }

    if i + 0x40 + n - 1 <= len {
        let v_a = _mm512_loadu_si512(ptr.add(i) as *const _);
        let v_b = _mm512_loadu_si512(ptr.add(i + 1) as *const _);
        let v_c = _mm512_loadu_si512(ptr.add(i + n - 1) as *const _);

        let mut eq = _mm512_cmpeq_epi8_mask(v_a, v_needle_a)
            & _mm512_cmpeq_epi8_mask(v_b, v_needle_b)
            & _mm512_cmpeq_epi8_mask(v_c, v_needle_c);

        while eq != 0 {
            let offset = eq.trailing_zeros() as usize;
            let cand = i + offset;
            if &haystack[cand..cand + n] == needle {
                return Some(cand);
            }
            eq &= eq - 1;
        }

        i += 0x40;
    }

    haystack[i..].windows(n).position(|w| w == needle).map(|pos| pos + i)
}

#[inline(always)]
#[cfg(target_arch = "aarch64")]
unsafe fn search_n_neon(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    #[inline(always)]
    unsafe fn has_match(v: uint8x16_t) -> bool {
        let u = vreinterpretq_u64_u8(v);
        (vgetq_lane_u64(u, 0) | vgetq_lane_u64(u, 1)) != 0
    }

    let n = needle.len();
    if n == 0 {
        return Some(0);
    }

    if n > haystack.len() {
        return None;
    }

    if n == 1 {
        return search_one(haystack, needle[0]);
    }

    if n == 2 {
        return search_two(haystack, [needle[0], needle[1]]);
    }

    if n == 3 {
        return search_three(haystack, [needle[0], needle[1], needle[2]]);
    }

    let v_needle_a = vdupq_n_u8(needle[0]);
    let v_needle_b = vdupq_n_u8(needle[1]);
    let v_needle_c = vdupq_n_u8(needle[n - 1]);

    let len = haystack.len();
    let ptr = haystack.as_ptr();

    let mut i = 0;
    while i + 0x40 + n - 1 <= len {
        let v1_a = vld1q_u8(ptr.add(i));
        let v1_b = vld1q_u8(ptr.add(i + 1));
        let v1_c = vld1q_u8(ptr.add(i + n - 1));

        let v2_a = vld1q_u8(ptr.add(i + 0x10));
        let v2_b = vld1q_u8(ptr.add(i + 0x11));
        let v2_c = vld1q_u8(ptr.add(i + 0x10 + n - 1));

        let v3_a = vld1q_u8(ptr.add(i + 0x20));
        let v3_b = vld1q_u8(ptr.add(i + 0x21));
        let v3_c = vld1q_u8(ptr.add(i + 0x20 + n - 1));

        let v4_a = vld1q_u8(ptr.add(i + 0x30));
        let v4_b = vld1q_u8(ptr.add(i + 0x31));
        let v4_c = vld1q_u8(ptr.add(i + 0x30 + n - 1));

        let eq1 = vandq_u8(
            vandq_u8(vceqq_u8(v1_a, v_needle_a), vceqq_u8(v1_b, v_needle_b)),
            vceqq_u8(v1_c, v_needle_c),
        );
        let eq2 = vandq_u8(
            vandq_u8(vceqq_u8(v2_a, v_needle_a), vceqq_u8(v2_b, v_needle_b)),
            vceqq_u8(v2_c, v_needle_c),
        );
        let eq3 = vandq_u8(
            vandq_u8(vceqq_u8(v3_a, v_needle_a), vceqq_u8(v3_b, v_needle_b)),
            vceqq_u8(v3_c, v_needle_c),
        );
        let eq4 = vandq_u8(
            vandq_u8(vceqq_u8(v4_a, v_needle_a), vceqq_u8(v4_b, v_needle_b)),
            vceqq_u8(v4_c, v_needle_c),
        );

        let or01 = vorrq_u8(eq1, eq2);
        let or23 = vorrq_u8(eq3, eq4);
        let or_all = vorrq_u8(or01, or23);

        if has_match(or_all) {
            if has_match(eq1) {
                if let Some(pos) = verify_candidates_neon(haystack, needle, i, eq1) {
                    return Some(pos);
                }
            }

            if has_match(eq2) {
                if let Some(pos) = verify_candidates_neon(haystack, needle, i + 0x10, eq2) {
                    return Some(pos);
                }
            }

            if has_match(eq3) {
                if let Some(pos) = verify_candidates_neon(haystack, needle, i + 0x20, eq3) {
                    return Some(pos);
                }
            }

            if has_match(eq4) {
                if let Some(pos) = verify_candidates_neon(haystack, needle, i + 0x30, eq4) {
                    return Some(pos);
                }
            }
        }

        i += 0x40;
    }

    while i + 0x10 + n - 1 <= len {
        let v_a = vld1q_u8(ptr.add(i));
        let v_b = vld1q_u8(ptr.add(i + 1));
        let v_c = vld1q_u8(ptr.add(i + n - 1));

        let eq = vandq_u8(
            vandq_u8(vceqq_u8(v_a, v_needle_a), vceqq_u8(v_b, v_needle_b)),
            vceqq_u8(v_c, v_needle_c),
        );

        if has_match(eq) {
            if let Some(pos) = verify_candidates_neon(haystack, needle, i, eq) {
                return Some(pos);
            }
        }

        i += 0x10;
    }

    haystack[i..].windows(n).position(|w| w == needle).map(|pos| pos + i)
}

#[inline(always)]
#[cfg(target_arch = "aarch64")]
unsafe fn verify_candidates_neon(
    haystack: &[u8],
    needle: &[u8],
    base_idx: usize,
    eq: uint8x16_t,
) -> Option<usize> {
    let n = needle.len();
    let eq_u64 = vreinterpretq_u64_u8(eq);
    let mut lane0 = vgetq_lane_u64(eq_u64, 0);

    while lane0 != 0 {
        let offset = (lane0.trailing_zeros() / 8) as usize;
        let cand = base_idx + offset;
        if &haystack[cand..cand + n] == needle {
            return Some(cand);
        }
        clear_lowest_match_64(&mut lane0);
    }

    let mut lane1 = vgetq_lane_u64(eq_u64, 1);
    while lane1 != 0 {
        let offset = 8 + (lane1.trailing_zeros() / 8) as usize;
        let cand = base_idx + offset;
        if &haystack[cand..cand + n] == needle {
            return Some(cand);
        }
        clear_lowest_match_64(&mut lane1);
    }

    None
}

#[target_feature(enable = "neon")]
#[cfg(all(target_arch = "arm", target_feature = "neon", target_pointer_width = "32"))]
unsafe fn search_n_neon(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    #[inline(always)]
    unsafe fn any_match(v: uint8x16_t) -> bool {
        let lanes = vreinterpretq_u64_u8(v);
        (vgetq_lane_u64(lanes, 0) | vgetq_lane_u64(lanes, 1)) != 0
    }

    let n = needle.len();
    if n == 0 {
        return Some(0);
    }

    if n > haystack.len() {
        return None;
    }

    if n == 1 {
        return search_one(haystack, needle[0]);
    }

    if n == 2 {
        return search_two(haystack, [needle[0], needle[1]]);
    }

    if n == 3 {
        return search_three(haystack, [needle[0], needle[1], needle[2]]);
    }

    let v_needle_a = vdupq_n_u8(needle[0]);
    let v_needle_b = vdupq_n_u8(needle[1]);
    let v_needle_c = vdupq_n_u8(needle[n - 1]);

    let len = haystack.len();
    let ptr = haystack.as_ptr();

    let mut i = 0;
    while i + 0x40 + n - 1 <= len {
        let v1_a = vld1q_u8(ptr.add(i));
        let v1_b = vld1q_u8(ptr.add(i + 1));
        let v1_c = vld1q_u8(ptr.add(i + n - 1));

        let v2_a = vld1q_u8(ptr.add(i + 0x10));
        let v2_b = vld1q_u8(ptr.add(i + 0x11));
        let v2_c = vld1q_u8(ptr.add(i + 0x10 + n - 1));

        let v3_a = vld1q_u8(ptr.add(i + 0x20));
        let v3_b = vld1q_u8(ptr.add(i + 0x21));
        let v3_c = vld1q_u8(ptr.add(i + 0x20 + n - 1));

        let v4_a = vld1q_u8(ptr.add(i + 0x30));
        let v4_b = vld1q_u8(ptr.add(i + 0x31));
        let v4_c = vld1q_u8(ptr.add(i + 0x30 + n - 1));

        let eq1 = vandq_u8(
            vandq_u8(vceqq_u8(v1_a, v_needle_a), vceqq_u8(v1_b, v_needle_b)),
            vceqq_u8(v1_c, v_needle_c),
        );
        let eq2 = vandq_u8(
            vandq_u8(vceqq_u8(v2_a, v_needle_a), vceqq_u8(v2_b, v_needle_b)),
            vceqq_u8(v2_c, v_needle_c),
        );
        let eq3 = vandq_u8(
            vandq_u8(vceqq_u8(v3_a, v_needle_a), vceqq_u8(v3_b, v_needle_b)),
            vceqq_u8(v3_c, v_needle_c),
        );
        let eq4 = vandq_u8(
            vandq_u8(vceqq_u8(v4_a, v_needle_a), vceqq_u8(v4_b, v_needle_b)),
            vceqq_u8(v4_c, v_needle_c),
        );

        let or01 = vorrq_u8(eq1, eq2);
        let or23 = vorrq_u8(eq3, eq4);
        let or_all = vorrq_u8(or01, or23);

        if any_match(or_all) {
            if any_match(eq1) {
                if let Some(pos) = verify_candidates_neon_arm32(haystack, needle, i, eq1) {
                    return Some(pos);
                }
            }

            if any_match(eq2) {
                if let Some(pos) = verify_candidates_neon_arm32(haystack, needle, i + 0x10, eq2) {
                    return Some(pos);
                }
            }

            if any_match(eq3) {
                if let Some(pos) = verify_candidates_neon_arm32(haystack, needle, i + 0x20, eq3) {
                    return Some(pos);
                }
            }

            if any_match(eq4) {
                if let Some(pos) = verify_candidates_neon_arm32(haystack, needle, i + 0x30, eq4) {
                    return Some(pos);
                }
            }
        }

        i += 0x40;
    }

    while i + 0x10 + n - 1 <= len {
        let v_a = vld1q_u8(ptr.add(i));
        let v_b = vld1q_u8(ptr.add(i + 1));
        let v_c = vld1q_u8(ptr.add(i + n - 1));

        let eq = vandq_u8(
            vandq_u8(vceqq_u8(v_a, v_needle_a), vceqq_u8(v_b, v_needle_b)),
            vceqq_u8(v_c, v_needle_c),
        );

        if any_match(eq) {
            if let Some(pos) = verify_candidates_neon_arm32(haystack, needle, i, eq) {
                return Some(pos);
            }
        }

        i += 0x10;
    }

    haystack[i..].windows(n).position(|w| w == needle).map(|pos| pos + i)
}

#[inline(always)]
#[cfg(all(target_arch = "arm", target_feature = "neon"))]
unsafe fn verify_candidates_neon_arm32(
    haystack: &[u8],
    needle: &[u8],
    base_idx: usize,
    eq: uint8x16_t,
) -> Option<usize> {
    let n = needle.len();
    let eq_u64 = vreinterpretq_u64_u8(eq);
    let mut lane0 = vgetq_lane_u64(eq_u64, 0);

    while lane0 != 0 {
        let offset = (lane0.trailing_zeros() / 8) as usize;
        let cand = base_idx + offset;
        if &haystack[cand..cand + n] == needle {
            return Some(cand);
        }
        clear_lowest_match_64(&mut lane0);
    }

    let mut lane1 = vgetq_lane_u64(eq_u64, 1);
    while lane1 != 0 {
        let offset = 8 + (lane1.trailing_zeros() / 8) as usize;
        let cand = base_idx + offset;
        if &haystack[cand..cand + n] == needle {
            return Some(cand);
        }
        clear_lowest_match_64(&mut lane1);
    }

    None
}

#[target_feature(enable = "simd128")]
#[cfg(all(target_arch = "wasm32", target_feature = "simd128"))]
unsafe fn search_n_simd128(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    let n = needle.len();
    if n == 0 {
        return Some(0);
    }

    if n > haystack.len() {
        return None;
    }

    if n == 1 {
        return search_one(haystack, needle[0]);
    }

    if n == 2 {
        return search_two(haystack, [needle[0], needle[1]]);
    }

    if n == 3 {
        return search_three(haystack, [needle[0], needle[1], needle[2]]);
    }

    let v_needle_a = u8x16_splat(needle[0]);
    let v_needle_b = u8x16_splat(needle[1]);
    let v_needle_c = u8x16_splat(needle[n - 1]);

    let len = haystack.len();
    let ptr = haystack.as_ptr();

    let mut i = 0;
    while i + 0x40 + n - 1 <= len {
        let v1_a = v128_load(ptr.add(i) as *const v128);
        let v1_b = v128_load(ptr.add(i + 1) as *const v128);
        let v1_c = v128_load(ptr.add(i + n - 1) as *const v128);

        let v2_a = v128_load(ptr.add(i + 0x10) as *const v128);
        let v2_b = v128_load(ptr.add(i + 0x11) as *const v128);
        let v2_c = v128_load(ptr.add(i + 0x10 + n - 1) as *const v128);

        let v3_a = v128_load(ptr.add(i + 0x20) as *const v128);
        let v3_b = v128_load(ptr.add(i + 0x21) as *const v128);
        let v3_c = v128_load(ptr.add(i + 0x20 + n - 1) as *const v128);

        let v4_a = v128_load(ptr.add(i + 0x30) as *const v128);
        let v4_b = v128_load(ptr.add(i + 0x31) as *const v128);
        let v4_c = v128_load(ptr.add(i + 0x30 + n - 1) as *const v128);

        let eq1 = v128_and(
            v128_and(u8x16_eq(v1_a, v_needle_a), u8x16_eq(v1_b, v_needle_b)),
            u8x16_eq(v1_c, v_needle_c),
        );
        let eq2 = v128_and(
            v128_and(u8x16_eq(v2_a, v_needle_a), u8x16_eq(v2_b, v_needle_b)),
            u8x16_eq(v2_c, v_needle_c),
        );
        let eq3 = v128_and(
            v128_and(u8x16_eq(v3_a, v_needle_a), u8x16_eq(v3_b, v_needle_b)),
            u8x16_eq(v3_c, v_needle_c),
        );
        let eq4 = v128_and(
            v128_and(u8x16_eq(v4_a, v_needle_a), u8x16_eq(v4_b, v_needle_b)),
            u8x16_eq(v4_c, v_needle_c),
        );

        let or1 = v128_or(eq1, eq2);
        let or2 = v128_or(eq3, eq4);
        let or_vec = v128_or(or1, or2);

        if u8x16_bitmask(or_vec) != 0 {
            let mut m1 = u8x16_bitmask(eq1) as u32;
            while m1 != 0 {
                let offset = m1.trailing_zeros() as usize;
                let cand = i + offset;
                if &haystack[cand..cand + n] == needle {
                    return Some(cand);
                }
                m1 &= m1 - 1;
            }

            let mut m2 = u8x16_bitmask(eq2) as u32;
            while m2 != 0 {
                let offset = m2.trailing_zeros() as usize;
                let cand = i + 0x10 + offset;
                if &haystack[cand..cand + n] == needle {
                    return Some(cand);
                }
                m2 &= m2 - 1;
            }

            let mut m3 = u8x16_bitmask(eq3) as u32;
            while m3 != 0 {
                let offset = m3.trailing_zeros() as usize;
                let cand = i + 0x20 + offset;
                if &haystack[cand..cand + n] == needle {
                    return Some(cand);
                }
                m3 &= m3 - 1;
            }

            let mut m4 = u8x16_bitmask(eq4) as u32;
            while m4 != 0 {
                let offset = m4.trailing_zeros() as usize;
                let cand = i + 0x30 + offset;
                if &haystack[cand..cand + n] == needle {
                    return Some(cand);
                }
                m4 &= m4 - 1;
            }
        }

        i += 0x40;
    }

    while i + 0x10 + n - 1 <= len {
        let v_a = v128_load(ptr.add(i) as *const v128);
        let v_b = v128_load(ptr.add(i + 1) as *const v128);
        let v_c = v128_load(ptr.add(i + n - 1) as *const v128);

        let eq = v128_and(
            v128_and(u8x16_eq(v_a, v_needle_a), u8x16_eq(v_b, v_needle_b)),
            u8x16_eq(v_c, v_needle_c),
        );
        let mut m = u8x16_bitmask(eq) as u32;

        while m != 0 {
            let offset = m.trailing_zeros() as usize;
            let cand = i + offset;
            if &haystack[cand..cand + n] == needle {
                return Some(cand);
            }
            m &= m - 1;
        }

        i += 0x10;
    }

    haystack[i..].windows(n).position(|w| w == needle).map(|pos| pos + i)
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

        type SearchFn = fn(&[u8], &[u8]) -> Option<usize>;
        let search_fns: [SearchFn; 2] = [search_n_swar64, search_n_swar32];

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

    #[test]
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    fn test_sse2_directly() {
        if std::is_x86_feature_detected!("sse2") {
            run_standard_suite(|h, n| unsafe { search_n_sse2(h, n) });
        }
    }

    #[test]
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    fn test_ssse3_directly() {
        if std::is_x86_feature_detected!("ssse3") {
            run_standard_suite(|h, n| unsafe { search_n_ssse3(h, n) });
        }
    }

    #[test]
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    fn test_sse42_directly() {
        if std::is_x86_feature_detected!("sse4.2") {
            run_standard_suite(|h, n| unsafe { search_n_sse42(h, n) });
        }
    }

    #[test]
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    fn test_avx2_directly() {
        if std::is_x86_feature_detected!("avx2") {
            run_standard_suite(|h, n| unsafe { search_n_avx2(h, n) });
        }
    }

    #[test]
    #[cfg(target_arch = "x86_64")]
    fn test_avx512_directly() {
        if std::is_x86_feature_detected!("avx512bw") {
            run_standard_suite(|h, n| unsafe { search_n_avx512(h, n) });
        }
    }

    #[test]
    #[cfg(target_arch = "aarch64")]
    fn test_neon_aarch64_directly() {
        run_standard_suite(|h, n| unsafe { search_n_neon(h, n) });
    }

    #[test]
    #[cfg(all(target_arch = "arm", target_feature = "neon"))]
    fn test_neon_arm32_directly() {
        run_standard_suite(|h, n| unsafe { search_n_neon(h, n) });
    }

    #[test]
    #[cfg(all(target_arch = "wasm32", target_feature = "simd128"))]
    fn test_simd128_wasm32_directly() {
        run_standard_suite(|h, n| unsafe { search_n_simd128(h, n) });
    }
}
