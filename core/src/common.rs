use core::ptr;

#[cfg(any(target_pointer_width = "64", test))]
pub(crate) const LSB64: u64 = 0x0101_0101_0101_0101;

#[cfg(any(target_pointer_width = "64", test))]
pub(crate) const MSB64: u64 = 0x8080_8080_8080_8080;

#[cfg(any(target_pointer_width = "64", test))]
pub(crate) const MASK7_64: u64 = 0x7F7F_7F7F_7F7F_7F7F;

#[cfg(any(target_pointer_width = "32", test))]
pub(crate) const LSB32: u32 = 0x0101_0101;

#[cfg(any(target_pointer_width = "32", test))]
pub(crate) const MSB32: u32 = 0x8080_8080;

#[cfg(any(target_pointer_width = "32", test))]
pub(crate) const MASK7_32: u32 = 0x7F7F_7F7F;

#[inline]
#[cfg(any(target_pointer_width = "64", test))]
pub(crate) fn match_qword(haystack_qword: u64, needle_qword: u64) -> u64 {
    let x = haystack_qword ^ needle_qword;
    let y = (x & MASK7_64).wrapping_add(MASK7_64) | x;
    !y & MSB64
}

#[inline]
#[cfg(any(target_pointer_width = "32", test))]
pub(crate) fn match_dword(haystack_dword: u32, needle_dword: u32) -> u32 {
    let x = haystack_dword ^ needle_dword;
    let y = (x & MASK7_32).wrapping_add(MASK7_32) | x;
    !y & MSB32
}

#[inline(always)]
#[cfg(any(target_pointer_width = "64", test))]
pub(crate) fn get_match_index_64(m: u64) -> usize {
    #[cfg(target_endian = "little")]
    {
        (m.trailing_zeros() / 0x08) as usize
    }

    #[cfg(target_endian = "big")]
    {
        (m.leading_zeros() / 0x08) as usize
    }
}

#[inline(always)]
#[cfg(any(target_pointer_width = "32", test))]
pub(crate) fn get_match_index_32(m: u32) -> usize {
    #[cfg(target_endian = "little")]
    {
        (m.trailing_zeros() / 0x08) as usize
    }

    #[cfg(target_endian = "big")]
    {
        (m.leading_zeros() / 0x08) as usize
    }
}

#[inline(always)]
#[cfg(any(target_pointer_width = "64", test))]
pub(crate) fn clear_lowest_match_64(m: &mut u64) {
    #[cfg(target_endian = "little")]
    {
        *m &= *m - 1;
    }

    #[cfg(target_endian = "big")]
    {
        let offset = get_match_index_64(*m);
        *m &= !(0x80u64 << (56 - offset * 8));
    }
}

#[inline(always)]
#[cfg(any(target_pointer_width = "32", test))]
pub(crate) fn clear_lowest_match_32(m: &mut u32) {
    #[cfg(target_endian = "little")]
    {
        *m &= *m - 1;
    }

    #[cfg(target_endian = "big")]
    {
        let offset = get_match_index_32(*m);
        *m &= !(0x80u32 << (24 - offset * 8));
    }
}

