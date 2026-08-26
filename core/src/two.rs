//! Two-byte needle search implementation with hardware-accelerated SIMD and SWAR routines
//!
//! ## Example
//!
//! ```
//! use ashwa::search_two;
//!
//! let text = b"The quick brown fox jumps over the lazy dog";
//! assert_eq!(search_two(text, *b"qu"), Some(4));
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

#[cfg(any(target_pointer_width = "64", test))]
use crate::common::{get_match_index_64, match_qword, LSB64, MSB64};

#[cfg(any(target_pointer_width = "32", test))]
use crate::common::{get_match_index_32, match_dword, LSB32, MSB32};

/// Searches for the first occurrence of a two-byte needle in a byte slice (`haystack`)
///
/// ## Example
///
/// ```
/// use ashwa::search_two;
///
/// let haystack = b"hello world";
/// assert_eq!(search_two(haystack, *b"el"), Some(1));
/// assert_eq!(search_two(haystack, *b"ld"), Some(9));
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
        ISA::AVX2 | ISA::AVX512BW => unsafe { search_two_avx2(haystack, needle) },
        _ => unreachable!(),
    }

    #[cfg(target_arch = "aarch64")]
    {
        #[cfg(forced_swar_backend)]
        return search_two_swar64(haystack, needle);

        search_two_swar64(haystack, needle)
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    {
        search_two_swar64(haystack, needle)
    }
}

