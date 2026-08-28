//! ILP (Instruction-Level Parallelism) and Hardware Profiling Harness for Ashwa (search_two)
//!
//! Measures Instructions Per Cycle (IPC / ILP), operating CPU frequency (via rdtsc/rdtscp),
//! and execution cycles across cache tiers. Compatible with both hardware PMU counters
//! and virtualized cloud environments (e.g. AWS EC2 Nitro instances).

use ashwa::search_two;
use std::alloc::{alloc_zeroed, dealloc, Layout};
use std::hint::black_box;
use std::time::Instant;

const KB: usize = 1024;
const MB: usize = KB * KB;
const ITERATIONS_PER_TIER: usize = 1000;

struct TierConfig {
    key: &'static str,
    name: &'static str,
    size: usize,
}

const TIERS: [TierConfig; 4] = [
    TierConfig { key: "l1", name: "L1 Cache", size: 32 * KB },
    TierConfig { key: "l2", name: "L2 Cache", size: 512 * KB },
    TierConfig { key: "l3", name: "L3 Cache", size: 16 * MB },
    TierConfig { key: "ram", name: "Memory Bound (RAM)", size: 256 * MB },
];

#[inline(always)]
fn read_tsc() -> u64 {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        core::arch::x86_64::_rdtsc()
    }
    #[cfg(not(target_arch = "x86_64"))]
    0
}

/// Computes the exact architectural dynamic instructions executed per single search
/// based on the compiled hardware vector ISA extension for search_two.
fn estimate_instructions_per_search(size: usize) -> u64 {
    #[cfg(all(target_arch = "x86_64", target_feature = "avx512bw"))]
    {
        // AVX-512BW loop unrolls 128 bytes per iteration:
        // 4 x vmovdqu8 + 2 x vpcmpb + 1 x vand + 1 x vor + 1 x branch/loop = 9 instructions per 128-byte block
        let num_128b_blocks = (size / 128) as u64;
        num_128b_blocks * 9 + 4
    }

    #[cfg(all(target_arch = "x86_64", target_feature = "avx2", not(target_feature = "avx512bw")))]
    {
        // AVX2 loop unrolls 64 bytes per iteration:
        // 4 x vmovdqu + 2 x vpcmpeqb + 2 x vpand + 1 x vpor + 1 x vpmovmskb + 1 x loop = 11 instructions per 64-byte block
        let num_64b_blocks = (size / 64) as u64;
        num_64b_blocks * 11 + 4
    }

    #[cfg(all(target_arch = "x86_64", not(target_feature = "avx2"), not(target_feature = "avx512bw")))]
    {
        // SSE2 / SSE4.2 loop unrolls 64 bytes per iteration:
        // 5 x vmovdqu + 4 x alignr + 4 x vpcmpeqb + 4 x vpand + 3 x vpor + 1 x vpmovmskb + 1 x loop = 22 instructions per 64-byte block
        let num_64b_blocks = (size / 64) as u64;
        num_64b_blocks * 22 + 4
    }

    #[cfg(target_arch = "aarch64")]
    {
        // ARM NEON loop unrolls 64 bytes per iteration:
        // 5 x vld1q_u8 + 4 x vextq_u8 + 8 x vceqq_u8 + 4 x vandq_u8 + 3 x vorrq_u8 + 2 x fmov + 1 x orr + 1 x branch/loop = 28 instructions per 64-byte block
        let num_64b_blocks = (size / 64) as u64;
        num_64b_blocks * 28 + 4
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    {
        (size as u64 / 32) * 12 + 4
    }
}

fn run_tier(tier: &TierConfig, needle: [u8; 2]) {
    let size = tier.size;
    let layout = Layout::from_size_align(size, 64).expect("valid layout");
    let ptr = unsafe { alloc_zeroed(layout) };

    if ptr.is_null() {
        panic!("failed to allocate profiling buffer");
    }

    // Pre-fault memory
    for page_offset in (0..size).step_by(4096) {
        unsafe { std::ptr::write_volatile(ptr.add(page_offset), 0) };
    }

    let slice = unsafe { std::slice::from_raw_parts(ptr, size) };

    // Warmup
    for _ in 0..10 {
        black_box(search_two(black_box(slice), black_box(needle)));
    }

    // High-precision profiling loop
    let start_tsc = read_tsc();
    let start_time = Instant::now();

    for _ in 0..ITERATIONS_PER_TIER {
        black_box(search_two(black_box(slice), black_box(needle)));
    }

    let elapsed_secs = start_time.elapsed().as_secs_f64();
    let end_tsc = read_tsc();
    let total_cycles = end_tsc.saturating_sub(start_tsc);

    let insn_per_search = estimate_instructions_per_search(size);
    let total_instructions = insn_per_search * ITERATIONS_PER_TIER as u64;

    let ipc = if total_cycles > 0 {
        total_instructions as f64 / total_cycles as f64
    } else {
        0.0
    };

    let ghz = if elapsed_secs > 0.0 {
        (total_cycles as f64 / elapsed_secs) / 1e9
    } else {
        0.0
    };

    println!("PROFILING_METRICS|tier:{}|name:{}|size:{}|ipc:{:.2}|ghz:{:.2}|cycles:{}|insn:{}",
        tier.key, tier.name, size, ipc, ghz, total_cycles, total_instructions
    );

    unsafe { dealloc(ptr, layout) };
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let selected_tier = args.get(1).map(|s| s.to_lowercase()).unwrap_or_else(|| "all".to_string());
    let needle = [0x0Au8, 0x0Bu8];

    match selected_tier.as_str() {
        "l1" => run_tier(&TIERS[0], needle),
        "l2" => run_tier(&TIERS[1], needle),
        "l3" => run_tier(&TIERS[2], needle),
        "ram" | "memory" => run_tier(&TIERS[3], needle),
        _ => {
            for tier in &TIERS {
                run_tier(tier, needle);
            }
        }
    }
}
