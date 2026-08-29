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
- [`AVX512` Support](#avx512-support)
- [Installation](#installation)
- [API](#api)
- [Benchmarks](#benchmarks)
  - [`search_one`](#search_one)
  - [`search_two`](#search_two)

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

## `AVX512` Support

> [!IMPORTANT]
> The `AVX512BW` ISA backend requires the **nightly** toolchain and explicit target feature flags passed at
> compile-time.
>
> On stable toolchains or when flags are omitted, `ashwa` gracefully falls back to AVX2 / SSE4.2 / SWAR routines
> with zero overhead.
>
> ```bash
> # Build with explicit AVX512BW feature flags
> RUSTFLAGS="-C target-feature=+avx512bw" cargo +nightly build --release
>
> # Or enable native host CPU features (including AVX-512 on supported processors)
> RUSTFLAGS="-C target-cpu=native" cargo +nightly build --release
> ```

## Installation

Add `ashwa` to your `Cargo.toml`:

```toml
[dependencies]
ashwa = "0.2.2"
```

## API

Refer to [docs.rs](https://docs.rs/ashwa/latest/ashwa/) for the complete crate documentation and API reference.

## Benchmarks

- [`search_one`](#search_one)
- [`search_two`](#search_two)

> [!NOTE]
> Benchmarks are evaluated across dedicated AWS EC2 hardware environments,
>
> * x86_64 (_x64_)
>   * Instance: Intel(R) Xeon(R) Platinum 8488C (8C/16T)
>   * ISA: _AVX512BW_ (`+nightly`)
>   * Cache: L1d: 384 KiB · L2: 16 MiB · L3: 105 MiB
>   * STREAM Triad: 25.76 GiB/s
>
> * AArch64 (_arm64_)
>   * Instance: AWS Graviton3 ARM Neoverse-V1 (16C/16T)
>   * ISA: _NEON_ (stable)
>   * Cache: L1d: 1 MiB · L2: 16 MiB · L3: 32 MiB
>   * STREAM Triad: 75.48 GiB/s

### `search_one`

| Level      | Payload   | Latency (x64) | Latency (arm64) | Throughput (x64) | Throughput (arm64) | ILP (x64) | ILP (arm64) |
|:-----------|:----------|:--------------|:----------------|:-----------------|:-------------------|:----------|:------------|
| L1 Cache   | 32 KiB    | 203.82 ns     | 668.40 ns       | 149.73 GiB/s     | 45.66 GiB/s        | 1.41      | 2.03        |
| L2 Cache   | 512 KiB   | 3.31 µs       | 11.29 µs        | 147.74 GiB/s     | 43.24 GiB/s        | 3.10      | 3.19        |
| L3 Cache   | 16 MiB    | 465.93 µs     | 461.74 µs       | 33.54 GiB/s      | 33.84 GiB/s        | 0.79      | 3.29        |
| RAM        | 256 MiB   | 21.44 ms      | 10.87 ms        | 11.66 GiB/s      | 23.00 GiB/s        | 0.28      | 1.97        |
| RAM        | 512 MiB   | 44.49 ms      | 20.91 ms        | 11.24 GiB/s      | 23.92 GiB/s        | -         | -           |
| RAM        | 1 GiB     | 89.23 ms      | 40.30 ms        | 11.21 GiB/s      | 24.82 GiB/s        | -         | -           |

### `search_two`

| Level      | Payload   | Latency (x64) | Latency (arm64) | Throughput (x64) | Throughput (arm64) | ILP (x64) | ILP (arm64) |
|:-----------|:----------|:--------------|:----------------|:-----------------|:-------------------|:----------|:------------|
| L1 Cache   | 32 KiB    | 388.76 ns     | 1.49 µs         | 78.50 GiB/s      | 20.55 GiB/s        | 1.45      | 1.69        |
| L2 Cache   | 512 KiB   | 9.32 µs       | 23.99 µs        | 52.41 GiB/s      | 20.35 GiB/s        | 1.68      | 4.09        |
| L3 Cache   | 16 MiB    | 505.33 µs     | 768.56 µs       | 30.92 GiB/s      | 20.33 GiB/s        | 0.91      | 4.19        |
| RAM        | 256 MiB   | 19.85 ms      | 13.00 ms        | 12.60 GiB/s      | 19.23 GiB/s        | 0.37      | 3.94        |
| RAM        | 512 MiB   | 41.67 ms      | 26.09 ms        | 12.00 GiB/s      | 19.16 GiB/s        | -         | -           |
| RAM        | 1 GiB     | 83.70 ms      | 52.12 ms        | 11.95 GiB/s      | 19.19 GiB/s        | -         | -           |