/// Searches for the first occurrence of a two-byte needle in a byte slice (`haystack`)
///
/// ## Example
///
/// ```
/// use ashwa::search_two;
///
/// let haystack = b"hello world";
/// assert_eq!(search_two(haystack, *b"el"), Some(1));
/// assert_eq!(search_two(haystack, *b"ld"), Some(9));
/// assert_eq!(search_two(haystack, *b"zz"), None);
/// ```
#[cfg(target_pointer_width = "32")]
pub fn search_two(haystack: &[u8], needle: [u8; 0x02]) -> Option<usize> {
    #[cfg(target_arch = "wasm32")]
    {
        search_two_swar32(haystack, needle)
    }

    #[cfg(target_arch = "arm")]
    {
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

/// 64-bit SWAR implementation of two-byte needle search
#[inline(always)]
#[cfg(any(target_pointer_width = "64", test))]
pub fn search_two_swar64(haystack: &[u8], needle: [u8; 0x02]) -> Option<usize> {
    let needle_a = (needle[0x00] as u64).wrapping_mul(LSB64);
    let needle_b = (needle[0x01] as u64).wrapping_mul(LSB64);

    let mut i = 0x00;
    let len = haystack.len();
    let ptr = haystack.as_ptr();

    // Process 4 words (32 bytes) at a time. Requires 33 bytes to safely read offset by +1.
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

    // Process 1 word (8 bytes) at a time. Requires 9 bytes to safely read offset by +1.
    while i + 0x09 <= len {
        let w_a = unsafe { ptr::read_unaligned(ptr.add(i) as *const u64) };
        let w_b = unsafe { ptr::read_unaligned(ptr.add(i + 0x01) as *const u64) };

        let m = match_qword(w_a, needle_a) & match_qword(w_b, needle_b);

        if m != 0x00 {
            return Some(i + get_match_index_64(m));
        }

        i += 0x08;
    }

    // Fallback for the remaining tail chunk
    haystack[i..].windows(0x02).position(|w| w == needle).map(|pos| pos + i)
}

/// 32-bit SWAR implementation of two-byte needle search
#[inline(always)]
#[cfg(any(target_pointer_width = "32", test))]
pub fn search_two_swar32(haystack: &[u8], needle: [u8; 0x02]) -> Option<usize> {
    let needle_a = (needle[0x00] as u32).wrapping_mul(LSB32);
    let needle_b = (needle[0x01] as u32).wrapping_mul(LSB32);

    let mut i = 0x00;
    let len = haystack.len();
    let ptr = haystack.as_ptr();

    // Process 4 words (16 bytes) at a time. Requires 17 bytes to safely read offset by +1.
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

    // Process 1 word (4 bytes) at a time. Requires 5 bytes to safely read offset by +1.
    while i + 0x05 <= len {
        let w_a = unsafe { ptr::read_unaligned(ptr.add(i) as *const u32) };
        let w_b = unsafe { ptr::read_unaligned(ptr.add(i + 0x01) as *const u32) };

        let m = match_dword(w_a, needle_a) & match_dword(w_b, needle_b);

        if m != 0x00 {
            return Some(i + get_match_index_32(m));
        }

        i += 0x04;
    }

    // Fallback for the remaining tail chunk
    haystack[i..].windows(0x02).position(|w| w == needle).map(|pos| pos + i)
}

/// SSE2 implementation of two-byte needle search
#[target_feature(enable = "sse2")]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub unsafe fn search_two_sse2(haystack: &[u8], needle: [u8; 0x02]) -> Option<usize> {
    let v_needle_a = _mm_set1_epi8(needle[0x00] as i8);
    let v_needle_b = _mm_set1_epi8(needle[0x01] as i8);

    let mut i = 0x00;
    let len = haystack.len();
    let ptr = haystack.as_ptr();

    // Process 4 vectors (64 bytes) at a time. Requires 65 bytes to safely read offset by +1.
    while i + 0x41 <= len {
        let v1_a = _mm_loadu_si128(ptr.add(i) as *const __m128i);
        let v1_b = _mm_loadu_si128(ptr.add(i + 0x01) as *const __m128i);
        let v2_a = _mm_loadu_si128(ptr.add(i + 0x10) as *const __m128i);
        let v2_b = _mm_loadu_si128(ptr.add(i + 0x11) as *const __m128i);
        let v3_a = _mm_loadu_si128(ptr.add(i + 0x20) as *const __m128i);
        let v3_b = _mm_loadu_si128(ptr.add(i + 0x21) as *const __m128i);
        let v4_a = _mm_loadu_si128(ptr.add(i + 0x30) as *const __m128i);
        let v4_b = _mm_loadu_si128(ptr.add(i + 0x31) as *const __m128i);

        let eq1 = _mm_and_si128(
            _mm_cmpeq_epi8(v1_a, v_needle_a),
            _mm_cmpeq_epi8(v1_b, v_needle_b),
        );
        let eq2 = _mm_and_si128(
            _mm_cmpeq_epi8(v2_a, v_needle_a),
            _mm_cmpeq_epi8(v2_b, v_needle_b),
        );
        let eq3 = _mm_and_si128(
            _mm_cmpeq_epi8(v3_a, v_needle_a),
            _mm_cmpeq_epi8(v3_b, v_needle_b),
        );
        let eq4 = _mm_and_si128(
            _mm_cmpeq_epi8(v4_a, v_needle_a),
            _mm_cmpeq_epi8(v4_b, v_needle_b),
        );

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

    // Process 1 vector (16 bytes) at a time. Requires 17 bytes to safely read offset by +1.
    while i + 0x11 <= len {
        let v_a = _mm_loadu_si128(ptr.add(i) as *const __m128i);
        let v_b = _mm_loadu_si128(ptr.add(i + 0x01) as *const __m128i);

        let eq = _mm_and_si128(
            _mm_cmpeq_epi8(v_a, v_needle_a),
            _mm_cmpeq_epi8(v_b, v_needle_b),
        );
        let m = _mm_movemask_epi8(eq);

        if m != 0x00 {
            return Some(i + m.trailing_zeros() as usize);
        }

        i += 0x10;
    }

    // Fallback for the remaining tail chunk
    haystack[i..].windows(0x02).position(|w| w == needle).map(|pos| pos + i)
}

/// SSSE3 implementation of two-byte needle search using `_mm_alignr_epi8`
#[target_feature(enable = "ssse3")]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub unsafe fn search_two_ssse3(haystack: &[u8], needle: [u8; 0x02]) -> Option<usize> {
    let v_needle_a = _mm_set1_epi8(needle[0x00] as i8);
    let v_needle_b = _mm_set1_epi8(needle[0x01] as i8);

    let mut i = 0x00;
    let len = haystack.len();
    let ptr = haystack.as_ptr();

    // Process 4 vectors (64 bytes) at a time using `_mm_alignr_epi8` to synthesize +1 offsets in registers.
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

        let eq1 = _mm_and_si128(
            _mm_cmpeq_epi8(v1, v_needle_a),
            _mm_cmpeq_epi8(v1_b, v_needle_b),
        );
        let eq2 = _mm_and_si128(
            _mm_cmpeq_epi8(v2, v_needle_a),
            _mm_cmpeq_epi8(v2_b, v_needle_b),
        );
        let eq3 = _mm_and_si128(
            _mm_cmpeq_epi8(v3, v_needle_a),
            _mm_cmpeq_epi8(v3_b, v_needle_b),
        );
        let eq4 = _mm_and_si128(
            _mm_cmpeq_epi8(v4, v_needle_a),
            _mm_cmpeq_epi8(v4_b, v_needle_b),
        );

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

    // Process 1 vector (16 bytes) at a time. Requires 17 bytes to safely read offset by +1.
    while i + 0x11 <= len {
        let v_a = _mm_loadu_si128(ptr.add(i) as *const __m128i);
        let v_b = _mm_loadu_si128(ptr.add(i + 0x01) as *const __m128i);

        let eq = _mm_and_si128(
            _mm_cmpeq_epi8(v_a, v_needle_a),
            _mm_cmpeq_epi8(v_b, v_needle_b),
        );
        let m = _mm_movemask_epi8(eq);

        if m != 0x00 {
            return Some(i + m.trailing_zeros() as usize);
        }

        i += 0x10;
    }

    // Fallback for the remaining tail chunk
    haystack[i..].windows(0x02).position(|w| w == needle).map(|pos| pos + i)
}

/// SSE4.2 implementation of two-byte needle search using `_mm_alignr_epi8` and `_mm_testz_si128`
#[target_feature(enable = "sse4.2")]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub unsafe fn search_two_sse42(haystack: &[u8], needle: [u8; 0x02]) -> Option<usize> {
    let v_needle_a = _mm_set1_epi8(needle[0x00] as i8);
    let v_needle_b = _mm_set1_epi8(needle[0x01] as i8);

    let mut i = 0x00;
    let len = haystack.len();
    let ptr = haystack.as_ptr();

    // Process 4 vectors (64 bytes) at a time using `_mm_alignr_epi8` and vector test `_mm_testz_si128`.
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

        let eq1 = _mm_and_si128(
            _mm_cmpeq_epi8(v1, v_needle_a),
            _mm_cmpeq_epi8(v1_b, v_needle_b),
        );
        let eq2 = _mm_and_si128(
            _mm_cmpeq_epi8(v2, v_needle_a),
            _mm_cmpeq_epi8(v2_b, v_needle_b),
        );
        let eq3 = _mm_and_si128(
            _mm_cmpeq_epi8(v3, v_needle_a),
            _mm_cmpeq_epi8(v3_b, v_needle_b),
        );
        let eq4 = _mm_and_si128(
            _mm_cmpeq_epi8(v4, v_needle_a),
            _mm_cmpeq_epi8(v4_b, v_needle_b),
        );

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

    // Process 1 vector (16 bytes) at a time. Requires 17 bytes to safely read offset by +1.
    while i + 0x11 <= len {
        let v_a = _mm_loadu_si128(ptr.add(i) as *const __m128i);
        let v_b = _mm_loadu_si128(ptr.add(i + 0x01) as *const __m128i);

        let eq = _mm_and_si128(
            _mm_cmpeq_epi8(v_a, v_needle_a),
            _mm_cmpeq_epi8(v_b, v_needle_b),
        );

        if _mm_testz_si128(eq, eq) == 0x00 {
            let m = _mm_movemask_epi8(eq);
            return Some(i + m.trailing_zeros() as usize);
        }

        i += 0x10;
    }

    // Fallback for the remaining tail chunk
    haystack[i..].windows(0x02).position(|w| w == needle).map(|pos| pos + i)
}

