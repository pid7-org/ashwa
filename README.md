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

| Architecture    | Target Platform                              | Hardware Acceleration                | Fallback    |
|:----------------|:---------------------------------------------|:-------------------------------------|:------------|
| x86_64          | Linux, macOS, Windows, Android, FreeBSD      | AVX-512BW, AVX2, SSE4.2, SSSE3, SSE2 | 64-bit SWAR |
| x86 (i686)      | Windows, Linux                               | SSE2                                 | 32-bit SWAR |
| AArch64 (ARM64) | Apple Silicon, Linux ARM64, Android, FreeBSD | 128-bit ARM NEON                     | 64-bit SWAR |
| ARMv7           | Linux ARM, Android                           | 128-bit ARM NEON                     | 32-bit SWAR |
| WebAssembly     | Browsers, Node.js (wasm32)                   | WASM SIMD128 (simd128)               | 32-bit SWAR |

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

- [`search_one`](#search_one)

### `search_one`

For _x86_64_ machine targeting _AVX-512BW_ SIMD ISA,

| Level      | Payload   | Latency   | Throughput   | ILP           |
|:-----------|:----------|:----------|:-------------|:--------------|
| L1 Cache   | 32 KiB    | 211.52 ns | 144.28 GiB/s | 1.74 insn/cyc |
| L2 Cache   | 512 KiB   | 3.32 µs   | 147.03 GiB/s | 2.53 insn/cyc |
| L3 Cache   | 16 MiB    | 495.57 µs | 31.53 GiB/s  | 0.54 insn/cyc |
| RAM        | 256 MiB   | 20.52 ms  | 12.18 GiB/s  | 0.21 insn/cyc |

Benchmarked using Intel(R) Xeon(R) Platinum 8375C CPU @ 2.90GHz (8C/16T) ·
L1d: 384 KiB, L1i: 256 KiB, L2: 10 MiB, L3: 54 MiB · STREAM Triad: 20.32 GB/s · _+nightly_ toolchain

For _aarch64_ machine targeting _NEON_ SIMD ISA,

| Level      | Payload   | Latency   | Throughput   | ILP           |
|:-----------|:----------|:----------|:-------------|:--------------|
| L1 Cache   | 32 KiB    | 667.79 ns | 45.70 GiB/s  | 0.51 insn/cyc |
| L2 Cache   | 512 KiB   | 10.82 µs  | 45.14 GiB/s  | 3.23 insn/cyc |
| L3 Cache   | 16 MiB    | 401.18 µs | 38.95 GiB/s  | 2.99 insn/cyc |
| RAM        | 256 MiB   | 9.89 ms   | 25.28 GiB/s  | 2.02 insn/cyc |

Benchmarked using ARM Neoverse-V1 (16C/16T) · L1d: 1 MiB, L1i: 1 MiB, L2: 16 MiB, L3: 32 MiB ·
STREAM Triad: 76.50 GB/s
