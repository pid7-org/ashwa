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
use crate::common::search_one_swar64;

#[cfg(any(target_pointer_width = "32", test))]
use crate::common::search_one_swar32;

/// Searches for the first occurrence of a single byte needle in a byte slice haystack
///
/// ## Example
///
/// ```
/// use ashwa::search_one;
///
/// let haystack = b"hello world";
/// assert_eq!(search_one(haystack, b'e'), Some(1));
/// assert_eq!(search_one(haystack, b'o'), Some(4));
/// assert_eq!(search_one(haystack, b'z'), None);
/// ```
#[inline(always)]
#[cfg(target_pointer_width = "64")]
pub fn search_one(haystack: &[u8], needle: u8) -> Option<usize> {
    #[cfg(target_arch = "x86_64")]
    match get_cpu_feature() {
        ISA::AVX2 => unsafe { search_one_avx2(haystack, needle) },
        ISA::SSE2 | ISA::SSSE3 | ISA::SSE4_2 => unsafe { search_one_sse2(haystack, needle) },
        ISA::AVX512BW => unsafe { search_one_avx512(haystack, needle) },
        _ => search_one_swar64(haystack, needle),
    }

    #[cfg(target_arch = "aarch64")]
    {
        #[cfg(forced_swar_backend)]
        return search_one_swar64(haystack, needle);

        unsafe { search_one_neon(haystack, needle) }
    }
}

