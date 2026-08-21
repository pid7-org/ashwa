[![Crates.io](https://img.shields.io/crates/v/ashwa?style=flat-square&logo=rust)](https://crates.io/crates/ashwa)
[![npm](https://img.shields.io/npm/v/@pid7/ashwa?style=flat-square&logo=npm)](https://www.npmjs.com/package/@pid7/ashwa)
[![PyPI](https://img.shields.io/pypi/v/ashwa?style=flat-square&logo=python&logoColor=white)](https://pypi.org/project/ashwa/)
[![Tests](https://img.shields.io/github/actions/workflow/status/pid7-org/ashwa/tests.yaml?style=flat-square&logo=github&label=tests)](https://github.com/pid7-org/ashwa/actions/workflows/tests.yaml)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue?style=flat-square)](LICENSE-MIT)

# Ashwa 🐎

Hardware accelerated routines for single substring search

## Language Support

`ashwa` is available across multiple language ecosystems,

- [`rust`](#) (documented below)
- [`npm`](https://github.com/pid7-org/ashwa/blob/master/npm/README.md)
- [`python`](https://github.com/pid7-org/ashwa/blob/master/pypi/README.md)

## Supported Platforms

| Architecture        | Target Platform                              | Hardware Acceleration                | Fallback    |
|:--------------------|:---------------------------------------------|:-------------------------------------|:------------|
| **x86_64**          | Linux, macOS, Windows, Android, FreeBSD      | AVX-512BW, AVX2, SSE4.2, SSSE3, SSE2 | 64-bit SWAR |
| **x86 (i686)**      | Windows, Linux                               | SSE2                                 | 32-bit SWAR |
| **AArch64 (ARM64)** | Apple Silicon, Linux ARM64, Android, FreeBSD | 128-bit ARM NEON                     | 64-bit SWAR |
| **ARMv7**           | Linux ARM, Android                           | 128-bit ARM NEON                     | 32-bit SWAR |
| **WebAssembly**     | Browsers, Node.js (wasm32)                   | WASM SIMD128 (simd128)               | 32-bit SWAR |

## Usage

Add `ashwa` to your `Cargo.toml`:

```toml
[dependencies]
ashwa = "0.1.8"
```

## Example

```rust
use ashwa::search_one;

fn main() {
    let text = b"The quick brown fox jumps over the lazy dog";
    match search_one(text, b'f') {
        Some(index) => println!("Found 'f' at byte index {}", index),
        None => println!("Byte not found"),
    }

    assert_eq!(search_one(text, b'z'), Some(0x25));
    assert_eq!(search_one(text, b'!'), None);
}
```

## Benchmarks

Observed benchmarks for `search_one`,

| Tier         | Buffer Size | Latency   | Throughput   | IPC           |
|:-------------|:------------|:----------|:-------------|:--------------|
| L1 Cache     | 32 KiB      | 211.96 ns | 143.98 GiB/s | 1.99 insn/cyc |
| L2 Cache     | 512 KiB     | 3.32 µs   | 146.94 GiB/s | 2.52 insn/cyc |
| L3 Cache     | 16 MiB      | 563.34 µs | 27.74 GiB/s  | 0.48 insn/cyc |
| Memory Bound | 256 MiB     | 21.96 ms  | 11.39 GiB/s  | 0.20 insn/cyc |

> **NOTE:**
> Benchmarked on an ephemeral **AWS EC2 `c6i.2xlarge`** instance (Intel(R) Xeon(R) Platinum 8375C CPU @ 2.90GHz, Ice Lake x86_64) pinned to an isolated CPU core with performance governor and ASLR disabled.
> - **ISA**: AVX-512BW (512-bit vector SIMD via `cargo +nightly` with `-C target-cpu=native -C target-feature=+avx512bw,+avx512f`)
> - **STREAM Triad Baseline**: 18.81 GB/s (19,264.97 MB/s, 12.46 ms)
> - **Cache Topology**: L1d: 384 KiB, L1i: 256 KiB, L2: 10 MiB, L3: 54 MiB
