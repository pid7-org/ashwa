//! Hardware accelerated routines for single substring search

#![cfg_attr(not(test), no_std)]
#![allow(unsafe_op_in_unsafe_fn)]
#![allow(unused)]

#[cfg(not(any(
    target_pointer_width = "64",
    target_pointer_width = "32",
    target_pointer_width = "16"
)))]
compile_error!("ashwa is only supported on 64, 32 and 16 bit targets");

#[cfg(all(target_arch = "x86_64"))]
use core::{arch::x86_64, sync::atomic};

mod one;

pub use one::search_one;

/// Best available ISA on the target microarchitecture
///
/// ISA Enumeration:
///
/// * 0: Uninitialized
/// * 1: SWAR
/// * 2: SSE2
/// * 3: SSSE3
/// * 4: SSSE4.2
/// * 5: AVX2
#[cfg(all(target_arch = "x86_64"))]
static CPU_FEATURE: atomic::AtomicU8 = atomic::AtomicU8::new(0);

#[inline(always)]
#[cfg(target_arch = "x86_64")]
pub(crate) fn get_cpu_feature() -> u8 {
    let feature = CPU_FEATURE.load(atomic::Ordering::Relaxed);
    if feature != 0 {
        return feature;
    }

    let detected = unsafe { detect_features_x86_64() };
    CPU_FEATURE.store(detected, atomic::Ordering::Relaxed);

    // sanity check
    debug_assert!(detected <= 5, "Invalid ID detected for CPU_FEATURE");

    detected
}

#[cold]
#[inline(never)]
#[cfg(target_arch = "x86_64")]
unsafe fn detect_features_x86_64() -> u8 {
    let cpuid1 = x86_64::__cpuid(1);

    let has_sse2 = (cpuid1.edx & (1 << 0x1A)) != 0;
    let has_ssse3 = (cpuid1.ecx & (1 << 9)) != 0;
    let has_sse42 = (cpuid1.ecx & (1 << 0x14)) != 0;

    let osxsave = (cpuid1.ecx & (1 << 0x1B)) != 0;
    if osxsave {
        let xcr0 = x86_64::_xgetbv(0);
        let xmm_ymm_enabled = (xcr0 & 0b110) == 0b110;

        if xmm_ymm_enabled {
            let cpuid7 = x86_64::__cpuid_count(7, 0);
            if (cpuid7.ebx & (1 << 5)) != 0 {
                return 5;
            }
        }
    }

    if has_sse42 {
        return 4;
    }

    if has_ssse3 {
        return 3;
    }

    if has_sse2 {
        return 2;
    }

    // fallback to SWAR
    1
}
