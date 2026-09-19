//! Benchmark measuring single-core sequential DRAM read bandwidth on x86_64.
//!
//! Evaluates AVX-512, AVX2, SSE2, and scalar 64-bit unrolled sequential read kernels
//! over an out-of-LLC memory buffer (default 1 GiB) pinned to a single CPU core.

#[cfg(not(target_arch = "x86_64"))]
fn main() {
    eprintln!("Error: This benchmark is strictly designed for x86_64 architecture.");
    std::process::exit(1);
}

#[cfg(target_arch = "x86_64")]
use std::alloc::{alloc_zeroed, dealloc, Layout};
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;
#[cfg(target_arch = "x86_64")]
use std::env;
#[cfg(target_arch = "x86_64")]
use std::hint::black_box;
#[cfg(target_arch = "x86_64")]
use std::time::Instant;

#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
#[repr(C)]
struct libc_cpu_set_t {
    __bits: [usize; 1024 / (8 * std::mem::size_of::<usize>())],
}

#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
unsafe extern "C" {
    fn sched_setaffinity(pid: i32, cpusetsize: usize, mask: *const std::ffi::c_void) -> i32;
    fn sched_getcpu() -> i32;
}

#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
fn pin_to_core(core_id: usize) -> std::io::Result<()> {
    unsafe {
        let mut set: libc_cpu_set_t = std::mem::zeroed();
        let word = core_id / (8 * std::mem::size_of::<usize>());
        let bit = core_id % (8 * std::mem::size_of::<usize>());
        if word < set.__bits.len() {
            set.__bits[word] |= 1usize << bit;
        } else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Core ID {} exceeds CPU set limit", core_id),
            ));
        }

        let ret = sched_setaffinity(
            0,
            std::mem::size_of::<libc_cpu_set_t>(),
            &set as *const libc_cpu_set_t as *const _,
        );
        if ret != 0 {
            return Err(std::io::Error::last_os_error());
        }
    }
    Ok(())
}

#[cfg(all(target_arch = "x86_64", not(target_os = "linux")))]
fn pin_to_core(_core_id: usize) -> std::io::Result<()> {
    eprintln!("Warning: Thread pinning is only directly supported on Linux x86_64.");
    Ok(())
}

#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
fn get_current_cpu() -> i32 {
    unsafe { sched_getcpu() }
}

#[cfg(all(target_arch = "x86_64", not(target_os = "linux")))]
fn get_current_cpu() -> i32 {
    -1
}

