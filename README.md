[![Crates.io](https://img.shields.io/crates/v/ashwa?style=flat-square&logo=rust)](https://crates.io/crates/ashwa)
[![npm](https://img.shields.io/npm/v/@pid7/ashwa?style=flat-square&logo=npm)](https://www.npmjs.com/package/@pid7/ashwa)
[![PyPI](https://img.shields.io/pypi/v/ashwa?style=flat-square&logo=python&logoColor=white)](https://pypi.org/project/ashwa/)
[![Docs.rs](https://img.shields.io/docsrs/ashwa?style=flat-square&logo=rust)](https://docs.rs/ashwa)
[![Tests](https://img.shields.io/github/actions/workflow/status/pid7-org/ashwa/tests.yaml?style=flat-square&logo=github&label=tests)](https://github.com/pid7-org/ashwa/actions/workflows/tests.yaml)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue?style=flat-square)](LICENSE-MIT)

# Ashwa 🐎

Hardware accelerated routines for single substring search

## Language Ecosystems

- [`rust`](#)
- [`npm`](https://github.com/pid7-org/ashwa/blob/master/npm/README.md)
- [`pypi`](https://github.com/pid7-org/ashwa/blob/master/pypi/README.md)

## Supported Targets

| Architecture    | Target Platform                              | Target ISA                           | Fallback    |
|:----------------|:---------------------------------------------|:-------------------------------------|:------------|
| x86_64          | Linux, macOS, Windows, Android, FreeBSD      | AVX-512BW, AVX2, SSE4.2, SSSE3, SSE2 | 64-bit SWAR |
| AArch64 (ARM64) | Apple Silicon, Linux ARM64, Android, FreeBSD | 128-bit ARM NEON                     | 64-bit SWAR |
| x86 (i686)      | Windows, Linux                               | SSE2                                 | 32-bit SWAR |
| ARMv7           | Linux ARM, Android                           | 128-bit ARM NEON                     | 32-bit SWAR |
| WebAssembly     | Browsers, Node.js (wasm32)                   | WASM SIMD128 (simd128)               | 32-bit SWAR |

## Installation

Add `ashwa` to your `Cargo.toml`:

```toml
[dependencies]
ashwa = "0.2.0"
```

## API

Refer to [docs.rs](https://docs.rs/ashwa/latest/ashwa/) for detailed documentation.

## Benchmarks

- [`search_one`](#search_one)
- [`search_two`](#search_two)

| Specs            | x86_64 (`x64`)                                        | AArch64 (`arm64`)                                 |
|:-----------------|:------------------------------------------------------|:--------------------------------------------------|
| SIMD Target      | `AVX-512BW`                                           | `ARM NEON`                                        |
| CPU Model        | Intel(R) Xeon(R) Platinum 8375C @ 2.90GHz (8C/16T)    | AWS Graviton3 ARM Neoverse-V1 (16C/16T)           |
| Cache Hierarchy  | L1d: 384 KiB · L1i: 256 KiB · L2: 10 MiB · L3: 54 MiB | L1d: 1 MiB · L1i: 1 MiB · L2: 16 MiB · L3: 32 MiB |
| Memory Bandwidth | 20.32 GB/s                                            | 76.50 GB/s                                        |
| Toolchain        | `+nightly`                                            | stable                                            |

### `search_one`

| Level      | Payload   | Latency (x64) | Latency (arm64) | Throughput (x64) | Throughput (arm64) | ILP (x64) | ILP (arm64) |
|:-----------|:----------|:--------------|:----------------|:-----------------|:-------------------|:----------|:------------|
| L1 Cache   | 32 KiB    | 211.52 ns     | 667.79 ns       | 144.28 GiB/s     | 45.70 GiB/s        | 1.74      | 0.51        |
| L2 Cache   | 512 KiB   | 3.32 µs       | 10.82 µs        | 147.03 GiB/s     | 45.14 GiB/s        | 2.53      | 3.23        |
| L3 Cache   | 16 MiB    | 495.57 µs     | 401.18 µs       | 31.53 GiB/s      | 38.95 GiB/s        | 0.54      | 2.99        |
| RAM        | 256 MiB   | 20.52 ms      | 9.89 ms         | 12.18 GiB/s      | 25.28 GiB/s        | 0.21      | 2.02        |

### `search_two`

| Level      | Payload   | Latency (x64) | Latency (arm64) | Throughput (x64) | Throughput (arm64) | ILP (x64) | ILP (arm64) |
|:-----------|:----------|:--------------|:----------------|:-----------------|:-------------------|:----------|:------------|
| L1 Cache   | 32 KiB    | 427.57 ns     | 1.48 µs         | 71.37 GiB/s      | 20.61 GiB/s        | 1.68      | 3.25        |
| L2 Cache   | 512 KiB   | 6.85 µs       | 23.76 µs        | 71.27 GiB/s      | 20.55 GiB/s        | 1.84      | 4.10        |
| L3 Cache   | 16 MiB    | 506.58 µs     | 783.73 µs       | 30.84 GiB/s      | 19.94 GiB/s        | 0.82      | 4.06        |
| RAM        | 256 MiB   | 21.28 ms      | 13.10 ms        | 11.75 GiB/s      | 19.08 GiB/s        | 0.30      | 3.92        |
