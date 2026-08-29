use ashwa::search_two;
use std::alloc::{alloc_zeroed, dealloc, Layout};
use std::hint::black_box;
use std::time::{Duration, Instant};

const KB: usize = 0x400;
const MB: usize = KB * KB;
const GB: usize = 0x400 * MB;
const SAMPLES: usize = 0x200;

struct TierConfig {
    name: &'static str,
    size: usize,
}

const TIERS: [TierConfig; 6] = [
    TierConfig { name: "L1", size: 0x20 * KB },
    TierConfig { name: "L2", size: 0x200 * KB },
    TierConfig { name: "L3", size: 0x10 * MB },
    TierConfig { name: "RAM", size: 0x100 * MB },
    TierConfig { name: "RAM", size: 0x200 * MB },
    TierConfig { name: "RAM", size: 1 * GB },
];

struct BenchResult {
    name: &'static str,
    size: usize,
    latency_secs: f64,
    throughput_gib: f64,
}

fn format_size(bytes: usize) -> String {
    if bytes >= GB {
        format!("{} GiB", bytes / GB)
    } else if bytes >= MB {
        format!("{} MiB", bytes / MB)
    } else if bytes >= KB {
        format!("{} KiB", bytes / KB)
    } else {
        format!("{} B", bytes)
    }
}

fn format_latency(secs: f64) -> String {
    let nanos = secs * 1e9;

    if nanos < 1_000.0 {
        format!("{:.2} ns", nanos)
    } else if nanos < 1_000_000.0 {
        format!("{:.2} µs", nanos / 1_000.0)
    } else if nanos < 1_000_000_000.0 {
        format!("{:.2} ms", nanos / 1_000_000.0)
    } else {
        format!("{:.2} s", secs)
    }
}

fn benchmark_tier(tier: &TierConfig, haystack: &[u8], needle: [u8; 2]) -> BenchResult {
    let size = tier.size;
    let slice = &haystack[..size];

    // warmup
    let warmup_start = Instant::now();
    let mut warmup_iters = 0usize;

    while (warmup_start.elapsed() < Duration::from_millis(0x64) && warmup_iters < 0x40)
        || warmup_iters < 2
    {
        black_box(search_two(black_box(slice), black_box(needle)));
        warmup_iters += 1;
    }

    let probe_start = Instant::now();
    let probe_iters = 0x0A.max(warmup_iters / 0x0A);

    for _ in 0..probe_iters {
        black_box(search_two(black_box(slice), black_box(needle)));
    }

    let probe_elapsed = probe_start.elapsed().as_secs_f64();
    let time_per_single_iter = (probe_elapsed / probe_iters as f64).max(1e-9);
    let batch_size = ((0.001 / time_per_single_iter).round() as usize).max(1);

    let mut sample_durations = Vec::with_capacity(SAMPLES);
    for _ in 0..SAMPLES {
        let sample_start = Instant::now();
        for _ in 0..batch_size {
            black_box(search_two(black_box(slice), black_box(needle)));
        }

        let elapsed = sample_start.elapsed();
        let per_iter_secs = elapsed.as_secs_f64() / (batch_size as f64);
        sample_durations.push(per_iter_secs);
    }

    sample_durations.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let median_secs = sample_durations[sample_durations.len() / 2];
    let gib_per_sec = (size as f64 / (1024.0 * 1024.0 * 1024.0)) / median_secs;

    BenchResult { name: tier.name, size, latency_secs: median_secs, throughput_gib: gib_per_sec }
}

fn print_table(results: &[BenchResult]) {
    let col_tier = "Tier / Level";
    let col_size = "Size";
    let col_lat = "Latency (Median)";
    let col_thrpt = "Throughput";

    let w_tier = 0x16;
    let w_size = 0x0A;
    let w_lat = 0x12;
    let w_thrpt = 0x10;

    let divider = format!(
        "+-{:-<w_tier$}-+-{:-<w_size$}-+-{:-<w_lat$}-+-{:-<w_thrpt$}-+",
        "",
        "",
        "",
        "",
        w_tier = w_tier,
        w_size = w_size,
        w_lat = w_lat,
        w_thrpt = w_thrpt
    );

    println!("{}", divider);
    println!(
        "| {:<w_tier$} | {:>w_size$} | {:>w_lat$} | {:>w_thrpt$} |",
        col_tier,
        col_size,
        col_lat,
        col_thrpt,
        w_tier = w_tier,
        w_size = w_size,
        w_lat = w_lat,
        w_thrpt = w_thrpt
    );
    println!("{}", divider);

    for r in results {
        println!(
            "| {:<w_tier$} | {:>w_size$} | {:>w_lat$} | {:>w_thrpt$} |",
            r.name,
            format_size(r.size),
            format_latency(r.latency_secs),
            format!("{:.2} GiB/s", r.throughput_gib),
            w_tier = w_tier,
            w_size = w_size,
            w_lat = w_lat,
            w_thrpt = w_thrpt
        );
    }

    println!("{}", divider);
}

fn main() {
    let needle = [0x0Au8, 0x0Bu8];
    let max_size = TIERS.iter().map(|t| t.size).max().unwrap_or(1 * GB);

    let layout = Layout::from_size_align(max_size, 0x40).expect("valid layout");
    let ptr = unsafe { alloc_zeroed(layout) };

    if ptr.is_null() {
        panic!("failed to allocate benchmark buffer");
    }

    for page_offset in (0..max_size).step_by(0x1000) {
        unsafe { std::ptr::write_volatile(ptr.add(page_offset), 0) };
    }

    let haystack = unsafe { std::slice::from_raw_parts(ptr, max_size) };
    let mut results = Vec::with_capacity(TIERS.len());

    for tier in &TIERS {
        let result = benchmark_tier(tier, haystack, needle);
        results.push(result);
    }

    print_table(&results);
    unsafe { dealloc(ptr, layout) };
}
