[![Crates.io](https://img.shields.io/crates/v/ashwa?style=flat-square&logo=rust)](https://crates.io/crates/ashwa)
[![npm](https://img.shields.io/npm/v/@pid7/ashwa?style=flat-square&logo=npm)](https://www.npmjs.com/package/@pid7/ashwa)
[![PyPI](https://img.shields.io/pypi/v/ashwa?style=flat-square&logo=python&logoColor=white)](https://pypi.org/project/ashwa/)
[![Docs.rs](https://img.shields.io/docsrs/ashwa?style=flat-square&logo=rust)](https://docs.rs/ashwa)
[![Tests](https://img.shields.io/github/actions/workflow/status/pid7-org/ashwa/tests.yaml?style=flat-square&logo=github&label=tests)](https://github.com/pid7-org/ashwa/actions/workflows/tests.yaml)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue?style=flat-square)](LICENSE-MIT)

# Ashwa 🐎

Hardware accelerated routines for single substring search

## Index

- [Language Ecosystems](#language-ecosystems)
- [Supported Targets](#supported-targets)
- [Installation](#installation)
- [API](#api)
- [Benchmarks](#benchmarks)
  - [`search_one`](#search_one)
  - [`search_two`](#search_two)
  - [`search_three`](#search_three)

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
ashwa = "0.2.5"
```

> [!IMPORTANT]
> Minimum Supported Rust Version (MSRV) is `1.89.0`.
>
> `ashwa` needs Rust _1.89.0_ or newer to use stable AVX-512BW support. This allows it to automatically
> choose the best instructions for each x86_64 processor without needing a nightly Rust version.

## API

Refer to [docs.rs](https://docs.rs/ashwa/latest/ashwa/) for the complete crate documentation and API reference.

## Benchmarks

- [`search_one`](#search_one)
- [`search_two`](#search_two)
- [`search_three`](#search_three)

> [!NOTE]
> Benchmarks are evaluated across dedicated AWS EC2 hardware environments,
>
> * x86_64 (_x64_)
>   * Instance: Intel(R) Xeon(R) Platinum 8488C (8C/16T)
>   * ISA: _AVX512BW_
>   * Cache: L1d: 384 KiB · L2: 16 MiB · L3: 105 MiB
>   * STREAM Triad: 25.76 GiB/s
>
> * AArch64 (_arm64_)
>   * Instance: AWS Graviton3 ARM Neoverse-V1 (16C/16T)
>   * ISA: _NEON_
>   * Cache: L1d: 1 MiB · L2: 16 MiB · L3: 32 MiB
>   * STREAM Triad: 75.48 GiB/s

### `search_one`

| Level      | Payload   | Latency (x64) | Latency (arm64) | Throughput (x64) | Throughput (arm64) |
|:-----------|:----------|:--------------|:----------------|:-----------------|:-------------------|
| L1 Cache   | 32 KiB    | 203.82 ns     | 668.40 ns       | 149.73 GiB/s     | 45.66 GiB/s        |
| L2 Cache   | 512 KiB   | 3.31 µs       | 11.29 µs        | 147.74 GiB/s     | 43.24 GiB/s        |
| L3 Cache   | 16 MiB    | 465.93 µs     | 461.74 µs       | 33.54 GiB/s      | 33.84 GiB/s        |
| RAM        | 256 MiB   | 21.44 ms      | 10.87 ms        | 11.66 GiB/s      | 23.00 GiB/s        |
| RAM        | 512 MiB   | 44.49 ms      | 20.91 ms        | 11.24 GiB/s      | 23.92 GiB/s        |
| RAM        | 1 GiB     | 89.23 ms      | 40.30 ms        | 11.21 GiB/s      | 24.82 GiB/s        |

### `search_two`

| Level      | Payload   | Latency (x64) | Latency (arm64) | Throughput (x64) | Throughput (arm64) |
|:-----------|:----------|:--------------|:----------------|:-----------------|:-------------------|
| L1 Cache   | 32 KiB    | 388.76 ns     | 1.49 µs         | 78.50 GiB/s      | 20.55 GiB/s        |
| L2 Cache   | 512 KiB   | 9.32 µs       | 23.99 µs        | 52.41 GiB/s      | 20.35 GiB/s        |
| L3 Cache   | 16 MiB    | 505.33 µs     | 768.56 µs       | 30.92 GiB/s      | 20.33 GiB/s        |
| RAM        | 256 MiB   | 19.85 ms      | 13.00 ms        | 12.60 GiB/s      | 19.23 GiB/s        |
| RAM        | 512 MiB   | 41.67 ms      | 26.09 ms        | 12.00 GiB/s      | 19.16 GiB/s        |
| RAM        | 1 GiB     | 83.70 ms      | 52.12 ms        | 11.95 GiB/s      | 19.19 GiB/s        |

### `search_three`

| Level      | Payload   | Latency (x64) | Latency (arm64) | Throughput (x64) | Throughput (arm64) |
|:-----------|:----------|:--------------|:----------------|:-----------------|:-------------------|
| L1 Cache   | 32 KiB    | 427.32 ns     | 2.17 µs         | 71.42 GiB/s      | 14.07 GiB/s        |
| L2 Cache   | 512 KiB   | 7.87 µs       | 34.69 µs        | 62.01 GiB/s      | 14.08 GiB/s        |
| L3 Cache   | 16 MiB    | 474.87 µs     | 1.12 ms         | 32.90 GiB/s      | 13.91 GiB/s        |
| RAM        | 256 MiB   | 20.33 ms      | 17.92 ms        | 12.30 GiB/s      | 13.95 GiB/s        |
| RAM        | 512 MiB   | 41.13 ms      | 35.96 ms        | 12.16 GiB/s      | 13.91 GiB/s        |
| RAM        | 1 GiB     | 87.32 ms      | 71.86 ms        | 11.45 GiB/s      | 13.92 GiB/s        |