/// Searches for the first occurrence of a single byte needle in a byte slice haystack
///
/// ## Example
///
/// ```
/// use ashwa::search_one;
///
/// let haystack = b"hello world";
/// assert_eq!(search_one(haystack, b'e'), Some(1));
/// assert_eq!(search_one(haystack, b'o'), Some(4));
/// assert_eq!(search_one(haystack, b'z'), None);
/// ```
#[cfg(target_pointer_width = "32")]
pub fn search_one(haystack: &[u8], needle: u8) -> Option<usize> {
    #[cfg(target_arch = "wasm32")]
    {
        #[cfg(target_feature = "simd128")]
        return unsafe { search_one_simd128(haystack, needle) };

        search_one_swar32(haystack, needle)
    }

    #[cfg(target_arch = "arm")]
    {
        #[cfg(target_feature = "neon")]
        return unsafe { search_one_neon(haystack, needle) };

        search_one_swar32(haystack, needle)
    }

    #[cfg(target_arch = "x86")]
    {
        #[cfg(target_feature = "sse2")]
        return unsafe { search_one_sse2(haystack, needle) };

        search_one_swar32(haystack, needle)
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn search_one_avx2(haystack: &[u8], needle: u8) -> Option<usize> {
    let v_needle = _mm256_set1_epi8(needle as i8);

    let mut i = 0;
    let len = haystack.len();
    let ptr = haystack.as_ptr();

    while i + 0x40 <= len {
        let v1 = _mm256_loadu_si256(ptr.add(i) as *const __m256i);
        let v2 = _mm256_loadu_si256(ptr.add(i + 0x20) as *const __m256i);

        let eq1 = _mm256_cmpeq_epi8(v1, v_needle);
        let eq2 = _mm256_cmpeq_epi8(v2, v_needle);

        let or_vec = _mm256_or_si256(eq1, eq2);
        if _mm256_movemask_epi8(or_vec) != 0 {
            let m1 = _mm256_movemask_epi8(eq1);
            if m1 != 0 {
                return Some(i + m1.trailing_zeros() as usize);
            }

            let m2 = _mm256_movemask_epi8(eq2);
            return Some(i + 0x20 + m2.trailing_zeros() as usize);
        }

        i += 0x40;
    }

    if i + 0x20 <= len {
        let v = _mm256_loadu_si256(ptr.add(i) as *const __m256i);
        let eq = _mm256_cmpeq_epi8(v, v_needle);
        let m = _mm256_movemask_epi8(eq);

        if m != 0 {
            return Some(i + m.trailing_zeros() as usize);
        }

        i += 0x20;
    }

    haystack[i..].iter().position(|&b| b == needle).map(|pos| pos + i)
}

#[target_feature(enable = "sse2")]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
unsafe fn search_one_sse2(haystack: &[u8], needle: u8) -> Option<usize> {
    let v_needle = _mm_set1_epi8(needle as i8);

    let mut i = 0;
    let len = haystack.len();
    let ptr = haystack.as_ptr();

    while i + 0x40 <= len {
        let v1 = _mm_loadu_si128(ptr.add(i) as *const __m128i);
        let v2 = _mm_loadu_si128(ptr.add(i + 0x10) as *const __m128i);
        let v3 = _mm_loadu_si128(ptr.add(i + 0x20) as *const __m128i);
        let v4 = _mm_loadu_si128(ptr.add(i + 0x30) as *const __m128i);

        let eq1 = _mm_cmpeq_epi8(v1, v_needle);
        let eq2 = _mm_cmpeq_epi8(v2, v_needle);
        let eq3 = _mm_cmpeq_epi8(v3, v_needle);
        let eq4 = _mm_cmpeq_epi8(v4, v_needle);

        let or1 = _mm_or_si128(eq1, eq2);
        let or2 = _mm_or_si128(eq3, eq4);
        let or_vec = _mm_or_si128(or1, or2);

        if _mm_movemask_epi8(or_vec) != 0 {
            let m1 = _mm_movemask_epi8(eq1);
            if m1 != 0 {
                return Some(i + m1.trailing_zeros() as usize);
            }

            let m2 = _mm_movemask_epi8(eq2);
            if m2 != 0 {
                return Some(i + 0x10 + m2.trailing_zeros() as usize);
            }

            let m3 = _mm_movemask_epi8(eq3);
            if m3 != 0 {
                return Some(i + 0x20 + m3.trailing_zeros() as usize);
            }

            let m4 = _mm_movemask_epi8(eq4);
            return Some(i + 0x30 + m4.trailing_zeros() as usize);
        }

        i += 0x40;
    }

    while i + 0x10 <= len {
        let v = _mm_loadu_si128(ptr.add(i) as *const __m128i);
        let eq = _mm_cmpeq_epi8(v, v_needle);
        let m = _mm_movemask_epi8(eq);

        if m != 0 {
            return Some(i + m.trailing_zeros() as usize);
        }

        i += 0x10;
    }

    haystack[i..].iter().position(|&b| b == needle).map(|pos| pos + i)
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512bw")]
unsafe fn search_one_avx512(haystack: &[u8], needle: u8) -> Option<usize> {
    let v_needle = _mm512_set1_epi8(needle as i8);

    let mut i = 0;
    let len = haystack.len();
    let ptr = haystack.as_ptr();

    while i + 0x80 <= len {
        let v1 = _mm512_loadu_si512(ptr.add(i) as *const _);
        let v2 = _mm512_loadu_si512(ptr.add(i + 0x40) as *const _);

        let eq1 = _mm512_cmpeq_epi8_mask(v1, v_needle);
        let eq2 = _mm512_cmpeq_epi8_mask(v2, v_needle);

        if eq1 != 0 {
            return Some(i + eq1.trailing_zeros() as usize);
        }

        if eq2 != 0 {
            return Some(i + 0x40 + eq2.trailing_zeros() as usize);
        }

        i += 0x80;
    }

    if i + 0x40 <= len {
        let v = _mm512_loadu_si512(ptr.add(i) as *const _);
        let eq = _mm512_cmpeq_epi8_mask(v, v_needle);

        if eq != 0 {
            return Some(i + eq.trailing_zeros() as usize);
        }

        i += 0x40;
    }

    haystack[i..].iter().position(|&b| b == needle).map(|pos| pos + i)
}

#[inline(always)]
#[cfg(target_arch = "aarch64")]
unsafe fn search_one_neon(haystack: &[u8], needle: u8) -> Option<usize> {
    #[inline(always)]
    unsafe fn has_match(v: uint8x16_t) -> bool {
        let u = vreinterpretq_u64_u8(v);
        (vgetq_lane_u64(u, 0) | vgetq_lane_u64(u, 1)) != 0
    }

    let v_needle = vdupq_n_u8(needle);

    let mut i = 0;
    let len = haystack.len();
    let ptr = haystack.as_ptr();

    while i + 0x100 <= len {
        let v0 = vld1q_u8(ptr.add(i));
        let v1 = vld1q_u8(ptr.add(i + 0x10));
        let v2 = vld1q_u8(ptr.add(i + 0x20));
        let v3 = vld1q_u8(ptr.add(i + 0x30));
        let v4 = vld1q_u8(ptr.add(i + 0x40));
        let v5 = vld1q_u8(ptr.add(i + 0x50));
        let v6 = vld1q_u8(ptr.add(i + 0x60));
        let v7 = vld1q_u8(ptr.add(i + 0x70));
        let v8 = vld1q_u8(ptr.add(i + 0x80));
        let v9 = vld1q_u8(ptr.add(i + 0x90));
        let v10 = vld1q_u8(ptr.add(i + 0xA0));
        let v11 = vld1q_u8(ptr.add(i + 0xB0));
        let v12 = vld1q_u8(ptr.add(i + 0xC0));
        let v13 = vld1q_u8(ptr.add(i + 0xD0));
        let v14 = vld1q_u8(ptr.add(i + 0xE0));
        let v15 = vld1q_u8(ptr.add(i + 0xF0));

        let eq0 = vceqq_u8(v0, v_needle);
        let eq1 = vceqq_u8(v1, v_needle);
        let eq2 = vceqq_u8(v2, v_needle);
        let eq3 = vceqq_u8(v3, v_needle);
        let eq4 = vceqq_u8(v4, v_needle);
        let eq5 = vceqq_u8(v5, v_needle);
        let eq6 = vceqq_u8(v6, v_needle);
        let eq7 = vceqq_u8(v7, v_needle);
        let eq8 = vceqq_u8(v8, v_needle);
        let eq9 = vceqq_u8(v9, v_needle);
        let eq10 = vceqq_u8(v10, v_needle);
        let eq11 = vceqq_u8(v11, v_needle);
        let eq12 = vceqq_u8(v12, v_needle);
        let eq13 = vceqq_u8(v13, v_needle);
        let eq14 = vceqq_u8(v14, v_needle);
        let eq15 = vceqq_u8(v15, v_needle);

        let or01 = vorrq_u8(eq0, eq1);
        let or23 = vorrq_u8(eq2, eq3);
        let or45 = vorrq_u8(eq4, eq5);
        let or67 = vorrq_u8(eq6, eq7);
        let or89 = vorrq_u8(eq8, eq9);
        let or1011 = vorrq_u8(eq10, eq11);
        let or1213 = vorrq_u8(eq12, eq13);
        let or1415 = vorrq_u8(eq14, eq15);

        let or0_3 = vorrq_u8(or01, or23);
        let or4_7 = vorrq_u8(or45, or67);
        let or8_11 = vorrq_u8(or89, or1011);
        let or12_15 = vorrq_u8(or1213, or1415);

        let or0_7 = vorrq_u8(or0_3, or4_7);
        let or8_15 = vorrq_u8(or8_11, or12_15);

        let or_all = vorrq_u8(or0_7, or8_15);

        if has_match(or_all) {
            if has_match(or0_7) {
                if has_match(or0_3) {
                    if has_match(or01) {
                        if has_match(eq0) {
                            return Some(i + get_match_index_neon(eq0));
                        }

                        return Some(i + 0x10 + get_match_index_neon(eq1));
                    }

                    if has_match(eq2) {
                        return Some(i + 0x20 + get_match_index_neon(eq2));
                    }

                    return Some(i + 0x30 + get_match_index_neon(eq3));
                }

                if has_match(or45) {
                    if has_match(eq4) {
                        return Some(i + 0x40 + get_match_index_neon(eq4));
                    }

                    return Some(i + 0x50 + get_match_index_neon(eq5));
                }

                if has_match(eq6) {
                    return Some(i + 0x60 + get_match_index_neon(eq6));
                }

                return Some(i + 0x70 + get_match_index_neon(eq7));
            }

            if has_match(or8_11) {
                if has_match(or89) {
                    if has_match(eq8) {
                        return Some(i + 0x80 + get_match_index_neon(eq8));
                    }

                    return Some(i + 0x90 + get_match_index_neon(eq9));
                }

                if has_match(eq10) {
                    return Some(i + 0xA0 + get_match_index_neon(eq10));
                }

                return Some(i + 0xB0 + get_match_index_neon(eq11));
            }

            if has_match(or1213) {
                if has_match(eq12) {
                    return Some(i + 0xC0 + get_match_index_neon(eq12));
                }

                return Some(i + 0xD0 + get_match_index_neon(eq13));
            }

            if has_match(eq14) {
                return Some(i + 0xE0 + get_match_index_neon(eq14));
            }

            return Some(i + 0xF0 + get_match_index_neon(eq15));
        }

        i += 0x100;
    }

    if i + 0x80 <= len {
        let v0 = vld1q_u8(ptr.add(i));
        let v1 = vld1q_u8(ptr.add(i + 0x10));
        let v2 = vld1q_u8(ptr.add(i + 0x20));
        let v3 = vld1q_u8(ptr.add(i + 0x30));
        let v4 = vld1q_u8(ptr.add(i + 0x40));
        let v5 = vld1q_u8(ptr.add(i + 0x50));
        let v6 = vld1q_u8(ptr.add(i + 0x60));
        let v7 = vld1q_u8(ptr.add(i + 0x70));

        let eq0 = vceqq_u8(v0, v_needle);
        let eq1 = vceqq_u8(v1, v_needle);
        let eq2 = vceqq_u8(v2, v_needle);
        let eq3 = vceqq_u8(v3, v_needle);
        let eq4 = vceqq_u8(v4, v_needle);
        let eq5 = vceqq_u8(v5, v_needle);
        let eq6 = vceqq_u8(v6, v_needle);
        let eq7 = vceqq_u8(v7, v_needle);

        let or01 = vorrq_u8(eq0, eq1);
        let or23 = vorrq_u8(eq2, eq3);
        let or45 = vorrq_u8(eq4, eq5);
        let or67 = vorrq_u8(eq6, eq7);

        let or0_3 = vorrq_u8(or01, or23);
        let or4_7 = vorrq_u8(or45, or67);
        let or_all = vorrq_u8(or0_3, or4_7);

        if has_match(or_all) {
            if has_match(or0_3) {
                if has_match(or01) {
                    if has_match(eq0) {
                        return Some(i + get_match_index_neon(eq0));
                    }

                    return Some(i + 0x10 + get_match_index_neon(eq1));
                }

                if has_match(eq2) {
                    return Some(i + 0x20 + get_match_index_neon(eq2));
                }

                return Some(i + 0x30 + get_match_index_neon(eq3));
            }

            if has_match(or45) {
                if has_match(eq4) {
                    return Some(i + 0x40 + get_match_index_neon(eq4));
                }

                return Some(i + 0x50 + get_match_index_neon(eq5));
            }

            if has_match(eq6) {
                return Some(i + 0x60 + get_match_index_neon(eq6));
            }

            return Some(i + 0x70 + get_match_index_neon(eq7));
        }

        i += 0x80;
    }

    if i + 0x40 <= len {
        let v0 = vld1q_u8(ptr.add(i));
        let v1 = vld1q_u8(ptr.add(i + 0x10));
        let v2 = vld1q_u8(ptr.add(i + 0x20));
        let v3 = vld1q_u8(ptr.add(i + 0x30));

        let eq0 = vceqq_u8(v0, v_needle);
        let eq1 = vceqq_u8(v1, v_needle);
        let eq2 = vceqq_u8(v2, v_needle);
        let eq3 = vceqq_u8(v3, v_needle);

        let or01 = vorrq_u8(eq0, eq1);
        let or23 = vorrq_u8(eq2, eq3);
        let or_all = vorrq_u8(or01, or23);

        if has_match(or_all) {
            if has_match(or01) {
                if has_match(eq0) {
                    return Some(i + get_match_index_neon(eq0));
                }

                return Some(i + 0x10 + get_match_index_neon(eq1));
            }

            if has_match(eq2) {
                return Some(i + 0x20 + get_match_index_neon(eq2));
            }

            return Some(i + 0x30 + get_match_index_neon(eq3));
        }

        i += 0x40;
    }

    if i + 0x20 <= len {
        let v0 = vld1q_u8(ptr.add(i));
        let v1 = vld1q_u8(ptr.add(i + 0x10));

        let eq0 = vceqq_u8(v0, v_needle);
        let eq1 = vceqq_u8(v1, v_needle);

        let or_all = vorrq_u8(eq0, eq1);

        if has_match(or_all) {
            if has_match(eq0) {
                return Some(i + get_match_index_neon(eq0));
            }

            return Some(i + 0x10 + get_match_index_neon(eq1));
        }

        i += 0x20;
    }

    if i + 0x10 <= len {
        let v0 = vld1q_u8(ptr.add(i));
        let eq0 = vceqq_u8(v0, v_needle);

        if has_match(eq0) {
            return Some(i + get_match_index_neon(eq0));
        }

        i += 0x10;
    }

    haystack[i..].iter().position(|&b| b == needle).map(|pos| pos + i)
}

#[inline(always)]
#[cfg(target_arch = "aarch64")]
unsafe fn get_match_index_neon(eq: uint8x16_t) -> usize {
    let eq_u64 = vreinterpretq_u64_u8(eq);
    let lane0 = vgetq_lane_u64(eq_u64, 0);

    if lane0 != 0 {
        return (lane0.trailing_zeros() / 8) as usize;
    }

    let lane1 = vgetq_lane_u64(eq_u64, 1);
    8 + (lane1.trailing_zeros() / 8) as usize
}

#[target_feature(enable = "neon")]
#[cfg(all(target_arch = "arm", target_feature = "neon", target_pointer_width = "32"))]
unsafe fn search_one_neon(haystack: &[u8], needle: u8) -> Option<usize> {
    #[inline(always)]
    unsafe fn any_match(v: uint8x16_t) -> bool {
        let lanes = vreinterpretq_u64_u8(v);
        vgetq_lane_u64(lanes, 0) != 0 || vgetq_lane_u64(lanes, 1) != 0
    }

    let v_needle = vdupq_n_u8(needle);

    let mut i = 0;
    let len = haystack.len();
    let ptr = haystack.as_ptr();

    while i + 0x40 <= len {
        let v1 = vld1q_u8(ptr.add(i));
        let v2 = vld1q_u8(ptr.add(i + 0x10));
        let v3 = vld1q_u8(ptr.add(i + 0x20));
        let v4 = vld1q_u8(ptr.add(i + 0x30));

        let eq1 = vceqq_u8(v1, v_needle);
        let eq2 = vceqq_u8(v2, v_needle);
        let eq3 = vceqq_u8(v3, v_needle);
        let eq4 = vceqq_u8(v4, v_needle);

        let or1 = vorrq_u8(eq1, eq2);
        let or2 = vorrq_u8(eq3, eq4);
        let or_vec = vorrq_u8(or1, or2);

        if any_match(or_vec) {
            if any_match(eq1) {
                return Some(i + get_match_index_neon(eq1));
            }

            if any_match(eq2) {
                return Some(i + 0x10 + get_match_index_neon(eq2));
            }

            if any_match(eq3) {
                return Some(i + 0x20 + get_match_index_neon(eq3));
            }

            return Some(i + 0x30 + get_match_index_neon(eq4));
        }

        i += 0x40;
    }

    while i + 0x10 <= len {
        let v = vld1q_u8(ptr.add(i));
        let eq = vceqq_u8(v, v_needle);

        if any_match(eq) {
            return Some(i + get_match_index_neon(eq));
        }

        i += 0x10;
    }

    haystack[i..].iter().position(|&b| b == needle).map(|pos| pos + i)
}

#[inline(always)]
#[cfg(all(target_arch = "arm", target_feature = "neon"))]
unsafe fn get_match_index_neon(eq: uint8x16_t) -> usize {
    let eq_u64 = vreinterpretq_u64_u8(eq);

    let lane0 = vgetq_lane_u64(eq_u64, 0);
    if lane0 != 0 {
        return (lane0.trailing_zeros() / 8) as usize;
    }

    let lane1 = vgetq_lane_u64(eq_u64, 1);
    8 + (lane1.trailing_zeros() / 8) as usize
}

#[target_feature(enable = "simd128")]
#[cfg(all(target_arch = "wasm32", target_feature = "simd128"))]
unsafe fn search_one_simd128(haystack: &[u8], needle: u8) -> Option<usize> {
    let v_needle = u8x16_splat(needle);

    let mut i = 0;
    let len = haystack.len();
    let ptr = haystack.as_ptr();

    while i + 0x40 <= len {
        let v1 = v128_load(ptr.add(i) as *const v128);
        let v2 = v128_load(ptr.add(i + 0x10) as *const v128);
        let v3 = v128_load(ptr.add(i + 0x20) as *const v128);
        let v4 = v128_load(ptr.add(i + 0x30) as *const v128);

        let eq1 = u8x16_eq(v1, v_needle);
        let eq2 = u8x16_eq(v2, v_needle);
        let eq3 = u8x16_eq(v3, v_needle);
        let eq4 = u8x16_eq(v4, v_needle);

        let or1 = v128_or(eq1, eq2);
        let or2 = v128_or(eq3, eq4);
        let or_vec = v128_or(or1, or2);

        if u8x16_bitmask(or_vec) != 0 {
            let m1 = u8x16_bitmask(eq1);
            if m1 != 0 {
                return Some(i + m1.trailing_zeros() as usize);
            }

            let m2 = u8x16_bitmask(eq2);
            if m2 != 0 {
                return Some(i + 0x10 + m2.trailing_zeros() as usize);
            }

            let m3 = u8x16_bitmask(eq3);
            if m3 != 0 {
                return Some(i + 0x20 + m3.trailing_zeros() as usize);
            }

            let m4 = u8x16_bitmask(eq4);
            return Some(i + 0x30 + m4.trailing_zeros() as usize);
        }

        i += 0x40;
    }

    while i + 0x10 <= len {
        let v = v128_load(ptr.add(i) as *const v128);
        let eq = u8x16_eq(v, v_needle);
        let mask = u8x16_bitmask(eq);

        if mask != 0 {
            return Some(i + mask.trailing_zeros() as usize);
        }

        i += 0x10;
    }

    haystack[i..].iter().position(|&b| b == needle).map(|p| p + i)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_standard_suite(search_fn: impl Fn(&[u8], u8) -> Option<usize>) {
        assert_eq!(search_fn(b"", b'a'), None);

        let haystack = b"the quick brown fox jumps over the lazy dog";
        assert_eq!(search_fn(haystack, b'Z'), None);
        assert_eq!(search_fn(haystack, b'!'), None);
        assert_eq!(search_fn(haystack, b'o'), Some(0x0C));

        for len in 1..8 {
            let mut haystack = vec![b'x'; len];
            haystack[len - 1] = b'a';

            assert_eq!(search_fn(&haystack, b'a'), Some(len - 1));
        }

        let mut h8 = [b'-'; 8];
        let mut h16 = [b'-'; 0x10];
        let mut h24 = [b'-'; 0x18];
        let mut h32 = [b'-'; 0x20];

        h8[7] = b'A';
        assert_eq!(search_fn(&h8, b'A'), Some(7));

        h16[0x0F] = b'B';
        assert_eq!(search_fn(&h16, b'B'), Some(0x0F));

        h24[0x17] = b'C';
        assert_eq!(search_fn(&h24, b'C'), Some(0x17));

        h32[0x1F] = b'D';
        assert_eq!(search_fn(&h32, b'D'), Some(0x1F));

        let mut haystack = vec![b'-'; 0x200];
        for i in 0..haystack.len() {
            haystack[i] = b'A';
            assert_eq!(search_fn(&haystack, b'A'), Some(i), "Failed finding needle at index {}", i);

            haystack[i] = b'-';
        }

        let mut haystack_high = vec![0x80; 0x40];
        haystack_high[0x3F] = 0xFF;

        assert_eq!(search_fn(&haystack_high, 0x7F), None);
        assert_eq!(search_fn(&haystack_high, 0xFF), Some(0x3F));

        let mut haystack_null = vec![0xFF; 0x50];
        haystack_null[0x2A] = 0x00;

        assert_eq!(search_fn(&haystack_null, 0x00), Some(0x2A));
        assert_eq!(search_fn(b"\x01\x01\x01\x01\x01\x01\x01\x01", 0x01), Some(0));
        assert_eq!(search_fn(b"\x80\x80\x80\x80\x80\x80\x80\x80", 0x80), Some(0));

        let buffer = [b'-'; 0x60];
        for offset in 1..8 {
            let mut haystack = buffer[offset..].to_vec();
            haystack[0x19] = b'Z';

            assert_eq!(search_fn(&haystack, b'Z'), Some(0x19));

            let end_idx = haystack.len() - 2;
            haystack[end_idx] = b'Y';

            assert_eq!(search_fn(&haystack, b'Y'), Some(end_idx));
        }

        let tail_lengths = [0x09, 0x0F, 0x11, 0x1F, 0x21, 0x3F, 0x41, 0x7F, 0x81, 0xFF, 0x101];
        for &len in &tail_lengths {
            let mut haystack = vec![b'-'; len];
            haystack[len - 1] = b'A';

            assert_eq!(
                search_fn(&haystack, b'A'),
                Some(len - 1),
                "Failed tail chunk fallback for length {}",
                len
            );
        }

        let mut haystack_lanes = vec![b'-'; 0x100];
        for i in (0..0x100).step_by(16) {
            haystack_lanes[i] = b'*';
        }

        assert_eq!(search_fn(&haystack_lanes, b'*'), Some(0));

        haystack_lanes[0] = b'-';
        assert_eq!(search_fn(&haystack_lanes, b'*'), Some(0x10));

        let mut huge_haystack = vec![b'x'; 0x64 * 0x400];
        assert_eq!(search_fn(&huge_haystack, b'Z'), None);

        huge_haystack[0x64 * 0x400 - 1] = b'Z';
        assert_eq!(search_fn(&huge_haystack, b'Z'), Some(0x64 * 0x400 - 1));
    }

    #[test]
    fn test_public_api() {
        run_standard_suite(search_one);
    }

    #[test]
    #[cfg(target_pointer_width = "64")]
    fn test_swar64_directly() {
        run_standard_suite(search_one_swar64);
    }

    #[test]
    #[cfg(any(target_pointer_width = "32", test))]
    fn test_swar32_directly() {
        run_standard_suite(search_one_swar32);
    }

    #[test]
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    fn test_sse2_directly() {
        if std::is_x86_feature_detected!("sse2") {
            run_standard_suite(|h, n| unsafe { search_one_sse2(h, n) });
        }
    }

    #[test]
    #[cfg(target_arch = "x86_64")]
    fn test_avx2_directly() {
        if std::is_x86_feature_detected!("avx2") {
            run_standard_suite(|h, n| unsafe { search_one_avx2(h, n) });
        }
    }

    #[test]
    #[cfg(target_arch = "x86_64")]
    fn test_avx512_directly() {
        if std::is_x86_feature_detected!("avx512bw") {
            run_standard_suite(|h, n| unsafe { search_one_avx512(h, n) });
        }
    }

    #[test]
    #[cfg(target_arch = "aarch64")]
    fn test_neon_aarch64_directly() {
        run_standard_suite(|h, n| unsafe { search_one_neon(h, n) });
    }

    #[test]
    #[cfg(all(target_arch = "arm", target_feature = "neon"))]
    fn test_neon_arm32_directly() {
        run_standard_suite(|h, n| unsafe { search_one_neon(h, n) });
    }

    #[test]
    #[cfg(all(target_arch = "wasm32", target_feature = "simd128"))]
    fn test_wasm_simd128_directly() {
        run_standard_suite(|h, n| unsafe { search_one_simd128(h, n) });
    }
}