// -----------------------------------------------------------------------------
// Read Kernels (Pure DRAM Reads - No Writes, Unrolled to saturate LFBs)
// -----------------------------------------------------------------------------

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f")]
unsafe fn read_avx512_unrolled(data: &[u8]) -> u64 {
    unsafe {
        let len = data.len();
        let ptr = data.as_ptr();

        let mut a0 = _mm512_setzero_si512();
        let mut a1 = _mm512_setzero_si512();
        let mut a2 = _mm512_setzero_si512();
        let mut a3 = _mm512_setzero_si512();
        let mut a4 = _mm512_setzero_si512();
        let mut a5 = _mm512_setzero_si512();
        let mut a6 = _mm512_setzero_si512();
        let mut a7 = _mm512_setzero_si512();

        let mut offset = 0;
        while offset + 512 <= len {
            let p = ptr.add(offset);
            let v0 = _mm512_loadu_si512(p.add(0) as *const _);
            let v1 = _mm512_loadu_si512(p.add(64) as *const _);
            let v2 = _mm512_loadu_si512(p.add(128) as *const _);
            let v3 = _mm512_loadu_si512(p.add(192) as *const _);
            let v4 = _mm512_loadu_si512(p.add(256) as *const _);
            let v5 = _mm512_loadu_si512(p.add(320) as *const _);
            let v6 = _mm512_loadu_si512(p.add(384) as *const _);
            let v7 = _mm512_loadu_si512(p.add(448) as *const _);

            a0 = _mm512_add_epi64(a0, v0);
            a1 = _mm512_add_epi64(a1, v1);
            a2 = _mm512_add_epi64(a2, v2);
            a3 = _mm512_add_epi64(a3, v3);
            a4 = _mm512_add_epi64(a4, v4);
            a5 = _mm512_add_epi64(a5, v5);
            a6 = _mm512_add_epi64(a6, v6);
            a7 = _mm512_add_epi64(a7, v7);

            offset += 512;
        }

        let s0 = _mm512_add_epi64(_mm512_add_epi64(a0, a1), _mm512_add_epi64(a2, a3));
        let s1 = _mm512_add_epi64(_mm512_add_epi64(a4, a5), _mm512_add_epi64(a6, a7));
        let sum = _mm512_add_epi64(s0, s1);

        let mut lanes = [0u64; 8];
        _mm512_storeu_si512(lanes.as_mut_ptr() as *mut _, sum);
        black_box(lanes[0] ^ lanes[1] ^ lanes[2] ^ lanes[3] ^ lanes[4] ^ lanes[5] ^ lanes[6] ^ lanes[7])
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn read_avx2_unrolled(data: &[u8]) -> u64 {
    unsafe {
        let len = data.len();
        let ptr = data.as_ptr();

        let mut a0 = _mm256_setzero_si256();
        let mut a1 = _mm256_setzero_si256();
        let mut a2 = _mm256_setzero_si256();
        let mut a3 = _mm256_setzero_si256();
        let mut a4 = _mm256_setzero_si256();
        let mut a5 = _mm256_setzero_si256();
        let mut a6 = _mm256_setzero_si256();
        let mut a7 = _mm256_setzero_si256();

        let mut offset = 0;
        while offset + 256 <= len {
            let p = ptr.add(offset);
            let v0 = _mm256_loadu_si256(p.add(0) as *const __m256i);
            let v1 = _mm256_loadu_si256(p.add(32) as *const __m256i);
            let v2 = _mm256_loadu_si256(p.add(64) as *const __m256i);
            let v3 = _mm256_loadu_si256(p.add(96) as *const __m256i);
            let v4 = _mm256_loadu_si256(p.add(128) as *const __m256i);
            let v5 = _mm256_loadu_si256(p.add(160) as *const __m256i);
            let v6 = _mm256_loadu_si256(p.add(192) as *const __m256i);
            let v7 = _mm256_loadu_si256(p.add(224) as *const __m256i);

            a0 = _mm256_add_epi64(a0, v0);
            a1 = _mm256_add_epi64(a1, v1);
            a2 = _mm256_add_epi64(a2, v2);
            a3 = _mm256_add_epi64(a3, v3);
            a4 = _mm256_add_epi64(a4, v4);
            a5 = _mm256_add_epi64(a5, v5);
            a6 = _mm256_add_epi64(a6, v6);
            a7 = _mm256_add_epi64(a7, v7);

            offset += 256;
        }

        let s0 = _mm256_add_epi64(_mm256_add_epi64(a0, a1), _mm256_add_epi64(a2, a3));
        let s1 = _mm256_add_epi64(_mm256_add_epi64(a4, a5), _mm256_add_epi64(a6, a7));
        let sum = _mm256_add_epi64(s0, s1);

        let mut lanes = [0u64; 4];
        _mm256_storeu_si256(lanes.as_mut_ptr() as *mut __m256i, sum);
        black_box(lanes[0] ^ lanes[1] ^ lanes[2] ^ lanes[3])
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse2")]
unsafe fn read_sse2_unrolled(data: &[u8]) -> u64 {
    unsafe {
        let len = data.len();
        let ptr = data.as_ptr();

        let mut a0 = _mm_setzero_si128();
        let mut a1 = _mm_setzero_si128();
        let mut a2 = _mm_setzero_si128();
        let mut a3 = _mm_setzero_si128();
        let mut a4 = _mm_setzero_si128();
        let mut a5 = _mm_setzero_si128();
        let mut a6 = _mm_setzero_si128();
        let mut a7 = _mm_setzero_si128();

        let mut offset = 0;
        while offset + 128 <= len {
            let p = ptr.add(offset);
            let v0 = _mm_loadu_si128(p.add(0) as *const __m128i);
            let v1 = _mm_loadu_si128(p.add(16) as *const __m128i);
            let v2 = _mm_loadu_si128(p.add(32) as *const __m128i);
            let v3 = _mm_loadu_si128(p.add(48) as *const __m128i);
            let v4 = _mm_loadu_si128(p.add(64) as *const __m128i);
            let v5 = _mm_loadu_si128(p.add(80) as *const __m128i);
            let v6 = _mm_loadu_si128(p.add(96) as *const __m128i);
            let v7 = _mm_loadu_si128(p.add(112) as *const __m128i);

            a0 = _mm_add_epi64(a0, v0);
            a1 = _mm_add_epi64(a1, v1);
            a2 = _mm_add_epi64(a2, v2);
            a3 = _mm_add_epi64(a3, v3);
            a4 = _mm_add_epi64(a4, v4);
            a5 = _mm_add_epi64(a5, v5);
            a6 = _mm_add_epi64(a6, v6);
            a7 = _mm_add_epi64(a7, v7);

            offset += 128;
        }

        let s0 = _mm_add_epi64(_mm_add_epi64(a0, a1), _mm_add_epi64(a2, a3));
        let s1 = _mm_add_epi64(_mm_add_epi64(a4, a5), _mm_add_epi64(a6, a7));
        let sum = _mm_add_epi64(s0, s1);

        let mut lanes = [0u64; 2];
        _mm_storeu_si128(lanes.as_mut_ptr() as *mut __m128i, sum);
        black_box(lanes[0] ^ lanes[1])
    }
}

#[cfg(target_arch = "x86_64")]
unsafe fn read_scalar_u64_unrolled(data: &[u8]) -> u64 {
    unsafe {
        let len = data.len();
        let ptr = data.as_ptr() as *const u64;
        let u64_count = len / 8;

        let mut a0 = 0u64;
        let mut a1 = 0u64;
        let mut a2 = 0u64;
        let mut a3 = 0u64;
        let mut a4 = 0u64;
        let mut a5 = 0u64;
        let mut a6 = 0u64;
        let mut a7 = 0u64;

        let mut i = 0;
        while i + 8 <= u64_count {
            a0 = a0.wrapping_add(ptr.add(i).read_unaligned());
            a1 = a1.wrapping_add(ptr.add(i + 1).read_unaligned());
            a2 = a2.wrapping_add(ptr.add(i + 2).read_unaligned());
            a3 = a3.wrapping_add(ptr.add(i + 3).read_unaligned());
            a4 = a4.wrapping_add(ptr.add(i + 4).read_unaligned());
            a5 = a5.wrapping_add(ptr.add(i + 5).read_unaligned());
            a6 = a6.wrapping_add(ptr.add(i + 6).read_unaligned());
            a7 = a7.wrapping_add(ptr.add(i + 7).read_unaligned());
            i += 8;
        }

        black_box(a0 ^ a1 ^ a2 ^ a3 ^ a4 ^ a5 ^ a6 ^ a7)
    }
}

// -----------------------------------------------------------------------------
// Benchmark Execution & Statistics
// -----------------------------------------------------------------------------

#[cfg(target_arch = "x86_64")]
struct KernelStat {
    name: &'static str,
    bytes: usize,
    times_secs: Vec<f64>,
}

#[cfg(target_arch = "x86_64")]
impl KernelStat {
    fn new(name: &'static str, bytes: usize) -> Self {
        Self { name, bytes, times_secs: Vec::new() }
    }

    fn min_secs(&self) -> f64 {
        self.times_secs.iter().copied().fold(f64::INFINITY, f64::min)
    }

    fn median_secs(&self) -> f64 {
        let mut sorted = self.times_secs.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        if sorted.is_empty() {
            0.0
        } else {
            sorted[sorted.len() / 2]
        }
    }

    fn mean_secs(&self) -> f64 {
        if self.times_secs.is_empty() {
            0.0
        } else {
            self.times_secs.iter().sum::<f64>() / self.times_secs.len() as f64
        }
    }

    fn stddev_secs(&self) -> f64 {
        if self.times_secs.len() <= 1 {
            return 0.0;
        }
        let mean = self.mean_secs();
        let var = self.times_secs.iter().map(|&x| (x - mean).powi(2)).sum::<f64>()
            / (self.times_secs.len() - 1) as f64;
        var.sqrt()
    }

    fn peak_gib_s(&self) -> f64 {
        let min_t = self.min_secs();
        if min_t > 0.0 {
            (self.bytes as f64 / (1024.0 * 1024.0 * 1024.0)) / min_t
        } else {
            0.0
        }
    }

    fn median_gib_s(&self) -> f64 {
        let med_t = self.median_secs();
        if med_t > 0.0 {
            (self.bytes as f64 / (1024.0 * 1024.0 * 1024.0)) / med_t
        } else {
            0.0
        }
    }

    fn peak_gb_s(&self) -> f64 {
        let min_t = self.min_secs();
        if min_t > 0.0 {
            (self.bytes as f64 / 1.0e9) / min_t
        } else {
            0.0
        }
    }
}

#[cfg(target_arch = "x86_64")]
fn run_kernel_benchmark<F>(
    name: &'static str,
    buffer: &[u8],
    warmups: usize,
    iters: usize,
    mut f: F,
) -> KernelStat
where
    F: FnMut(&[u8]) -> u64,
{
    // Warmup
    for _ in 0..warmups {
        black_box(f(black_box(buffer)));
    }

    let mut stat = KernelStat::new(name, buffer.len());
    for _ in 0..iters {
        let start = Instant::now();
        let res = black_box(f(black_box(buffer)));
        let elapsed = start.elapsed().as_secs_f64();
        black_box(res);
        stat.times_secs.push(elapsed);
    }

    stat
}

#[cfg(target_arch = "x86_64")]
fn print_results_table(results: &[KernelStat], buffer_size: usize) {
    let size_gib = buffer_size as f64 / (1024.0 * 1024.0 * 1024.0);
    println!("\n+---------------------------------------------------------------------------------------------------+");
    println!("| {:<24} | {:>10} | {:>14} | {:>14} | {:>12} | {:>10} |", "Kernel", "Size (GiB)", "Peak (GiB/s)", "Median (GiB/s)", "Peak (GB/s)", "Min (ms)");
    println!("+---------------------------------------------------------------------------------------------------+");

    for s in results {
        println!(
            "| {:<24} | {:>10.2} | {:>14.2} | {:>14.2} | {:>12.2} | {:>10.2} |",
            s.name,
            size_gib,
            s.peak_gib_s(),
            s.median_gib_s(),
            s.peak_gb_s(),
            s.min_secs() * 1000.0,
        );
    }
    println!("+---------------------------------------------------------------------------------------------------+\n");
}

#[cfg(target_arch = "x86_64")]
fn print_detailed_breakdown(results: &[KernelStat]) {
    println!("--- Detailed Statistical Breakdown ---");
    for s in results {
        let mean = s.mean_secs();
        let stddev = s.stddev_secs();
        let rsd = if mean > 0.0 { (stddev / mean) * 100.0 } else { 0.0 };
        println!(
            "• {:<22}: Min={:.3}ms | Med={:.3}ms | Mean={:.3}ms | StdDev={:.3}ms (RSD: {:.2}%) | Peak={:.2} GiB/s ({:.2} GB/s)",
            s.name,
            s.min_secs() * 1000.0,
            s.median_secs() * 1000.0,
            mean * 1000.0,
            stddev * 1000.0,
            rsd,
            s.peak_gib_s(),
            s.peak_gb_s()
        );
    }
    println!();
}

#[cfg(target_arch = "x86_64")]
fn main() {
    let args: Vec<String> = env::args().collect();

    let mut core_id = env::var("CPU_CORE").ok().and_then(|s| s.parse::<usize>().ok()).unwrap_or(2);
    let mut size_gib = 1.0f64;
    let mut iters = 10usize;
    let mut warmups = 3usize;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-c" | "--core" => {
                if i + 1 < args.len() {
                    core_id = args[i + 1].parse().unwrap_or(core_id);
                    i += 1;
                }
            }
            "-s" | "--size-gib" => {
                if i + 1 < args.len() {
                    size_gib = args[i + 1].parse().unwrap_or(size_gib);
                    i += 1;
                }
            }
            "-n" | "--iters" => {
                if i + 1 < args.len() {
                    iters = args[i + 1].parse().unwrap_or(iters);
                    i += 1;
                }
            }
            "-w" | "--warmups" => {
                if i + 1 < args.len() {
                    warmups = args[i + 1].parse().unwrap_or(warmups);
                    i += 1;
                }
            }
            "-h" | "--help" => {
                println!("Usage: dram_read [OPTIONS]");
                println!("Options:");
                println!("  -c, --core <ID>        CPU core to pin (default: $CPU_CORE or 2)");
                println!("  -s, --size-gib <GIB>   Buffer allocation size in GiB (default: 1.0)");
                println!("  -n, --iters <N>        Number of timed iterations (default: 10)");
                println!("  -w, --warmups <N>      Number of warmup iterations (default: 3)");
                println!("  -h, --help             Show this help message");
                return;
            }
            _ => {}
        }
        i += 1;
    }

    println!("================================================================================");
    println!("        x86_64 Single-Core Sequential DRAM Read Bandwidth Benchmark             ");
    println!("================================================================================");

    // Pinning
    match pin_to_core(core_id) {
        Ok(_) => {
            let actual_cpu = get_current_cpu();
            println!("Thread pinned to CPU Core: {} (Verified affinity on core: {})", core_id, actual_cpu);
        }
        Err(e) => {
            eprintln!("Warning: Failed to pin to CPU core {}: {}. Proceeding unpinned.", core_id, e);
        }
    }

    let buffer_size = (size_gib * 1024.0 * 1024.0 * 1024.0).round() as usize;
    println!("Allocating buffer: {:.2} GiB ({} bytes), 4096-byte aligned...", size_gib, buffer_size);

    let layout = Layout::from_size_align(buffer_size, 4096).expect("Invalid layout");
    let ptr = unsafe { alloc_zeroed(layout) };
    if ptr.is_null() {
        panic!("Failed to allocate {} bytes for benchmark buffer!", buffer_size);
    }

    print!("Pre-faulting and dirtying all pages across DRAM buffer... ");
    for offset in (0..buffer_size).step_by(4096) {
        unsafe {
            std::ptr::write_volatile(ptr.add(offset), 0x5A);
        }
    }
    println!("Done.");

    let buffer = unsafe { std::slice::from_raw_parts(ptr, buffer_size) };

    println!("Running benchmark (Warmups: {}, Timed Iterations: {})...\n", warmups, iters);

    let mut results = Vec::new();

    // 1. AVX-512 (if supported)
    if is_x86_feature_detected!("avx512f") {
        print!("Benchmarking [AVX-512 (512-bit, 8x unrolled)]... ");
        let stat = run_kernel_benchmark(
            "AVX-512 (512-bit, 8x)",
            buffer,
            warmups,
            iters,
            |buf| unsafe { read_avx512_unrolled(buf) },
        );
        println!("Peak: {:.2} GiB/s", stat.peak_gib_s());
        results.push(stat);
    } else {
        println!("Skipping AVX-512 (CPU does not support avx512f)");
    }

    // 2. AVX2 (if supported)
    if is_x86_feature_detected!("avx2") {
        print!("Benchmarking [AVX2 (256-bit, 8x unrolled)]... ");
        let stat = run_kernel_benchmark(
            "AVX2 (256-bit, 8x)",
            buffer,
            warmups,
            iters,
            |buf| unsafe { read_avx2_unrolled(buf) },
        );
        println!("Peak: {:.2} GiB/s", stat.peak_gib_s());
        results.push(stat);
    } else {
        println!("Skipping AVX2 (CPU does not support avx2)");
    }

    // 3. SSE2
    if is_x86_feature_detected!("sse2") {
        print!("Benchmarking [SSE2 (128-bit, 8x unrolled)]... ");
        let stat = run_kernel_benchmark(
            "SSE2 (128-bit, 8x)",
            buffer,
            warmups,
            iters,
            |buf| unsafe { read_sse2_unrolled(buf) },
        );
        println!("Peak: {:.2} GiB/s", stat.peak_gib_s());
        results.push(stat);
    }

    // 4. Scalar 64-bit
    {
        print!("Benchmarking [Scalar u64 (64-bit, 8x unrolled)]... ");
        let stat = run_kernel_benchmark(
            "Scalar u64 (64-bit, 8x)",
            buffer,
            warmups,
            iters,
            |buf| unsafe { read_scalar_u64_unrolled(buf) },
        );
        println!("Peak: {:.2} GiB/s", stat.peak_gib_s());
        results.push(stat);
    }

    // Print summary
    print_results_table(&results, buffer_size);
    print_detailed_breakdown(&results);

    // Cleanup
    unsafe {
        dealloc(ptr, layout);
    }
}