/// AVX2 implementation of two-byte needle search
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
pub unsafe fn search_two_avx2(haystack: &[u8], needle: [u8; 0x02]) -> Option<usize> {
    let v_needle_a = _mm256_set1_epi8(needle[0x00] as i8);
    let v_needle_b = _mm256_set1_epi8(needle[0x01] as i8);

    let mut i = 0x00;
    let len = haystack.len();
    let ptr = haystack.as_ptr();

    // Process 2 vectors (64 bytes) at a time. Requires 65 bytes to safely read offset by +1.
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

    // Process 1 vector (32 bytes) at a time. Requires 33 bytes to safely read offset by +1.
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

    // Fallback for the remaining tail chunk
    haystack[i..].windows(0x02).position(|w| w == needle).map(|pos| pos + i)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_standard_suite(search_fn: impl Fn(&[u8], [u8; 0x02]) -> Option<usize>) {
        // Empty haystack
        assert_eq!(search_fn(b"", *b"ab"), None);

        // Sub-needle length haystacks
        assert_eq!(search_fn(b"a", *b"ab"), None);
        assert_eq!(search_fn(b"b", *b"ab"), None);

        // Exact match of length 2
        assert_eq!(search_fn(b"ab", *b"ab"), Some(0x00));
        assert_eq!(search_fn(b"ac", *b"ab"), None);
        assert_eq!(search_fn(b"ba", *b"ab"), None);

        // Sentence search
        let haystack = b"the quick brown fox jumps over the lazy dog";
        assert_eq!(search_fn(haystack, *b"ZZ"), None);
        assert_eq!(search_fn(haystack, *b"!!"), None);
        assert_eq!(search_fn(haystack, *b"th"), Some(0x00));
        assert_eq!(search_fn(haystack, *b"he"), Some(0x01));
        assert_eq!(search_fn(haystack, *b"qu"), Some(0x04));
        assert_eq!(search_fn(haystack, *b"ox"), Some(0x11));
        assert_eq!(search_fn(haystack, *b"do"), Some(0x28));
        assert_eq!(search_fn(haystack, *b"og"), Some(0x29));

        // Short haystacks: all positions in lengths 2..9
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

        // Exact block boundaries
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

        // Cross-word boundary positions (straddling 4-byte, 8-byte, and 32-byte chunks)
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

        // Exhaustive position sweep across a 512-byte buffer
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

        // False positive resistance: first byte matches everywhere, second byte does not
        let mut haystack_first_match = vec![b'A'; 0x100];
        assert_eq!(search_fn(&haystack_first_match, *b"AB"), None);
        haystack_first_match[0x7A] = b'B';
        assert_eq!(search_fn(&haystack_first_match, *b"AB"), Some(0x79));

        // False positive resistance: second byte matches everywhere, first byte does not
        let mut haystack_second_match = vec![b'B'; 0x100];
        assert_eq!(search_fn(&haystack_second_match, *b"AB"), None);
        haystack_second_match[0x40] = b'A';
        assert_eq!(search_fn(&haystack_second_match, *b"AB"), Some(0x40));

        // Repeating patterns and overlapping needles
        assert_eq!(search_fn(b"aaaaaa", *b"aa"), Some(0x00));
        assert_eq!(search_fn(b"baaaaa", *b"aa"), Some(0x01));
        assert_eq!(search_fn(b"bbaaaa", *b"aa"), Some(0x02));
        assert_eq!(search_fn(b"ababab", *b"ab"), Some(0x00));
        assert_eq!(search_fn(b"bababa", *b"ab"), Some(0x01));

        // High bit and binary edge cases
        let mut haystack_high = vec![0x80; 0x40];
        haystack_high[0x3E] = 0xFE;
        haystack_high[0x3F] = 0xFF;
        assert_eq!(search_fn(&haystack_high, [0x7F, 0x80]), None);
        assert_eq!(search_fn(&haystack_high, [0xFE, 0xFF]), Some(0x3E));

        let mut haystack_null = vec![0xFF; 0x50];
        haystack_null[0x2A] = 0x00;
        haystack_null[0x2B] = 0x00;
        assert_eq!(search_fn(&haystack_null, [0x00, 0x00]), Some(0x2A));

        assert_eq!(
            search_fn(b"\x01\x01\x01\x01\x01\x01\x01\x01\x01", [0x01, 0x01]),
            Some(0x00)
        );
        assert_eq!(
            search_fn(b"\x80\x80\x80\x80\x80\x80\x80\x80\x80", [0x80, 0x80]),
            Some(0x00)
        );
        assert_eq!(
            search_fn(b"\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF", [0xFF, 0xFF]),
            Some(0x00)
        );
        assert_eq!(
            search_fn(b"\x00\x00\x00\x00\x00\x00\x00\x00\x00", [0x00, 0x00]),
            Some(0x00)
        );

        // Buffer alignment offsets
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

        // Tail chunk lengths
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

        // Multiple occurrences in haystack (must return first)
        let mut haystack_multi = vec![b'-'; 0x100];
        haystack_multi[0x10] = b'M';
        haystack_multi[0x11] = b'N';
        haystack_multi[0x50] = b'M';
        haystack_multi[0x51] = b'N';
        assert_eq!(search_fn(&haystack_multi, *b"MN"), Some(0x10));

        // Huge haystack
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
}
