//! Common SWAR constants and helper routines for multi-platform SIMD/SWAR search

#[cfg(target_pointer_width = "64")]
pub(crate) const LSB64: u64 = 0x0101_0101_0101_0101;

#[cfg(target_pointer_width = "64")]
pub(crate) const MSB64: u64 = 0x8080_8080_8080_8080;

#[cfg(target_pointer_width = "32")]
pub(crate) const LSB32: u32 = 0x0101_0101;

#[cfg(target_pointer_width = "32")]
pub(crate) const MSB32: u32 = 0x8080_8080;

#[inline]
#[cfg(target_pointer_width = "64")]
pub(crate) fn match_qword(haystack_qword: u64, needle_qword: u64) -> u64 {
    let x = haystack_qword ^ needle_qword;
    x.wrapping_sub(LSB64) & !x & MSB64
}

#[inline]
#[cfg(target_pointer_width = "32")]
pub(crate) fn match_dword(haystack_dword: u32, needle_dword: u32) -> u32 {
    let x = haystack_dword ^ needle_dword;
    let m = x.wrapping_sub(LSB32) & !x & MSB32;

    m
}

#[inline(always)]
#[cfg(target_pointer_width = "64")]
pub(crate) fn get_match_index_64(m: u64) -> usize {
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
pub(crate) fn get_match_index_32(m: u32) -> usize {
    #[cfg(target_endian = "little")]
    {
        (m.trailing_zeros() / 8) as usize
    }

    #[cfg(target_endian = "big")]
    {
        (m.leading_zeros() / 8) as usize
    }
}