#[inline(always)]
#[cfg(any(target_pointer_width = "64", test))]
pub(crate) fn search_one_swar64(haystack: &[u8], needle: u8) -> Option<usize> {
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

#[inline(always)]
#[cfg(any(target_pointer_width = "32", test))]
pub(crate) fn search_one_swar32(haystack: &[u8], needle: u8) -> Option<usize> {
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
#[cfg(any(target_pointer_width = "64", test))]
pub(crate) fn search_two_swar64(haystack: &[u8], needle: [u8; 0x02]) -> Option<usize> {
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
pub(crate) fn search_two_swar32(haystack: &[u8], needle: [u8; 0x02]) -> Option<usize> {
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

#[inline(always)]
#[cfg(any(target_pointer_width = "64", test))]
pub(crate) fn search_three_swar64(haystack: &[u8], needle: [u8; 0x03]) -> Option<usize> {
    let needle_a = (needle[0x00] as u64).wrapping_mul(LSB64);
    let needle_b = (needle[0x01] as u64).wrapping_mul(LSB64);
    let needle_c = (needle[0x02] as u64).wrapping_mul(LSB64);

    let mut i = 0x00;
    let len = haystack.len();
    let ptr = haystack.as_ptr();

    while i + 0x22 <= len {
        let w1_a = unsafe { ptr::read_unaligned(ptr.add(i) as *const u64) };
        let w1_b = unsafe { ptr::read_unaligned(ptr.add(i + 0x01) as *const u64) };
        let w1_c = unsafe { ptr::read_unaligned(ptr.add(i + 0x02) as *const u64) };
        let w2_a = unsafe { ptr::read_unaligned(ptr.add(i + 0x08) as *const u64) };
        let w2_b = unsafe { ptr::read_unaligned(ptr.add(i + 0x09) as *const u64) };
        let w2_c = unsafe { ptr::read_unaligned(ptr.add(i + 0x0A) as *const u64) };
        let w3_a = unsafe { ptr::read_unaligned(ptr.add(i + 0x10) as *const u64) };
        let w3_b = unsafe { ptr::read_unaligned(ptr.add(i + 0x11) as *const u64) };
        let w3_c = unsafe { ptr::read_unaligned(ptr.add(i + 0x12) as *const u64) };
        let w4_a = unsafe { ptr::read_unaligned(ptr.add(i + 0x18) as *const u64) };
        let w4_b = unsafe { ptr::read_unaligned(ptr.add(i + 0x19) as *const u64) };
        let w4_c = unsafe { ptr::read_unaligned(ptr.add(i + 0x1A) as *const u64) };

        let m1 =
            match_qword(w1_a, needle_a) & match_qword(w1_b, needle_b) & match_qword(w1_c, needle_c);
        let m2 =
            match_qword(w2_a, needle_a) & match_qword(w2_b, needle_b) & match_qword(w2_c, needle_c);
        let m3 =
            match_qword(w3_a, needle_a) & match_qword(w3_b, needle_b) & match_qword(w3_c, needle_c);
        let m4 =
            match_qword(w4_a, needle_a) & match_qword(w4_b, needle_b) & match_qword(w4_c, needle_c);

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

    while i + 0x0A <= len {
        let w_a = unsafe { ptr::read_unaligned(ptr.add(i) as *const u64) };
        let w_b = unsafe { ptr::read_unaligned(ptr.add(i + 0x01) as *const u64) };
        let w_c = unsafe { ptr::read_unaligned(ptr.add(i + 0x02) as *const u64) };

        let m =
            match_qword(w_a, needle_a) & match_qword(w_b, needle_b) & match_qword(w_c, needle_c);

        if m != 0x00 {
            return Some(i + get_match_index_64(m));
        }

        i += 0x08;
    }

    haystack[i..].windows(0x03).position(|w| w == needle).map(|pos| pos + i)
}

#[inline(always)]
#[cfg(any(target_pointer_width = "32", test))]
pub(crate) fn search_three_swar32(haystack: &[u8], needle: [u8; 0x03]) -> Option<usize> {
    let needle_a = (needle[0x00] as u32).wrapping_mul(LSB32);
    let needle_b = (needle[0x01] as u32).wrapping_mul(LSB32);
    let needle_c = (needle[0x02] as u32).wrapping_mul(LSB32);

    let len = haystack.len();
    let ptr = haystack.as_ptr();

    let mut i = 0x00;
    while i + 0x12 <= len {
        let w1_a = unsafe { ptr::read_unaligned(ptr.add(i) as *const u32) };
        let w1_b = unsafe { ptr::read_unaligned(ptr.add(i + 0x01) as *const u32) };
        let w1_c = unsafe { ptr::read_unaligned(ptr.add(i + 0x02) as *const u32) };
        let w2_a = unsafe { ptr::read_unaligned(ptr.add(i + 0x04) as *const u32) };
        let w2_b = unsafe { ptr::read_unaligned(ptr.add(i + 0x05) as *const u32) };
        let w2_c = unsafe { ptr::read_unaligned(ptr.add(i + 0x06) as *const u32) };
        let w3_a = unsafe { ptr::read_unaligned(ptr.add(i + 0x08) as *const u32) };
        let w3_b = unsafe { ptr::read_unaligned(ptr.add(i + 0x09) as *const u32) };
        let w3_c = unsafe { ptr::read_unaligned(ptr.add(i + 0x0A) as *const u32) };
        let w4_a = unsafe { ptr::read_unaligned(ptr.add(i + 0x0C) as *const u32) };
        let w4_b = unsafe { ptr::read_unaligned(ptr.add(i + 0x0D) as *const u32) };
        let w4_c = unsafe { ptr::read_unaligned(ptr.add(i + 0x0E) as *const u32) };

        let m1 =
            match_dword(w1_a, needle_a) & match_dword(w1_b, needle_b) & match_dword(w1_c, needle_c);
        let m2 =
            match_dword(w2_a, needle_a) & match_dword(w2_b, needle_b) & match_dword(w2_c, needle_c);
        let m3 =
            match_dword(w3_a, needle_a) & match_dword(w3_b, needle_b) & match_dword(w3_c, needle_c);
        let m4 =
            match_dword(w4_a, needle_a) & match_dword(w4_b, needle_b) & match_dword(w4_c, needle_c);

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

    while i + 0x06 <= len {
        let w_a = unsafe { ptr::read_unaligned(ptr.add(i) as *const u32) };
        let w_b = unsafe { ptr::read_unaligned(ptr.add(i + 0x01) as *const u32) };
        let w_c = unsafe { ptr::read_unaligned(ptr.add(i + 0x02) as *const u32) };

        let m =
            match_dword(w_a, needle_a) & match_dword(w_b, needle_b) & match_dword(w_c, needle_c);
        if m != 0x00 {
            return Some(i + get_match_index_32(m));
        }

        i += 0x04;
    }

    haystack[i..].windows(0x03).position(|w| w == needle).map(|pos| pos + i)
}

#[inline(always)]
#[cfg(any(target_pointer_width = "64", test))]
pub(crate) fn search_n_swar64(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    let n = needle.len();
    if n == 0 {
        return Some(0);
    }

    if n > haystack.len() {
        return None;
    }

    if n == 1 {
        return search_one_swar64(haystack, needle[0]);
    }

    if n == 2 {
        return search_two_swar64(haystack, [needle[0], needle[1]]);
    }

    if n == 3 {
        return search_three_swar64(haystack, [needle[0], needle[1], needle[2]]);
    }

    let needle_a = (needle[0] as u64).wrapping_mul(LSB64);
    let needle_b = (needle[1] as u64).wrapping_mul(LSB64);
    let needle_c = (needle[n - 1] as u64).wrapping_mul(LSB64);

    let mut i = 0;
    let len = haystack.len();
    let ptr = haystack.as_ptr();

    while i + 0x20 + n - 1 <= len {
        let w1_a = unsafe { ptr::read_unaligned(ptr.add(i) as *const u64) };
        let w1_b = unsafe { ptr::read_unaligned(ptr.add(i + 1) as *const u64) };
        let w1_c = unsafe { ptr::read_unaligned(ptr.add(i + n - 1) as *const u64) };

        let w2_a = unsafe { ptr::read_unaligned(ptr.add(i + 8) as *const u64) };
        let w2_b = unsafe { ptr::read_unaligned(ptr.add(i + 9) as *const u64) };
        let w2_c = unsafe { ptr::read_unaligned(ptr.add(i + 8 + n - 1) as *const u64) };

        let w3_a = unsafe { ptr::read_unaligned(ptr.add(i + 0x10) as *const u64) };
        let w3_b = unsafe { ptr::read_unaligned(ptr.add(i + 0x11) as *const u64) };
        let w3_c = unsafe { ptr::read_unaligned(ptr.add(i + 0x10 + n - 1) as *const u64) };

        let w4_a = unsafe { ptr::read_unaligned(ptr.add(i + 0x18) as *const u64) };
        let w4_b = unsafe { ptr::read_unaligned(ptr.add(i + 0x19) as *const u64) };
        let w4_c = unsafe { ptr::read_unaligned(ptr.add(i + 0x18 + n - 1) as *const u64) };

        let m1 =
            match_qword(w1_a, needle_a) & match_qword(w1_b, needle_b) & match_qword(w1_c, needle_c);
        let m2 =
            match_qword(w2_a, needle_a) & match_qword(w2_b, needle_b) & match_qword(w2_c, needle_c);
        let m3 =
            match_qword(w3_a, needle_a) & match_qword(w3_b, needle_b) & match_qword(w3_c, needle_c);
        let m4 =
            match_qword(w4_a, needle_a) & match_qword(w4_b, needle_b) & match_qword(w4_c, needle_c);

        if (m1 | m2 | m3 | m4) != 0 {
            if m1 != 0 {
                let mut mask = m1;
                while mask != 0 {
                    let offset = get_match_index_64(mask);
                    let cand = i + offset;
                    if &haystack[cand..cand + n] == needle {
                        return Some(cand);
                    }
                    clear_lowest_match_64(&mut mask);
                }
            }

            if m2 != 0 {
                let mut mask = m2;
                while mask != 0 {
                    let offset = get_match_index_64(mask);
                    let cand = i + 8 + offset;
                    if &haystack[cand..cand + n] == needle {
                        return Some(cand);
                    }
                    clear_lowest_match_64(&mut mask);
                }
            }

            if m3 != 0 {
                let mut mask = m3;
                while mask != 0 {
                    let offset = get_match_index_64(mask);
                    let cand = i + 0x10 + offset;
                    if &haystack[cand..cand + n] == needle {
                        return Some(cand);
                    }
                    clear_lowest_match_64(&mut mask);
                }
            }

            if m4 != 0 {
                let mut mask = m4;
                while mask != 0 {
                    let offset = get_match_index_64(mask);
                    let cand = i + 0x18 + offset;
                    if &haystack[cand..cand + n] == needle {
                        return Some(cand);
                    }
                    clear_lowest_match_64(&mut mask);
                }
            }
        }

        i += 0x20;
    }

    while i + 8 + n - 1 <= len {
        let w_a = unsafe { ptr::read_unaligned(ptr.add(i) as *const u64) };
        let w_b = unsafe { ptr::read_unaligned(ptr.add(i + 1) as *const u64) };
        let w_c = unsafe { ptr::read_unaligned(ptr.add(i + n - 1) as *const u64) };

        let m =
            match_qword(w_a, needle_a) & match_qword(w_b, needle_b) & match_qword(w_c, needle_c);

        if m != 0 {
            let mut mask = m;
            while mask != 0 {
                let offset = get_match_index_64(mask);
                let cand = i + offset;
                if &haystack[cand..cand + n] == needle {
                    return Some(cand);
                }
                clear_lowest_match_64(&mut mask);
            }
        }

        i += 8;
    }

    haystack[i..].windows(n).position(|w| w == needle).map(|pos| pos + i)
}

#[inline(always)]
#[cfg(any(target_pointer_width = "32", test))]
pub(crate) fn search_n_swar32(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    let n = needle.len();
    if n == 0 {
        return Some(0);
    }

    if n > haystack.len() {
        return None;
    }

    if n == 1 {
        return search_one_swar32(haystack, needle[0]);
    }

    if n == 2 {
        return search_two_swar32(haystack, [needle[0], needle[1]]);
    }

    if n == 3 {
        return search_three_swar32(haystack, [needle[0], needle[1], needle[2]]);
    }

    let needle_a = (needle[0] as u32).wrapping_mul(LSB32);
    let needle_b = (needle[1] as u32).wrapping_mul(LSB32);
    let needle_c = (needle[n - 1] as u32).wrapping_mul(LSB32);

    let len = haystack.len();
    let ptr = haystack.as_ptr();

    let mut i = 0;
    while i + 0x10 + n - 1 <= len {
        let w1_a = unsafe { ptr::read_unaligned(ptr.add(i) as *const u32) };
        let w1_b = unsafe { ptr::read_unaligned(ptr.add(i + 1) as *const u32) };
        let w1_c = unsafe { ptr::read_unaligned(ptr.add(i + n - 1) as *const u32) };

        let w2_a = unsafe { ptr::read_unaligned(ptr.add(i + 4) as *const u32) };
        let w2_b = unsafe { ptr::read_unaligned(ptr.add(i + 5) as *const u32) };
        let w2_c = unsafe { ptr::read_unaligned(ptr.add(i + 4 + n - 1) as *const u32) };

        let w3_a = unsafe { ptr::read_unaligned(ptr.add(i + 8) as *const u32) };
        let w3_b = unsafe { ptr::read_unaligned(ptr.add(i + 9) as *const u32) };
        let w3_c = unsafe { ptr::read_unaligned(ptr.add(i + 8 + n - 1) as *const u32) };

        let w4_a = unsafe { ptr::read_unaligned(ptr.add(i + 0x0C) as *const u32) };
        let w4_b = unsafe { ptr::read_unaligned(ptr.add(i + 0x0D) as *const u32) };
        let w4_c = unsafe { ptr::read_unaligned(ptr.add(i + 0x0C + n - 1) as *const u32) };

        let m1 =
            match_dword(w1_a, needle_a) & match_dword(w1_b, needle_b) & match_dword(w1_c, needle_c);
        let m2 =
            match_dword(w2_a, needle_a) & match_dword(w2_b, needle_b) & match_dword(w2_c, needle_c);
        let m3 =
            match_dword(w3_a, needle_a) & match_dword(w3_b, needle_b) & match_dword(w3_c, needle_c);
        let m4 =
            match_dword(w4_a, needle_a) & match_dword(w4_b, needle_b) & match_dword(w4_c, needle_c);

        if (m1 | m2 | m3 | m4) != 0 {
            if m1 != 0 {
                let mut mask = m1;
                while mask != 0 {
                    let offset = get_match_index_32(mask);
                    let cand = i + offset;
                    if &haystack[cand..cand + n] == needle {
                        return Some(cand);
                    }
                    clear_lowest_match_32(&mut mask);
                }
            }

            if m2 != 0 {
                let mut mask = m2;
                while mask != 0 {
                    let offset = get_match_index_32(mask);
                    let cand = i + 4 + offset;
                    if &haystack[cand..cand + n] == needle {
                        return Some(cand);
                    }
                    clear_lowest_match_32(&mut mask);
                }
            }

            if m3 != 0 {
                let mut mask = m3;
                while mask != 0 {
                    let offset = get_match_index_32(mask);
                    let cand = i + 8 + offset;
                    if &haystack[cand..cand + n] == needle {
                        return Some(cand);
                    }
                    clear_lowest_match_32(&mut mask);
                }
            }

            if m4 != 0 {
                let mut mask = m4;
                while mask != 0 {
                    let offset = get_match_index_32(mask);
                    let cand = i + 0x0C + offset;
                    if &haystack[cand..cand + n] == needle {
                        return Some(cand);
                    }
                    clear_lowest_match_32(&mut mask);
                }
            }
        }

        i += 0x10;
    }

    while i + 4 + n - 1 <= len {
        let w_a = unsafe { ptr::read_unaligned(ptr.add(i) as *const u32) };
        let w_b = unsafe { ptr::read_unaligned(ptr.add(i + 1) as *const u32) };
        let w_c = unsafe { ptr::read_unaligned(ptr.add(i + n - 1) as *const u32) };

        let m =
            match_dword(w_a, needle_a) & match_dword(w_b, needle_b) & match_dword(w_c, needle_c);

        if m != 0 {
            let mut mask = m;
            while mask != 0 {
                let offset = get_match_index_32(mask);
                let cand = i + offset;
                if &haystack[cand..cand + n] == needle {
                    return Some(cand);
                }
                clear_lowest_match_32(&mut mask);
            }
        }

        i += 4;
    }

    haystack[i..].windows(n).position(|w| w == needle).map(|pos| pos + i)
}
