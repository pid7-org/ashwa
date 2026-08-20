use ashwa::search_one;
use std::alloc::{alloc_zeroed, dealloc, Layout};
use std::hint::black_box;
use std::time::{Duration, Instant};

const KB: usize = 0x400;
const MB: usize = KB * KB;
const SAMPLES: usize = 0x200;
const SIZES: [usize; 7] = [0x20 * KB, 0x80 * KB, 0x200 * KB, 2 * MB, 4 * MB, 0x10 * MB, 0x100 * MB];

struct BenchResult {
    size: usize,
    throughput_gib: f64,
}

fn format_size(bytes: usize) -> String {
    if bytes >= MB {
        format!("{} MiB", bytes / MB)
    } else if bytes >= KB {
        format!("{} KiB", bytes / KB)
    } else {
        format!("{} B", bytes)
    }
}

fn benchmark_size(haystack: &[u8], needle: u8) -> BenchResult {
    let size = haystack.len();
    let warmup_start = Instant::now();

    let mut warmup_iters = 0usize;
    while warmup_start.elapsed() < Duration::from_millis(0x80) || warmup_iters < 0x40 {
        black_box(search_one(black_box(haystack), black_box(needle)));
        warmup_iters += 1;
    }

    let probe_start = Instant::now();
    let probe_iters = 20.max(warmup_iters / 0x0A);

    for _ in 0..probe_iters {
        black_box(search_one(black_box(haystack), black_box(needle)));
    }

    let probe_elapsed = probe_start.elapsed().as_secs_f64();
    let time_per_single_iter = (probe_elapsed / probe_iters as f64).max(1e-9);
    let batch_size = ((0.001 / time_per_single_iter).round() as usize).max(1);

    let mut sample_durations = Vec::with_capacity(SAMPLES);
    for _ in 0..SAMPLES {
        let sample_start = Instant::now();
        for _ in 0..batch_size {
            black_box(search_one(black_box(haystack), black_box(needle)));
        }

        let elapsed = sample_start.elapsed();
        let per_iter_secs = elapsed.as_secs_f64() / (batch_size as f64);

        sample_durations.push(per_iter_secs);
    }

    sample_durations.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let median_secs = sample_durations[sample_durations.len() / 2];
    let gib_per_sec = (size as f64 / (1024.0 * 1024.0 * 1024.0)) / median_secs;

    BenchResult { size, throughput_gib: gib_per_sec }
}

fn print_table(results: &[BenchResult]) {
    let col_size = "Size";
    let col_thrpt = "Throughput";

    let max_size_len = 0x0C;
    let max_thrpt_len = 0x10;

    let divider = format!(
        "+-{:-<w_size$}-+-{:-<w_thrpt$}-+",
        "",
        "",
        w_size = max_size_len,
        w_thrpt = max_thrpt_len
    );

    println!("{}", divider);
    println!(
        "| {:<w_size$} | {:>w_thrpt$} |",
        col_size,
        col_thrpt,
        w_size = max_size_len,
        w_thrpt = max_thrpt_len
    );
    println!("{}", divider);

    for r in results {
        let size_str = format_size(r.size);
        let thrpt_str = format!("{:.2} GiB/s", r.throughput_gib);

        println!(
            "| {:<w_size$} | {:>w_thrpt$} |",
            size_str,
            thrpt_str,
            w_size = max_size_len,
            w_thrpt = max_thrpt_len
        );
    }

    println!("{}", divider);

    if !results.is_empty() {
        let mean_thrpt: f64 =
            results.iter().map(|r| r.throughput_gib).sum::<f64>() / results.len() as f64;

        let mut thrpts: Vec<f64> = results.iter().map(|r| r.throughput_gib).collect();
        thrpts.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let median_thrpt = thrpts[thrpts.len() / 2];

        let (higher_thrpt, label) =
            if mean_thrpt >= median_thrpt { (mean_thrpt, "avg") } else { (median_thrpt, "median") };

        println!();
        println!("Throughput ({}): {:.2} GiB/s", label, higher_thrpt);
    }
}

fn main() {
    let needle = 0x0Au8;
    let max_size = *SIZES.iter().max().unwrap_or(&(0x100 * MB));

    let layout = Layout::from_size_align(max_size, 0x40).expect("valid layout");
    let ptr = unsafe { alloc_zeroed(layout) };

    if ptr.is_null() {
        panic!("failed to allocate benchmark buffer");
    }

    for page_offset in (0..max_size).step_by(0x1000) {
        unsafe { std::ptr::write_volatile(ptr.add(page_offset), 0) };
    }

    let haystack = unsafe { std::slice::from_raw_parts(ptr, max_size) };
    let mut results = Vec::with_capacity(SIZES.len());

    for &size in &SIZES {
        let slice = &haystack[..size];
        let result = benchmark_size(slice, needle);

        results.push(result);
    }

    print_table(&results);
    unsafe { dealloc(ptr, layout) };
}
