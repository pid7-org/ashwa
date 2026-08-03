use core::ptr;

#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

#[cfg(target_arch = "aarch64")]
use core::arch::aarch64::*;

#[cfg(target_pointer_width = "64")]
const LSB64: u64 = 0x0101_0101_0101_0101;

#[cfg(target_pointer_width = "64")]
const MSB64: u64 = 0x8080_8080_8080_8080;

#[cfg(target_pointer_width = "32")]
const LSB32: u32 = 0x0101_0101;

#[cfg(target_pointer_width = "32")]
const MSB32: u32 = 0x8080_8080;

#[cfg(target_pointer_width = "16")]
const LSB16: u16 = 0x0101;

#[cfg(target_pointer_width = "16")]
const MSB16: u16 = 0x8080;

#[inline(always)]
#[cfg(target_pointer_width = "64")]
pub fn search_one(haystack: &[u8], needle: u8) -> Option<usize> {
    #[cfg(target_arch = "x86_64")]
    match crate::get_cpu_feature() {
        1 => search_one_swar64(haystack, needle),
        2 | 3 | 4 => unsafe { search_one_sse2(haystack, needle) },
        5 => unsafe { search_one_avx2(haystack, needle) },
        6 => {
            #[cfg(not(target_feature = "avx512bw"))]
            return unsafe { search_one_avx2(haystack, needle) };

            #[cfg(target_feature = "avx512bw")]
            return unsafe { search_one_avx512(haystack, needle) };
        }
        _ => unreachable!(),
    }

    #[cfg(target_arch = "aarch64")]
    unsafe {
        search_one_neon(haystack, needle)
    }
}

#[inline(always)]
#[cfg(target_pointer_width = "32")]
pub fn search_one(haystack: &[u8], needle: u8) -> Option<usize> {
    let needle_word = (needle as u32).wrapping_mul(LSB32);

    let mut i = 0;
    let len = haystack.len();
    let ptr = haystack.as_ptr();

    while i + 8 <= len {
        let w1 = unsafe { ptr::read_unaligned(ptr.add(i) as *const u32) };
        let w2 = unsafe { ptr::read_unaligned(ptr.add(i + 4) as *const u32) };

        let m1 = match_dword(w1, needle_word);
        let m2 = match_dword(w2, needle_word);

        if (m1 | m2) != 0 {
            if m1 != 0 {
                return Some(i + get_match_index_32(m1));
            }

            return Some(i + 4 + get_match_index_32(m2));
        }

        i += 8;
    }

    if i + 4 <= len {
        let w1 = unsafe { ptr::read_unaligned(ptr.add(i) as *const u32) };
        let m1 = match_dword(w1, needle_word);

        if m1 != 0 {
            return Some(i + get_match_index_32(m1));
        }

        i += 4;
    }

    haystack[i..].iter().position(|&b| b == needle).map(|pos| pos + i)
}

#[inline(always)]
#[cfg(target_pointer_width = "16")]
pub fn search_one(haystack: &[u8], needle: u8) -> Option<usize> {
    let needle_word = (needle as u16).wrapping_mul(LSB16);

    let mut i = 0;
    let len = haystack.len();
    let ptr = haystack.as_ptr();

    while i + 4 <= len {
        let w1 = unsafe { ptr::read_unaligned(ptr.add(i) as *const u16) };
        let w2 = unsafe { ptr::read_unaligned(ptr.add(i + 2) as *const u16) };

        let m1 = match_word(w1, needle_word);
        let m2 = match_word(w2, needle_word);

        if (m1 | m2) != 0 {
            if m1 != 0 {
                return Some(i + get_match_index_16(m1));
            }

            return Some(i + 2 + get_match_index_16(m2));
        }

        i += 4;
    }

    if i + 2 <= len {
        let w1 = unsafe { ptr::read_unaligned(ptr.add(i) as *const u16) };
        let m1 = match_word(w1, needle_word);

        if m1 != 0 {
            return Some(i + get_match_index_16(m1));
        }

        i += 2;
    }

    haystack[i..].iter().position(|&b| b == needle).map(|pos| pos + i)
}

#[inline]
#[cfg(target_pointer_width = "64")]
fn match_qword(haystack_qword: u64, needle_qword: u64) -> u64 {
    let x = haystack_qword ^ needle_qword;
    x.wrapping_sub(LSB64) & !x & MSB64
}

#[inline]
#[cfg(target_pointer_width = "32")]
fn match_dword(haystack_dword: u32, needle_dword: u32) -> u32 {
    let x = haystack_dword ^ needle_dword;
    let m = x.wrapping_sub(LSB32) & !x & MSB32;

    m
}

#[inline]
#[cfg(target_pointer_width = "16")]
fn match_word(haystack_word: u16, needle_word: u16) -> u16 {
    let x = haystack_word ^ needle_word;
    let m = x.wrapping_sub(LSB16) & !x & MSB16;

    m
}

