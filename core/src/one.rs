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
        let w2 = unsafe { ptr::read_unaligned(ptr.add(i + 8) as *const u64) };
        let w3 = unsafe { ptr::read_unaligned(ptr.add(i + 0x10) as *const u64) };
        let w4 = unsafe { ptr::read_unaligned(ptr.add(i + 0x18) as *const u64) };

        let m1 = match_64(w1, needle_word);
        if m1 != 0 {
            return Some(i + (m1.trailing_zeros() / 8) as usize);
        }

        let m2 = match_64(w2, needle_word);
        if m2 != 0 {
            return Some(i + 8 + (m2.trailing_zeros() / 8) as usize);
        }

        let m3 = match_64(w3, needle_word);
        if m3 != 0 {
            return Some(i + 0x10 + (m3.trailing_zeros() / 8) as usize);
        }

        let m4 = match_64(w4, needle_word);
        if m4 != 0 {
            return Some(i + 0x18 + (m4.trailing_zeros() / 8) as usize);
        }

        i += 0x20;
    }

    if i + 0x10 <= len {
        let w1 = unsafe { ptr::read_unaligned(ptr.add(i) as *const u64) };
        let w2 = unsafe { ptr::read_unaligned(ptr.add(i + 8) as *const u64) };

        let m1 = match_64(w1, needle_word);
        if m1 != 0 {
            return Some(i + (m1.trailing_zeros() / 8) as usize);
        }

        let m2 = match_64(w2, needle_word);
        if m2 != 0 {
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

    haystack[i..]
        .iter()
        .position(|&b| b == needle)
        .map(|pos| pos + i)
}

#[inline]
fn match_64(haystack_word: u64, needle_word: u64) -> u64 {
    let x = haystack_word ^ needle_word;
    let m = x.wrapping_sub(LSB) & !x & MSB;

    m
}
