//! Two-byte needle search implementation with hardware-accelerated SIMD and SWAR routines
//!
//! ## Example
//!
//! ```
//! use ashwa::search_two;
//!
//! let text = b"The quick brown fox jumps over the lazy dog";
//! assert_eq!(search_two(text, *b"qu"), Some(0x04));
//! assert_eq!(search_two(text, *b"ox"), Some(0x11));
//! assert_eq!(search_two(text, *b"!!"), None);
//! ```

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

#[cfg(any(target_pointer_width = "64", test))]
use crate::common::{get_match_index_64, match_qword, LSB64, MSB64};

#[cfg(any(target_pointer_width = "32", test))]
use crate::common::{get_match_index_32, match_dword, LSB32, MSB32};

/// Searches for the first occurrence of a two-byte needle needle in a byte slice haystack
///
/// ## Example
///
/// ```
/// use ashwa::search_two;
///
/// let haystack = b"hello world";
/// assert_eq!(search_two(haystack, *b"el"), Some(0x01));
/// assert_eq!(search_two(haystack, *b"ld"), Some(0x09));
/// assert_eq!(search_two(haystack, *b"zz"), None);
/// ```
#[inline(always)]
#[cfg(target_pointer_width = "64")]
pub fn search_two(haystack: &[u8], needle: [u8; 0x02]) -> Option<usize> {
    #[cfg(target_arch = "x86_64")]
    match get_cpu_feature() {
        ISA::SWAR => search_two_swar64(haystack, needle),
        ISA::SSE2 => unsafe { search_two_sse2(haystack, needle) },
        ISA::SSSE3 => unsafe { search_two_ssse3(haystack, needle) },
        ISA::SSE4_2 => unsafe { search_two_sse42(haystack, needle) },
        ISA::AVX2 => unsafe { search_two_avx2(haystack, needle) },
        ISA::AVX512BW => {
            #[cfg(not(target_feature = "avx512bw"))]
            return unsafe { search_two_avx2(haystack, needle) };

            #[cfg(target_feature = "avx512bw")]
            return unsafe { search_two_avx512(haystack, needle) };
        }
        _ => unreachable!(),
    }

    #[cfg(target_arch = "aarch64")]
    {
        #[cfg(forced_swar_backend)]
        return search_two_swar64(haystack, needle);

        unsafe { search_two_neon(haystack, needle) }
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    {
        search_two_swar64(haystack, needle)
    }
}

/// Searches for the first occurrence of a two-byte needle needle in a byte slice haystack
///
/// ## Example
///
/// ```
/// use ashwa::search_two;
///
/// let haystack = b"hello world";
/// assert_eq!(search_two(haystack, *b"el"), Some(0x01));
/// assert_eq!(search_two(haystack, *b"ld"), Some(0x09));
/// assert_eq!(search_two(haystack, *b"zz"), None);
/// ```
#[cfg(target_pointer_width = "32")]
pub fn search_two(haystack: &[u8], needle: [u8; 0x02]) -> Option<usize> {
    #[cfg(target_arch = "wasm32")]
    {
        #[cfg(target_feature = "simd128")]
        return unsafe { search_two_simd128(haystack, needle) };

        search_two_swar32(haystack, needle)
    }

    #[cfg(target_arch = "arm")]
    {
        #[cfg(target_feature = "neon")]
        return unsafe { search_two_neon(haystack, needle) };

        search_two_swar32(haystack, needle)
    }

    #[cfg(target_arch = "x86")]
    {
        #[cfg(target_feature = "sse4.2")]
        return unsafe { search_two_sse42(haystack, needle) };

        #[cfg(target_feature = "ssse3")]
        return unsafe { search_two_ssse3(haystack, needle) };

        #[cfg(target_feature = "sse2")]
        return unsafe { search_two_sse2(haystack, needle) };

        search_two_swar32(haystack, needle)
    }

    #[cfg(not(any(target_arch = "wasm32", target_arch = "arm", target_arch = "x86")))]
    {
        search_two_swar32(haystack, needle)
    }
}

#[inline(always)]
#[cfg(any(target_pointer_width = "64", test))]
fn search_two_swar64(haystack: &[u8], needle: [u8; 0x02]) -> Option<usize> {
    let needle_a = (needle[0x00] as u64).wrapping_mul(LSB64);
    let needle_b = (needle[0x01] as u64).wrapping_mul(LSB64);

    let mut i = 0x00;
    let len = haystack.len();
    let ptr = haystack.as_ptr();

    while i + 0x21 <= len {
        let w1_a = unsafe { ptr::read_unaligned(ptr.add(i) as *const u64) };
        let w1_b = unsafe { ptr::read_unaligned(ptr.add(i + 0x01) as *const u64) };
        let w2_a = unsafe { ptr::read_unaligned(ptr.add(i + 0x08) as *const u64) };
        let w2_b = unsafe { ptr::read_unaligned(ptr.add(i + 0x09) as *const u64) };
        let w3_a = unsafe { ptr::read_unaligned(ptr.add(i + 0x10) as *const u64) };
        let w3_b = unsafe { ptr::read_unaligned(ptr.add(i + 0x11) as *const u64) };
        let w4_a = unsafe { ptr::read_unaligned(ptr.add(i + 0x18) as *const u64) };
        let w4_b = unsafe { ptr::read_unaligned(ptr.add(i + 0x19) as *const u64) };

        let m1 = match_qword(w1_a, needle_a) & match_qword(w1_b, needle_b);
        let m2 = match_qword(w2_a, needle_a) & match_qword(w2_b, needle_b);
        let m3 = match_qword(w3_a, needle_a) & match_qword(w3_b, needle_b);
        let m4 = match_qword(w4_a, needle_a) & match_qword(w4_b, needle_b);

        if (m1 | m2 | m3 | m4) != 0x00 {
            if m1 != 0x00 {
                return Some(i + get_match_index_64(m1));
            }

            if m2 != 0x00 {
                return Some(i + 0x08 + get_match_index_64(m2));
            }

            if m3 != 0x00 {
                return Some(i + 0x10 + get_match_index_64(m3));
            }

            return Some(i + 0x18 + get_match_index_64(m4));
        }

        i += 0x20;
    }

    while i + 0x09 <= len {
        let w_a = unsafe { ptr::read_unaligned(ptr.add(i) as *const u64) };
        let w_b = unsafe { ptr::read_unaligned(ptr.add(i + 0x01) as *const u64) };

        let m = match_qword(w_a, needle_a) & match_qword(w_b, needle_b);

        if m != 0x00 {
            return Some(i + get_match_index_64(m));
        }

        i += 0x08;
    }

    haystack[i..].windows(0x02).position(|w| w == needle).map(|pos| pos + i)
}

#[inline(always)]
#[cfg(any(target_pointer_width = "32", test))]
fn search_two_swar32(haystack: &[u8], needle: [u8; 0x02]) -> Option<usize> {
    let needle_a = (needle[0x00] as u32).wrapping_mul(LSB32);
    let needle_b = (needle[0x01] as u32).wrapping_mul(LSB32);

    let len = haystack.len();
    let ptr = haystack.as_ptr();

    let mut i = 0x00;
    while i + 0x11 <= len {
        let w1_a = unsafe { ptr::read_unaligned(ptr.add(i) as *const u32) };
        let w1_b = unsafe { ptr::read_unaligned(ptr.add(i + 0x01) as *const u32) };
        let w2_a = unsafe { ptr::read_unaligned(ptr.add(i + 0x04) as *const u32) };
        let w2_b = unsafe { ptr::read_unaligned(ptr.add(i + 0x05) as *const u32) };
        let w3_a = unsafe { ptr::read_unaligned(ptr.add(i + 0x08) as *const u32) };
        let w3_b = unsafe { ptr::read_unaligned(ptr.add(i + 0x09) as *const u32) };
        let w4_a = unsafe { ptr::read_unaligned(ptr.add(i + 0x0C) as *const u32) };
        let w4_b = unsafe { ptr::read_unaligned(ptr.add(i + 0x0D) as *const u32) };

        let m1 = match_dword(w1_a, needle_a) & match_dword(w1_b, needle_b);
        let m2 = match_dword(w2_a, needle_a) & match_dword(w2_b, needle_b);
        let m3 = match_dword(w3_a, needle_a) & match_dword(w3_b, needle_b);
        let m4 = match_dword(w4_a, needle_a) & match_dword(w4_b, needle_b);

        if (m1 | m2 | m3 | m4) != 0x00 {
            if m1 != 0x00 {
                return Some(i + get_match_index_32(m1));
            }

            if m2 != 0x00 {
                return Some(i + 0x04 + get_match_index_32(m2));
            }

            if m3 != 0x00 {
                return Some(i + 0x08 + get_match_index_32(m3));
            }

            return Some(i + 0x0C + get_match_index_32(m4));
        }

        i += 0x10;
    }

    while i + 0x05 <= len {
        let w_a = unsafe { ptr::read_unaligned(ptr.add(i) as *const u32) };
        let w_b = unsafe { ptr::read_unaligned(ptr.add(i + 0x01) as *const u32) };

        let m = match_dword(w_a, needle_a) & match_dword(w_b, needle_b);
        if m != 0x00 {
            return Some(i + get_match_index_32(m));
        }

        i += 0x04;
    }

    haystack[i..].windows(0x02).position(|w| w == needle).map(|pos| pos + i)
}

#[target_feature(enable = "sse2")]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
unsafe fn search_two_sse2(haystack: &[u8], needle: [u8; 0x02]) -> Option<usize> {
    let v_needle_a = _mm_set1_epi8(needle[0x00] as i8);
    let v_needle_b = _mm_set1_epi8(needle[0x01] as i8);

    let len = haystack.len();
    let ptr = haystack.as_ptr();

    let mut i = 0x00;
    while i + 0x41 <= len {
        let v1_a = _mm_loadu_si128(ptr.add(i) as *const __m128i);
        let v1_b = _mm_loadu_si128(ptr.add(i + 0x01) as *const __m128i);
        let v2_a = _mm_loadu_si128(ptr.add(i + 0x10) as *const __m128i);
        let v2_b = _mm_loadu_si128(ptr.add(i + 0x11) as *const __m128i);
        let v3_a = _mm_loadu_si128(ptr.add(i + 0x20) as *const __m128i);
        let v3_b = _mm_loadu_si128(ptr.add(i + 0x21) as *const __m128i);
        let v4_a = _mm_loadu_si128(ptr.add(i + 0x30) as *const __m128i);
        let v4_b = _mm_loadu_si128(ptr.add(i + 0x31) as *const __m128i);

        let eq1 = _mm_and_si128(_mm_cmpeq_epi8(v1_a, v_needle_a), _mm_cmpeq_epi8(v1_b, v_needle_b));
        let eq2 = _mm_and_si128(_mm_cmpeq_epi8(v2_a, v_needle_a), _mm_cmpeq_epi8(v2_b, v_needle_b));
        let eq3 = _mm_and_si128(_mm_cmpeq_epi8(v3_a, v_needle_a), _mm_cmpeq_epi8(v3_b, v_needle_b));
        let eq4 = _mm_and_si128(_mm_cmpeq_epi8(v4_a, v_needle_a), _mm_cmpeq_epi8(v4_b, v_needle_b));

        let or1 = _mm_or_si128(eq1, eq2);
        let or2 = _mm_or_si128(eq3, eq4);
        let or_vec = _mm_or_si128(or1, or2);

        if _mm_movemask_epi8(or_vec) != 0x00 {
            let m1 = _mm_movemask_epi8(eq1);
            if m1 != 0x00 {
                return Some(i + m1.trailing_zeros() as usize);
            }

            let m2 = _mm_movemask_epi8(eq2);
            if m2 != 0x00 {
                return Some(i + 0x10 + m2.trailing_zeros() as usize);
            }

            let m3 = _mm_movemask_epi8(eq3);
            if m3 != 0x00 {
                return Some(i + 0x20 + m3.trailing_zeros() as usize);
            }

            let m4 = _mm_movemask_epi8(eq4);
            return Some(i + 0x30 + m4.trailing_zeros() as usize);
        }

        i += 0x40;
    }

    while i + 0x11 <= len {
        let v_a = _mm_loadu_si128(ptr.add(i) as *const __m128i);
        let v_b = _mm_loadu_si128(ptr.add(i + 0x01) as *const __m128i);

        let eq = _mm_and_si128(_mm_cmpeq_epi8(v_a, v_needle_a), _mm_cmpeq_epi8(v_b, v_needle_b));
        let m = _mm_movemask_epi8(eq);

        if m != 0x00 {
            return Some(i + m.trailing_zeros() as usize);
        }

        i += 0x10;
    }

    haystack[i..].windows(0x02).position(|w| w == needle).map(|pos| pos + i)
}

#[target_feature(enable = "ssse3")]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
unsafe fn search_two_ssse3(haystack: &[u8], needle: [u8; 0x02]) -> Option<usize> {
    let v_needle_a = _mm_set1_epi8(needle[0x00] as i8);
    let v_needle_b = _mm_set1_epi8(needle[0x01] as i8);

    let len = haystack.len();
    let ptr = haystack.as_ptr();

    let mut i = 0x00;
    while i + 0x50 <= len {
        let v1 = _mm_loadu_si128(ptr.add(i) as *const __m128i);
        let v2 = _mm_loadu_si128(ptr.add(i + 0x10) as *const __m128i);
        let v3 = _mm_loadu_si128(ptr.add(i + 0x20) as *const __m128i);
        let v4 = _mm_loadu_si128(ptr.add(i + 0x30) as *const __m128i);
        let v5 = _mm_loadu_si128(ptr.add(i + 0x40) as *const __m128i);

        let v1_b = _mm_alignr_epi8(v2, v1, 0x01);
        let v2_b = _mm_alignr_epi8(v3, v2, 0x01);
        let v3_b = _mm_alignr_epi8(v4, v3, 0x01);
        let v4_b = _mm_alignr_epi8(v5, v4, 0x01);

        let eq1 = _mm_and_si128(_mm_cmpeq_epi8(v1, v_needle_a), _mm_cmpeq_epi8(v1_b, v_needle_b));
        let eq2 = _mm_and_si128(_mm_cmpeq_epi8(v2, v_needle_a), _mm_cmpeq_epi8(v2_b, v_needle_b));
        let eq3 = _mm_and_si128(_mm_cmpeq_epi8(v3, v_needle_a), _mm_cmpeq_epi8(v3_b, v_needle_b));
        let eq4 = _mm_and_si128(_mm_cmpeq_epi8(v4, v_needle_a), _mm_cmpeq_epi8(v4_b, v_needle_b));

        let or1 = _mm_or_si128(eq1, eq2);
        let or2 = _mm_or_si128(eq3, eq4);
        let or_vec = _mm_or_si128(or1, or2);

        if _mm_movemask_epi8(or_vec) != 0x00 {
            let m1 = _mm_movemask_epi8(eq1);
            if m1 != 0x00 {
                return Some(i + m1.trailing_zeros() as usize);
            }

            let m2 = _mm_movemask_epi8(eq2);
            if m2 != 0x00 {
                return Some(i + 0x10 + m2.trailing_zeros() as usize);
            }

            let m3 = _mm_movemask_epi8(eq3);
            if m3 != 0x00 {
                return Some(i + 0x20 + m3.trailing_zeros() as usize);
            }

            let m4 = _mm_movemask_epi8(eq4);
            return Some(i + 0x30 + m4.trailing_zeros() as usize);
        }

        i += 0x40;
    }

    while i + 0x11 <= len {
        let v_a = _mm_loadu_si128(ptr.add(i) as *const __m128i);
        let v_b = _mm_loadu_si128(ptr.add(i + 0x01) as *const __m128i);

        let eq = _mm_and_si128(_mm_cmpeq_epi8(v_a, v_needle_a), _mm_cmpeq_epi8(v_b, v_needle_b));
        let m = _mm_movemask_epi8(eq);

        if m != 0x00 {
            return Some(i + m.trailing_zeros() as usize);
        }

        i += 0x10;
    }

    haystack[i..].windows(0x02).position(|w| w == needle).map(|pos| pos + i)
}

#[target_feature(enable = "sse4.2")]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
unsafe fn search_two_sse42(haystack: &[u8], needle: [u8; 0x02]) -> Option<usize> {
    let v_needle_a = _mm_set1_epi8(needle[0x00] as i8);
    let v_needle_b = _mm_set1_epi8(needle[0x01] as i8);

    let mut i = 0x00;
    let len = haystack.len();
    let ptr = haystack.as_ptr();

    while i + 0x50 <= len {
        let v1 = _mm_loadu_si128(ptr.add(i) as *const __m128i);
        let v2 = _mm_loadu_si128(ptr.add(i + 0x10) as *const __m128i);
        let v3 = _mm_loadu_si128(ptr.add(i + 0x20) as *const __m128i);
        let v4 = _mm_loadu_si128(ptr.add(i + 0x30) as *const __m128i);
        let v5 = _mm_loadu_si128(ptr.add(i + 0x40) as *const __m128i);

        let v1_b = _mm_alignr_epi8(v2, v1, 0x01);
        let v2_b = _mm_alignr_epi8(v3, v2, 0x01);
        let v3_b = _mm_alignr_epi8(v4, v3, 0x01);
        let v4_b = _mm_alignr_epi8(v5, v4, 0x01);

        let eq1 = _mm_and_si128(_mm_cmpeq_epi8(v1, v_needle_a), _mm_cmpeq_epi8(v1_b, v_needle_b));
        let eq2 = _mm_and_si128(_mm_cmpeq_epi8(v2, v_needle_a), _mm_cmpeq_epi8(v2_b, v_needle_b));
        let eq3 = _mm_and_si128(_mm_cmpeq_epi8(v3, v_needle_a), _mm_cmpeq_epi8(v3_b, v_needle_b));
        let eq4 = _mm_and_si128(_mm_cmpeq_epi8(v4, v_needle_a), _mm_cmpeq_epi8(v4_b, v_needle_b));

        let or1 = _mm_or_si128(eq1, eq2);
        let or2 = _mm_or_si128(eq3, eq4);
        let or_vec = _mm_or_si128(or1, or2);

        if _mm_testz_si128(or_vec, or_vec) == 0x00 {
            let m1 = _mm_movemask_epi8(eq1);
            if m1 != 0x00 {
                return Some(i + m1.trailing_zeros() as usize);
            }

            let m2 = _mm_movemask_epi8(eq2);
            if m2 != 0x00 {
                return Some(i + 0x10 + m2.trailing_zeros() as usize);
            }

            let m3 = _mm_movemask_epi8(eq3);
            if m3 != 0x00 {
                return Some(i + 0x20 + m3.trailing_zeros() as usize);
            }

            let m4 = _mm_movemask_epi8(eq4);
            return Some(i + 0x30 + m4.trailing_zeros() as usize);
        }

        i += 0x40;
    }

    while i + 0x11 <= len {
        let v_a = _mm_loadu_si128(ptr.add(i) as *const __m128i);
        let v_b = _mm_loadu_si128(ptr.add(i + 0x01) as *const __m128i);

        let eq = _mm_and_si128(_mm_cmpeq_epi8(v_a, v_needle_a), _mm_cmpeq_epi8(v_b, v_needle_b));
        if _mm_testz_si128(eq, eq) == 0x00 {
            let m = _mm_movemask_epi8(eq);
            return Some(i + m.trailing_zeros() as usize);
        }

        i += 0x10;
    }

    haystack[i..].windows(0x02).position(|w| w == needle).map(|pos| pos + i)
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn search_two_avx2(haystack: &[u8], needle: [u8; 0x02]) -> Option<usize> {
    let v_needle_a = _mm256_set1_epi8(needle[0x00] as i8);
    let v_needle_b = _mm256_set1_epi8(needle[0x01] as i8);

    let len = haystack.len();
    let ptr = haystack.as_ptr();

    let mut i = 0x00;
    while i + 0x41 <= len {
        let v1_a = _mm256_loadu_si256(ptr.add(i) as *const __m256i);
        let v1_b = _mm256_loadu_si256(ptr.add(i + 0x01) as *const __m256i);
        let v2_a = _mm256_loadu_si256(ptr.add(i + 0x20) as *const __m256i);
        let v2_b = _mm256_loadu_si256(ptr.add(i + 0x21) as *const __m256i);

        let eq1 = _mm256_and_si256(
            _mm256_cmpeq_epi8(v1_a, v_needle_a),
            _mm256_cmpeq_epi8(v1_b, v_needle_b),
        );
        let eq2 = _mm256_and_si256(
            _mm256_cmpeq_epi8(v2_a, v_needle_a),
            _mm256_cmpeq_epi8(v2_b, v_needle_b),
        );

        let or_vec = _mm256_or_si256(eq1, eq2);
        if _mm256_movemask_epi8(or_vec) != 0x00 {
            let m1 = _mm256_movemask_epi8(eq1);
            if m1 != 0x00 {
                return Some(i + m1.trailing_zeros() as usize);
            }

            let m2 = _mm256_movemask_epi8(eq2);
            return Some(i + 0x20 + m2.trailing_zeros() as usize);
        }

        i += 0x40;
    }

    if i + 0x21 <= len {
        let v_a = _mm256_loadu_si256(ptr.add(i) as *const __m256i);
        let v_b = _mm256_loadu_si256(ptr.add(i + 0x01) as *const __m256i);

        let eq = _mm256_and_si256(
            _mm256_cmpeq_epi8(v_a, v_needle_a),
            _mm256_cmpeq_epi8(v_b, v_needle_b),
        );

        let m = _mm256_movemask_epi8(eq);
        if m != 0x00 {
            return Some(i + m.trailing_zeros() as usize);
        }

        i += 0x20;
    }

    haystack[i..].windows(0x02).position(|w| w == needle).map(|pos| pos + i)
}

#[target_feature(enable = "avx512bw")]
#[cfg(all(target_arch = "x86_64", target_feature = "avx512bw"))]
unsafe fn search_two_avx512(haystack: &[u8], needle: [u8; 0x02]) -> Option<usize> {
    let v_needle_a = _mm512_set1_epi8(needle[0x00] as i8);
    let v_needle_b = _mm512_set1_epi8(needle[0x01] as i8);

    let len = haystack.len();
    let ptr = haystack.as_ptr();

    let mut i = 0x00;
    while i + 0x81 <= len {
        let v1_a = _mm512_loadu_si512(ptr.add(i) as *const _);
        let v1_b = _mm512_loadu_si512(ptr.add(i + 0x01) as *const _);
        let v2_a = _mm512_loadu_si512(ptr.add(i + 0x40) as *const _);
        let v2_b = _mm512_loadu_si512(ptr.add(i + 0x41) as *const _);

        let eq1 =
            _mm512_cmpeq_epi8_mask(v1_a, v_needle_a) & _mm512_cmpeq_epi8_mask(v1_b, v_needle_b);
        let eq2 =
            _mm512_cmpeq_epi8_mask(v2_a, v_needle_a) & _mm512_cmpeq_epi8_mask(v2_b, v_needle_b);

        if (eq1 | eq2) != 0x00 {
            if eq1 != 0x00 {
                return Some(i + eq1.trailing_zeros() as usize);
            }

            return Some(i + 0x40 + eq2.trailing_zeros() as usize);
        }

        i += 0x80;
    }

    if i + 0x41 <= len {
        let v_a = _mm512_loadu_si512(ptr.add(i) as *const _);
        let v_b = _mm512_loadu_si512(ptr.add(i + 0x01) as *const _);

        let eq = _mm512_cmpeq_epi8_mask(v_a, v_needle_a) & _mm512_cmpeq_epi8_mask(v_b, v_needle_b);
        if eq != 0x00 {
            return Some(i + eq.trailing_zeros() as usize);
        }

        i += 0x40;
    }

    haystack[i..].windows(0x02).position(|w| w == needle).map(|pos| pos + i)
}

#[inline(always)]
#[cfg(target_arch = "aarch64")]
unsafe fn search_two_neon(haystack: &[u8], needle: [u8; 0x02]) -> Option<usize> {
    #[inline(always)]
    unsafe fn has_match(v: uint8x16_t) -> bool {
        let u = vreinterpretq_u64_u8(v);
        (vgetq_lane_u64(u, 0x00) | vgetq_lane_u64(u, 0x01)) != 0x00
    }

    let v_needle_a = vdupq_n_u8(needle[0x00]);
    let v_needle_b = vdupq_n_u8(needle[0x01]);

    let len = haystack.len();
    let ptr = haystack.as_ptr();

    let mut i = 0x00;
    while i + 0x50 <= len {
        let v1 = vld1q_u8(ptr.add(i));
        let v2 = vld1q_u8(ptr.add(i + 0x10));
        let v3 = vld1q_u8(ptr.add(i + 0x20));
        let v4 = vld1q_u8(ptr.add(i + 0x30));
        let v5 = vld1q_u8(ptr.add(i + 0x40));

        let v1_b = vextq_u8(v1, v2, 0x01);
        let v2_b = vextq_u8(v2, v3, 0x01);
        let v3_b = vextq_u8(v3, v4, 0x01);
        let v4_b = vextq_u8(v4, v5, 0x01);

        let eq1 = vandq_u8(vceqq_u8(v1, v_needle_a), vceqq_u8(v1_b, v_needle_b));
        let eq2 = vandq_u8(vceqq_u8(v2, v_needle_a), vceqq_u8(v2_b, v_needle_b));
        let eq3 = vandq_u8(vceqq_u8(v3, v_needle_a), vceqq_u8(v3_b, v_needle_b));
        let eq4 = vandq_u8(vceqq_u8(v4, v_needle_a), vceqq_u8(v4_b, v_needle_b));

        let or01 = vorrq_u8(eq1, eq2);
        let or23 = vorrq_u8(eq3, eq4);
        let or_all = vorrq_u8(or01, or23);

        if has_match(or_all) {
            if has_match(or01) {
                if has_match(eq1) {
                    return Some(i + get_match_index_neon(eq1));
                }

                return Some(i + 0x10 + get_match_index_neon(eq2));
            }

            if has_match(eq3) {
                return Some(i + 0x20 + get_match_index_neon(eq3));
            }

            return Some(i + 0x30 + get_match_index_neon(eq4));
        }

        i += 0x40;
    }

    while i + 0x11 <= len {
        let v_a = vld1q_u8(ptr.add(i));
        let v_b = vld1q_u8(ptr.add(i + 0x01));

        let eq = vandq_u8(vceqq_u8(v_a, v_needle_a), vceqq_u8(v_b, v_needle_b));
        if has_match(eq) {
            return Some(i + get_match_index_neon(eq));
        }

        i += 0x10;
    }

    haystack[i..].windows(0x02).position(|w| w == needle).map(|pos| pos + i)
}

#[inline(always)]
#[cfg(target_arch = "aarch64")]
unsafe fn get_match_index_neon(eq: uint8x16_t) -> usize {
    let eq_u64 = vreinterpretq_u64_u8(eq);
    let lane0 = vgetq_lane_u64(eq_u64, 0x00);

    if lane0 != 0x00 {
        return (lane0.trailing_zeros() / 0x08) as usize;
    }

    let lane1 = vgetq_lane_u64(eq_u64, 0x01);
    0x08 + (lane1.trailing_zeros() / 0x08) as usize
}

#[target_feature(enable = "neon")]
#[cfg(all(target_arch = "arm", target_feature = "neon", target_pointer_width = "32"))]
unsafe fn search_two_neon(haystack: &[u8], needle: [u8; 0x02]) -> Option<usize> {
    #[inline(always)]
    unsafe fn any_match(v: uint8x16_t) -> bool {
        let lanes = vreinterpretq_u64_u8(v);
        vgetq_lane_u64(lanes, 0x00) != 0x00 || vgetq_lane_u64(lanes, 0x01) != 0x00
    }

    let v_needle_a = vdupq_n_u8(needle[0x00]);
    let v_needle_b = vdupq_n_u8(needle[0x01]);

    let len = haystack.len();
    let ptr = haystack.as_ptr();

    let mut i = 0x00;
    while i + 0x50 <= len {
        let v1 = vld1q_u8(ptr.add(i));
        let v2 = vld1q_u8(ptr.add(i + 0x10));
        let v3 = vld1q_u8(ptr.add(i + 0x20));
        let v4 = vld1q_u8(ptr.add(i + 0x30));
        let v5 = vld1q_u8(ptr.add(i + 0x40));

        let v1_b = vextq_u8(v1, v2, 0x01);
        let v2_b = vextq_u8(v2, v3, 0x01);
        let v3_b = vextq_u8(v3, v4, 0x01);
        let v4_b = vextq_u8(v4, v5, 0x01);

        let eq1 = vandq_u8(vceqq_u8(v1, v_needle_a), vceqq_u8(v1_b, v_needle_b));
        let eq2 = vandq_u8(vceqq_u8(v2, v_needle_a), vceqq_u8(v2_b, v_needle_b));
        let eq3 = vandq_u8(vceqq_u8(v3, v_needle_a), vceqq_u8(v3_b, v_needle_b));
        let eq4 = vandq_u8(vceqq_u8(v4, v_needle_a), vceqq_u8(v4_b, v_needle_b));

        let or01 = vorrq_u8(eq1, eq2);
        let or23 = vorrq_u8(eq3, eq4);
        let or_all = vorrq_u8(or01, or23);

        if any_match(or_all) {
            if any_match(or01) {
                if any_match(eq1) {
                    return Some(i + get_match_index_neon_arm32(eq1));
                }

                return Some(i + 0x10 + get_match_index_neon_arm32(eq2));
            }

            if any_match(eq3) {
                return Some(i + 0x20 + get_match_index_neon_arm32(eq3));
            }

            return Some(i + 0x30 + get_match_index_neon_arm32(eq4));
        }

        i += 0x40;
    }

    while i + 0x11 <= len {
        let v_a = vld1q_u8(ptr.add(i));
        let v_b = vld1q_u8(ptr.add(i + 0x01));

        let eq = vandq_u8(vceqq_u8(v_a, v_needle_a), vceqq_u8(v_b, v_needle_b));
        if any_match(eq) {
            return Some(i + get_match_index_neon_arm32(eq));
        }

        i += 0x10;
    }

    haystack[i..].windows(0x02).position(|w| w == needle).map(|pos| pos + i)
}

#[inline(always)]
#[cfg(all(target_arch = "arm", target_feature = "neon"))]
unsafe fn get_match_index_neon_arm32(eq: uint8x16_t) -> usize {
    let eq_u64 = vreinterpretq_u64_u8(eq);

    let lane0 = vgetq_lane_u64(eq_u64, 0x00);
    if lane0 != 0x00 {
        return (lane0.trailing_zeros() / 0x08) as usize;
    }

    let lane1 = vgetq_lane_u64(eq_u64, 0x01);
    0x08 + (lane1.trailing_zeros() / 0x08) as usize
}

#[target_feature(enable = "simd128")]
#[cfg(all(target_arch = "wasm32", target_feature = "simd128"))]
unsafe fn search_two_simd128(haystack: &[u8], needle: [u8; 0x02]) -> Option<usize> {
    let v_needle_a = u8x16_splat(needle[0x00]);
    let v_needle_b = u8x16_splat(needle[0x01]);

    let len = haystack.len();
    let ptr = haystack.as_ptr();

    let mut i = 0x00;
    while i + 0x50 <= len {
        let v1 = v128_load(ptr.add(i) as *const v128);
        let v2 = v128_load(ptr.add(i + 0x10) as *const v128);
        let v3 = v128_load(ptr.add(i + 0x20) as *const v128);
        let v4 = v128_load(ptr.add(i + 0x30) as *const v128);
        let v5 = v128_load(ptr.add(i + 0x40) as *const v128);

        let v1_b = i8x16_shuffle::<
            0x01,
            0x02,
            0x03,
            0x04,
            0x05,
            0x06,
            0x07,
            0x08,
            0x09,
            0x0A,
            0x0B,
            0x0C,
            0x0D,
            0x0E,
            0x0F,
            0x10,
        >(v1, v2);
        let v2_b = i8x16_shuffle::<
            0x01,
            0x02,
            0x03,
            0x04,
            0x05,
            0x06,
            0x07,
            0x08,
            0x09,
            0x0A,
            0x0B,
            0x0C,
            0x0D,
            0x0E,
            0x0F,
            0x10,
        >(v2, v3);
        let v3_b = i8x16_shuffle::<
            0x01,
            0x02,
            0x03,
            0x04,
            0x05,
            0x06,
            0x07,
            0x08,
            0x09,
            0x0A,
            0x0B,
            0x0C,
            0x0D,
            0x0E,
            0x0F,
            0x10,
        >(v3, v4);
        let v4_b = i8x16_shuffle::<
            0x01,
            0x02,
            0x03,
            0x04,
            0x05,
            0x06,
            0x07,
            0x08,
            0x09,
            0x0A,
            0x0B,
            0x0C,
            0x0D,
            0x0E,
            0x0F,
            0x10,
        >(v4, v5);

        let eq1 = v128_and(u8x16_eq(v1, v_needle_a), u8x16_eq(v1_b, v_needle_b));
        let eq2 = v128_and(u8x16_eq(v2, v_needle_a), u8x16_eq(v2_b, v_needle_b));
        let eq3 = v128_and(u8x16_eq(v3, v_needle_a), u8x16_eq(v3_b, v_needle_b));
        let eq4 = v128_and(u8x16_eq(v4, v_needle_a), u8x16_eq(v4_b, v_needle_b));

        let or1 = v128_or(eq1, eq2);
        let or2 = v128_or(eq3, eq4);
        let or_vec = v128_or(or1, or2);

        if u8x16_bitmask(or_vec) != 0x00 {
            let m1 = u8x16_bitmask(eq1);
            if m1 != 0x00 {
                return Some(i + m1.trailing_zeros() as usize);
            }

            let m2 = u8x16_bitmask(eq2);
            if m2 != 0x00 {
                return Some(i + 0x10 + m2.trailing_zeros() as usize);
            }

            let m3 = u8x16_bitmask(eq3);
            if m3 != 0x00 {
                return Some(i + 0x20 + m3.trailing_zeros() as usize);
            }

            let m4 = u8x16_bitmask(eq4);
            return Some(i + 0x30 + m4.trailing_zeros() as usize);
        }

        i += 0x40;
    }

    while i + 0x11 <= len {
        let v_a = v128_load(ptr.add(i) as *const v128);
        let v_b = v128_load(ptr.add(i + 0x01) as *const v128);

        let eq = v128_and(u8x16_eq(v_a, v_needle_a), u8x16_eq(v_b, v_needle_b));
        let mask = u8x16_bitmask(eq);

        if mask != 0x00 {
            return Some(i + mask.trailing_zeros() as usize);
        }

        i += 0x10;
    }

    haystack[i..].windows(0x02).position(|w| w == needle).map(|pos| pos + i)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_standard_suite(search_fn: impl Fn(&[u8], [u8; 0x02]) -> Option<usize>) {
        assert_eq!(search_fn(b"", *b"ab"), None);

        assert_eq!(search_fn(b"a", *b"ab"), None);
        assert_eq!(search_fn(b"b", *b"ab"), None);

        assert_eq!(search_fn(b"ab", *b"ab"), Some(0x00));
        assert_eq!(search_fn(b"ac", *b"ab"), None);
        assert_eq!(search_fn(b"ba", *b"ab"), None);

        let haystack = b"the quick brown fox jumps over the lazy dog";
        assert_eq!(search_fn(haystack, *b"ZZ"), None);
        assert_eq!(search_fn(haystack, *b"!!"), None);
        assert_eq!(search_fn(haystack, *b"th"), Some(0x00));
        assert_eq!(search_fn(haystack, *b"he"), Some(0x01));
        assert_eq!(search_fn(haystack, *b"qu"), Some(0x04));
        assert_eq!(search_fn(haystack, *b"ox"), Some(0x11));
        assert_eq!(search_fn(haystack, *b"do"), Some(0x28));
        assert_eq!(search_fn(haystack, *b"og"), Some(0x29));

        for len in 0x02..0x09 {
            let mut h = vec![b'x'; len];
            for pos in 0x00..len - 0x01 {
                h[pos] = b'a';
                h[pos + 0x01] = b'b';
                assert_eq!(
                    search_fn(&h, *b"ab"),
                    Some(pos),
                    "Failed finding needle at pos {} in len {}",
                    pos,
                    len
                );

                h[pos] = b'x';
                h[pos + 0x01] = b'x';
            }
        }

        let mut h8 = [b'-'; 0x08];
        let mut h9 = [b'-'; 0x09];
        let mut h16 = [b'-'; 0x10];
        let mut h17 = [b'-'; 0x11];
        let mut h24 = [b'-'; 0x18];
        let mut h25 = [b'-'; 0x19];
        let mut h32 = [b'-'; 0x20];
        let mut h33 = [b'-'; 0x21];
        let mut h34 = [b'-'; 0x22];

        h8[0x06] = b'A';
        h8[0x07] = b'B';
        assert_eq!(search_fn(&h8, *b"AB"), Some(0x06));

        h9[0x07] = b'A';
        h9[0x08] = b'B';
        assert_eq!(search_fn(&h9, *b"AB"), Some(0x07));

        h16[0x0E] = b'C';
        h16[0x0F] = b'D';
        assert_eq!(search_fn(&h16, *b"CD"), Some(0x0E));

        h17[0x0F] = b'C';
        h17[0x10] = b'D';
        assert_eq!(search_fn(&h17, *b"CD"), Some(0x0F));

        h24[0x16] = b'E';
        h24[0x17] = b'F';
        assert_eq!(search_fn(&h24, *b"EF"), Some(0x16));

        h25[0x17] = b'E';
        h25[0x18] = b'F';
        assert_eq!(search_fn(&h25, *b"EF"), Some(0x17));

        h32[0x1E] = b'G';
        h32[0x1F] = b'H';
        assert_eq!(search_fn(&h32, *b"GH"), Some(0x1E));

        h33[0x1F] = b'G';
        h33[0x20] = b'H';
        assert_eq!(search_fn(&h33, *b"GH"), Some(0x1F));

        h34[0x20] = b'G';
        h34[0x21] = b'H';
        assert_eq!(search_fn(&h34, *b"GH"), Some(0x20));

        let cross_positions = [
            0x03, 0x04, 0x07, 0x08, 0x0B, 0x0C, 0x0F, 0x10, 0x13, 0x14, 0x17, 0x18, 0x1B, 0x1C,
            0x1F, 0x20, 0x27, 0x28, 0x3F, 0x40,
        ];
        let mut cross_buf = vec![b'-'; 0x80];
        for &pos in &cross_positions {
            cross_buf[pos] = b'Y';
            cross_buf[pos + 0x01] = b'Z';
            assert_eq!(
                search_fn(&cross_buf, *b"YZ"),
                Some(pos),
                "Failed straddling cross-word position {}",
                pos
            );
            cross_buf[pos] = b'-';
            cross_buf[pos + 0x01] = b'-';
        }

        let mut haystack = vec![b'-'; 0x200];
        for i in 0x00..haystack.len() - 0x01 {
            haystack[i] = b'A';
            haystack[i + 0x01] = b'B';
            assert_eq!(
                search_fn(&haystack, *b"AB"),
                Some(i),
                "Failed finding needle at index {}",
                i
            );

            haystack[i] = b'-';
            haystack[i + 0x01] = b'-';
        }

        let mut haystack_first_match = vec![b'A'; 0x100];
        assert_eq!(search_fn(&haystack_first_match, *b"AB"), None);

        haystack_first_match[0x7A] = b'B';
        assert_eq!(search_fn(&haystack_first_match, *b"AB"), Some(0x79));

        let mut haystack_second_match = vec![b'B'; 0x100];
        assert_eq!(search_fn(&haystack_second_match, *b"AB"), None);

        haystack_second_match[0x40] = b'A';
        assert_eq!(search_fn(&haystack_second_match, *b"AB"), Some(0x40));

        assert_eq!(search_fn(b"aaaaaa", *b"aa"), Some(0x00));
        assert_eq!(search_fn(b"baaaaa", *b"aa"), Some(0x01));
        assert_eq!(search_fn(b"bbaaaa", *b"aa"), Some(0x02));
        assert_eq!(search_fn(b"ababab", *b"ab"), Some(0x00));
        assert_eq!(search_fn(b"bababa", *b"ab"), Some(0x01));

        let mut haystack_high = vec![0x80; 0x40];
        haystack_high[0x3E] = 0xFE;
        haystack_high[0x3F] = 0xFF;
        assert_eq!(search_fn(&haystack_high, [0x7F, 0x80]), None);
        assert_eq!(search_fn(&haystack_high, [0xFE, 0xFF]), Some(0x3E));

        let mut haystack_null = vec![0xFF; 0x50];
        haystack_null[0x2A] = 0x00;
        haystack_null[0x2B] = 0x00;
        assert_eq!(search_fn(&haystack_null, [0x00, 0x00]), Some(0x2A));

        assert_eq!(search_fn(b"\x01\x01\x01\x01\x01\x01\x01\x01\x01", [0x01, 0x01]), Some(0x00));
        assert_eq!(search_fn(b"\x80\x80\x80\x80\x80\x80\x80\x80\x80", [0x80, 0x80]), Some(0x00));
        assert_eq!(search_fn(b"\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF", [0xFF, 0xFF]), Some(0x00));
        assert_eq!(search_fn(b"\x00\x00\x00\x00\x00\x00\x00\x00\x00", [0x00, 0x00]), Some(0x00));

        let buffer = [b'-'; 0x60];
        for offset in 0x01..0x08 {
            let mut h = buffer[offset..].to_vec();
            h[0x19] = b'Y';
            h[0x1A] = b'Z';
            assert_eq!(search_fn(&h, *b"YZ"), Some(0x19));

            let end_idx = h.len() - 0x02;
            h[end_idx] = b'K';
            h[end_idx + 0x01] = b'L';
            assert_eq!(search_fn(&h, *b"KL"), Some(end_idx));
        }

        let tail_lengths = [
            0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0E, 0x0F, 0x10, 0x11, 0x12,
            0x1F, 0x20, 0x21, 0x22, 0x2F, 0x30, 0x31, 0x3F, 0x40, 0x41, 0x7F, 0x80, 0x81, 0xFF,
            0x100, 0x101,
        ];
        for &len in &tail_lengths {
            let mut h = vec![b'-'; len];
            h[len - 0x02] = b'A';
            h[len - 0x01] = b'B';

            assert_eq!(
                search_fn(&h, *b"AB"),
                Some(len - 0x02),
                "Failed tail chunk fallback for length {}",
                len
            );
        }

        let mut haystack_multi = vec![b'-'; 0x100];
        haystack_multi[0x10] = b'M';
        haystack_multi[0x11] = b'N';
        haystack_multi[0x50] = b'M';
        haystack_multi[0x51] = b'N';
        assert_eq!(search_fn(&haystack_multi, *b"MN"), Some(0x10));

        let mut huge_haystack = vec![b'x'; 0x64 * 0x400];
        assert_eq!(search_fn(&huge_haystack, *b"YZ"), None);

        let last_pos = 0x64 * 0x400 - 0x02;
        huge_haystack[last_pos] = b'Y';
        huge_haystack[last_pos + 0x01] = b'Z';
        assert_eq!(search_fn(&huge_haystack, *b"YZ"), Some(last_pos));
    }

    #[test]
    fn test_public_api() {
        run_standard_suite(search_two);
    }

    #[test]
    #[cfg(any(target_pointer_width = "64", test))]
    fn test_swar64_directly() {
        run_standard_suite(search_two_swar64);
    }

    #[test]
    #[cfg(any(target_pointer_width = "32", test))]
    fn test_swar32_directly() {
        run_standard_suite(search_two_swar32);
    }

    #[test]
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    fn test_sse2_directly() {
        if std::is_x86_feature_detected!("sse2") {
            run_standard_suite(|h, n| unsafe { search_two_sse2(h, n) });
        }
    }

    #[test]
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    fn test_ssse3_directly() {
        if std::is_x86_feature_detected!("ssse3") {
            run_standard_suite(|h, n| unsafe { search_two_ssse3(h, n) });
        }
    }

    #[test]
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    fn test_sse42_directly() {
        if std::is_x86_feature_detected!("sse4.2") {
            run_standard_suite(|h, n| unsafe { search_two_sse42(h, n) });
        }
    }

    #[test]
    #[cfg(target_arch = "x86_64")]
    fn test_avx2_directly() {
        if std::is_x86_feature_detected!("avx2") {
            run_standard_suite(|h, n| unsafe { search_two_avx2(h, n) });
        }
    }

    #[test]
    #[cfg(all(target_arch = "x86_64", target_feature = "avx512bw"))]
    fn test_avx512_directly() {
        if std::is_x86_feature_detected!("avx512bw") {
            run_standard_suite(|h, n| unsafe { search_two_avx512(h, n) });
        }
    }

    #[test]
    #[cfg(target_arch = "aarch64")]
    fn test_neon_aarch64_directly() {
        run_standard_suite(|h, n| unsafe { search_two_neon(h, n) });
    }

    #[test]
    #[cfg(all(target_arch = "arm", target_feature = "neon"))]
    fn test_neon_arm32_directly() {
        run_standard_suite(|h, n| unsafe { search_two_neon(h, n) });
    }

    #[test]
    #[cfg(all(target_arch = "wasm32", target_feature = "simd128"))]
    fn test_simd128_directly() {
        run_standard_suite(|h, n| unsafe { search_two_simd128(h, n) });
    }
}
