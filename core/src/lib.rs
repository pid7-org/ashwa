//! Hardware accelerated routines for single substring search
//!
//! ## Example
//!
//! ```
//! use ashwa::search_one;
//!
//! let haystack = b"The quick brown fox jumps over the lazy dog";
//! assert_eq!(search_one(haystack, b'f'), Some(0x10));
//! assert_eq!(search_one(haystack, b'z'), Some(0x25));
//! assert_eq!(search_one(haystack, b'!'), None);
//! ```

#![cfg_attr(not(test), no_std)]
#![allow(unsafe_op_in_unsafe_fn)]
#![deny(
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_results
)]

#[cfg(not(any(target_pointer_width = "64", target_pointer_width = "32")))]
compile_error!("ashwa is only supported on 64 and 32 bit targets");

#[cfg(all(target_arch = "x86_64"))]
use core::{arch::x86_64, sync::atomic};

mod common;
mod one;
mod two;

pub use one::search_one;
pub use two::search_two;

#[repr(u8)]
#[cfg(target_arch = "x86_64")]
pub(crate) enum ISA {
    NONE,
    SWAR,
    SSE2,
    SSSE3,
    SSE4_2,
    AVX2,
    AVX512BW,
}

#[cfg(target_arch = "x86_64")]
impl From<u8> for ISA {
    #[inline(always)]
    fn from(value: u8) -> Self {
        match value {
            0 => ISA::NONE,
            1 => ISA::SWAR,
            2 => ISA::SSE2,
            3 => ISA::SSSE3,
            4 => ISA::SSE4_2,
            5 => ISA::AVX2,
            6 => ISA::AVX512BW,
            _ => unreachable!("invalid ISA {}", value),
        }
    }
}

/// Best available ISA on the target microarchitecture
#[cfg(target_arch = "x86_64")]
static CPU_FEATURE: atomic::AtomicU8 = atomic::AtomicU8::new(ISA::NONE as u8);

#[inline(always)]
#[allow(unreachable_code)]
#[cfg(target_arch = "x86_64")]
pub(crate) fn get_cpu_feature() -> ISA {
    #[cfg(forced_swar_backend)]
    return ISA::SWAR;

    #[cfg(target_feature = "avx512bw")]
    return ISA::AVX512BW;

    #[cfg(target_feature = "avx2")]
    return ISA::AVX2;

    #[cfg(target_feature = "sse4.2")]
    return ISA::SSE4_2;

    #[cfg(target_feature = "ssse3")]
    return ISA::SSSE3;

    #[cfg(target_feature = "sse2")]
    return ISA::SSE2;

    let feature = CPU_FEATURE.load(atomic::Ordering::Relaxed);
    if feature != 0 {
        return feature.into();
    }

    let detected = unsafe { detect_features_x86_64() };
    CPU_FEATURE.store(detected as u8, atomic::Ordering::Relaxed);

    detected.into()
}

#[cold]
#[inline(never)]
#[cfg(target_arch = "x86_64")]
unsafe fn detect_features_x86_64() -> ISA {
    let cpuid1 = x86_64::__cpuid(1);

    let has_sse2 = (cpuid1.edx & (1 << 0x1A)) != 0;
    let has_ssse3 = (cpuid1.ecx & (1 << 9)) != 0;
    let has_sse4_2 = (cpuid1.ecx & (1 << 0x14)) != 0;

    let osxsave = (cpuid1.ecx & (1 << 0x1B)) != 0;
    if osxsave {
        let xcr0 = x86_64::_xgetbv(0);
        let xmm_ymm_enabled = (xcr0 & 0b110) == 0b110;

        if xmm_ymm_enabled {
            let cpuid7 = x86_64::__cpuid_count(7, 0);

            #[cfg(target_feature = "avx512bw")]
            {
                let avx512f_bw = (1 << 16) | (1 << 30);
                let vbmi_vbmi2 = (1 << 1) | (1 << 6);
                if (cpuid7.ebx & avx512f_bw) == avx512f_bw
                    && (cpuid7.ecx & vbmi_vbmi2) == vbmi_vbmi2
                {
                    return ISA::AVX512BW;
                }
            }

            if (cpuid7.ebx & (1 << 5)) != 0 {
                return ISA::AVX2;
            }
        }
    }

    if has_sse4_2 {
        return ISA::SSE4_2;
    }

    if has_ssse3 {
        return ISA::SSSE3;
    }

    if has_sse2 {
        return ISA::SSE2;
    }

    ISA::SWAR
}
