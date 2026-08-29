"""Ashwa search_two Throughput & Latency Microbenchmark Suite (Python / PyO3 Native C-Extension)"""

import os
import sys
import time

# Ensure package directory is on sys.path if running directly
_PKG_DIR = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "python"))
if _PKG_DIR not in sys.path:
    sys.path.insert(0, _PKG_DIR)

import ashwa

KB = 0x400
MB = KB * KB
GB = 0x400 * MB
SAMPLES = 0x200

TIERS = [
    {"name": "L1", "size": 0x20 * KB},
    {"name": "L2", "size": 0x200 * KB},
    {"name": "L3", "size": 0x10 * MB},
    {"name": "RAM", "size": 0x100 * MB},
    {"name": "RAM", "size": 0x200 * MB},
    {"name": "RAM", "size": 1 * GB},
]

# Anti-DCE (Dead Code Elimination) optimization barrier sink
black_hole = None

def format_size(bytes_count: int) -> str:
    if bytes_count >= GB:
        return f"{bytes_count // GB} GiB"
    elif bytes_count >= MB:
        return f"{bytes_count // MB} MiB"
    elif bytes_count >= KB:
        return f"{bytes_count // KB} KiB"
    return f"{bytes_count} B"


def format_latency(secs: float) -> str:
    nanos = secs * 1e9
    if nanos < 1_000.0:
        return f"{nanos:.2f} ns"
    elif nanos < 1_000_000.0:
        return f"{nanos / 1_000.0:.2f} µs"
    elif nanos < 1_000_000_000.0:
        return f"{nanos / 1_000_000.0:.2f} ms"
    return f"{secs:.2f} s"

def benchmark_tier(tier: dict, haystack: bytearray, needle: bytes) -> dict:
    global black_hole
    size = tier["size"]
    view = memoryview(haystack)[:size]
    search_two = ashwa.search_two

    # Warmup phase
    warmup_start = time.perf_counter_ns()
    warmup_iters = 0

    while (
        (time.perf_counter_ns() - warmup_start) < 0x5F5E100  # 100ms
        and warmup_iters < 0x40
    ) or warmup_iters < 2:
        black_hole = search_two(view, needle)
        warmup_iters += 1

    probe_start = time.perf_counter_ns()
    probe_iters = max(0x0A, warmup_iters // 0x0A)

    for _ in range(probe_iters):
        black_hole = search_two(view, needle)

    probe_elapsed_secs = (time.perf_counter_ns() - probe_start) / 1e9
    time_per_single_iter = max(probe_elapsed_secs / probe_iters, 1e-9)

    batch_size = max(1, round(0.001 / time_per_single_iter))
    sample_durations = [0.0] * SAMPLES

    for s in range(SAMPLES):
        sample_start = time.perf_counter_ns()

        for _ in range(batch_size):
            black_hole = search_two(view, needle)

        elapsed_secs = (time.perf_counter_ns() - sample_start) / 1e9
        sample_durations[s] = elapsed_secs / batch_size

    sample_durations.sort()
    median_secs = sample_durations[len(sample_durations) // 2]
    gib_per_sec = (size / (1024.0 * 1024.0 * 1024.0)) / median_secs

    return {
        "name": tier["name"],
        "size": size,
        "latency_secs": median_secs,
        "throughput_gib": gib_per_sec,
    }

def print_table(results: list) -> None:
    col_tier = "Tier / Level"
    col_size = "Size"
    col_lat = "Latency (Median)"
    col_thrpt = "Throughput"

    w_tier = 0x16  
    w_size = 0x0A   
    w_lat = 0x12   
    w_thrpt = 0x10 

    divider = (
        f"+-{'-' * w_tier}-+-{'-' * w_size}-+-{'-' * w_lat}-+-{'-' * w_thrpt}-+"
    )

    print(divider)
    print(
        f"| {col_tier.ljust(w_tier)} | {col_size.rjust(w_size)} | {col_lat.rjust(w_lat)} | {col_thrpt.rjust(w_thrpt)} |"
    )
    print(divider)

    for r in results:
        size_str = format_size(r["size"])
        lat_str = format_latency(r["latency_secs"])
        thrpt_str = f"{r['throughput_gib']:.2f} GiB/s"
        print(
            f"| {r['name'].ljust(w_tier)} | {size_str.rjust(w_size)} | {lat_str.rjust(w_lat)} | {thrpt_str.rjust(w_thrpt)} |"
        )

    print(divider)

def main() -> None:
    needle = b"\x0a\x0b"
    max_size = max(t["size"] for t in TIERS)

    haystack = bytearray(max_size)

    for page_offset in range(0, max_size, 0x1000):  
        haystack[page_offset] = 0

    results = []
    for tier in TIERS:
        results.append(benchmark_tier(tier, haystack, needle))

    print_table(results)

if __name__ == "__main__":
    main()
