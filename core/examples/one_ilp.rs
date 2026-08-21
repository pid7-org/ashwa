//! ILP (Instruction-Level Parallelism) and Hardware Profiling Harness for Ashwa
//!
//! Designed to run under Linux `perf stat` to measure Instructions Per Cycle (IPC),
//! CPU frequency, and branch predictors with a standard 1K sample size per tier.

use ashwa::search_one;
use std::alloc::{alloc_zeroed, dealloc, Layout};
use std::hint::black_box;

const KB: usize = 1024;
const MB: usize = KB * KB;
const ITERATIONS_PER_TIER: usize = 1000;

struct TierConfig {
    name: &'static str,
    size: usize,
}

const TIERS: [TierConfig; 4] = [
    TierConfig { name: "L1 Cache", size: 32 * KB },
    TierConfig { name: "L2 Cache", size: 512 * KB },
    TierConfig { name: "L3 Cache", size: 16 * MB },
    TierConfig { name: "Memory Bound (RAM)", size: 256 * MB },
];

fn run_tier(tier: &TierConfig, needle: u8) {
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
    println!("Profiling tier: {} (Size: {} bytes, Sample size: {} iters)", tier.name, size, ITERATIONS_PER_TIER);

    // Warmup
    for _ in 0..10 {
        black_box(search_one(black_box(slice), black_box(needle)));
    }

    // Profiling loop (1,000 iterations)
    for _ in 0..ITERATIONS_PER_TIER {
        black_box(search_one(black_box(slice), black_box(needle)));
    }

    unsafe { dealloc(ptr, layout) };
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let selected_tier = args.get(1).map(|s| s.to_lowercase()).unwrap_or_else(|| "all".to_string());
    let needle = 0x0Au8;

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