#[inline(always)]
#[cfg(target_pointer_width = "64")]
fn get_match_index_64(m: u64) -> usize {
    #[cfg(target_endian = "little")]
    {
        (m.trailing_zeros() / 8) as usize
    }

    #[cfg(target_endian = "big")]
    {
        (m.leading_zeros() / 8) as usize
    }
}

#[inline(always)]
#[cfg(target_pointer_width = "32")]
fn get_match_index_32(m: u32) -> usize {
    #[cfg(target_endian = "little")]
    {
        (m.trailing_zeros() / 8) as usize
    }

    #[cfg(target_endian = "big")]
    {
        (m.leading_zeros() / 8) as usize
    }
}

#[inline(always)]
#[cfg(target_pointer_width = "16")]
fn get_match_index_16(m: u16) -> usize {
    #[cfg(target_endian = "little")]
    {
        (m.trailing_zeros() / 8) as usize
    }

    #[cfg(target_endian = "big")]
    {
        (m.leading_zeros() / 8) as usize
    }
}

#[inline(always)]
#[cfg(target_arch = "x86_64")]
fn search_one_swar64(haystack: &[u8], needle: u8) -> Option<usize> {
    let needle_qword = (needle as u64).wrapping_mul(LSB64);

    let mut i = 0;
    let len = haystack.len();
    let ptr = haystack.as_ptr();

    while i + 0x20 <= len {
        let w1 = unsafe { ptr::read_unaligned(ptr.add(i) as *const u64) };
        let w2 = unsafe { ptr::read_unaligned(ptr.add(i + 8) as *const u64) };
        let w3 = unsafe { ptr::read_unaligned(ptr.add(i + 0x10) as *const u64) };
        let w4 = unsafe { ptr::read_unaligned(ptr.add(i + 0x18) as *const u64) };

        let m1 = match_qword(w1, needle_qword);
        let m2 = match_qword(w2, needle_qword);
        let m3 = match_qword(w3, needle_qword);
        let m4 = match_qword(w4, needle_qword);

        if (m1 | m2 | m3 | m4) != 0 {
            if m1 != 0 {
                return Some(i + get_match_index_64(m1));
            }

            if m2 != 0 {
                return Some(i + 8 + get_match_index_64(m2));
            }

            if m3 != 0 {
                return Some(i + 0x10 + get_match_index_64(m3));
            }

            return Some(i + 0x18 + get_match_index_64(m4));
        }

        i += 0x20;
    }

    if i + 0x10 <= len {
        let w1 = unsafe { ptr::read_unaligned(ptr.add(i) as *const u64) };
        let w2 = unsafe { ptr::read_unaligned(ptr.add(i + 8) as *const u64) };

        let m1 = match_qword(w1, needle_qword);
        let m2 = match_qword(w2, needle_qword);

        if (m1 | m2) != 0 {
            if m1 != 0 {
                return Some(i + get_match_index_64(m1));
            }

            return Some(i + 8 + get_match_index_64(m2));
        }

        i += 0x10;
    }

    if i + 8 <= len {
        let w1 = unsafe { ptr::read_unaligned(ptr.add(i) as *const u64) };
        let m1 = match_qword(w1, needle_qword);

        if m1 != 0 {
            return Some(i + get_match_index_64(m1));
        }

        i += 8;
    }

    haystack[i..].iter().position(|&b| b == needle).map(|pos| pos + i)
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

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse2")]
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
#[cfg(target_feature = "avx512bw")]
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

#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
unsafe fn search_one_neon(haystack: &[u8], needle: u8) -> Option<usize> {
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

        if vmaxvq_u32(vreinterpretq_u32_u8(or_vec)) != 0 {
            if vmaxvq_u32(vreinterpretq_u32_u8(eq1)) != 0 {
                return Some(i + get_match_index_neon(eq1));
            }

            if vmaxvq_u32(vreinterpretq_u32_u8(eq2)) != 0 {
                return Some(i + 0x10 + get_match_index_neon(eq2));
            }

            if vmaxvq_u32(vreinterpretq_u32_u8(eq3)) != 0 {
                return Some(i + 0x20 + get_match_index_neon(eq3));
            }

            return Some(i + 0x30 + get_match_index_neon(eq4));
        }

        i += 0x40;
    }

    while i + 0x10 <= len {
        let v = vld1q_u8(ptr.add(i));
        let eq = vceqq_u8(v, v_needle);

        if vmaxvq_u32(vreinterpretq_u32_u8(eq)) != 0 {
            return Some(i + get_match_index_neon(eq));
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
        (lane0.trailing_zeros() / 8) as usize
    } else {
        let lane1 = vgetq_lane_u64(eq_u64, 1);
        8 + (lane1.trailing_zeros() / 8) as usize
    }
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
