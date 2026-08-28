[![PyPI](https://img.shields.io/pypi/v/ashwa?style=flat-square&logo=python&logoColor=white)](https://pypi.org/project/ashwa/)
[![Tests](https://img.shields.io/github/actions/workflow/status/pid7-org/ashwa/tests.yaml?style=flat-square&logo=github&label=tests)](https://github.com/pid7-org/ashwa/actions/workflows/tests.yaml)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue?style=flat-square)](../LICENSE-MIT)

# Ashwa 🐎

Hardware accelerated routines for single substring search

> 💡 **Tip:** `ashwa` provides native CPython extension bindings powered by Rust and SIMD vectorization.
>
> ℹ️ **Note:** The best available SIMD instruction set (AVX-512BW, AVX2, SSE4.2, SSSE3, SSE2, ARM NEON) is detected automatically at runtime on the host CPU with seamless fallback to SWAR routines.

## Index

- [Supported Platforms](#supported-platforms)
- [Installation](#installation)
- [API Reference](#api-reference)
  - [`search_one`](#search_one)
  - [`search_two`](#search_two)
- [Benchmarks](#benchmarks)
  - [`search_one`](#search_one-1)
  - [`search_two`](#search_two-1)

## Supported Platforms

| Architecture    | Target Platform                              | Target ISA                            | Fallback    |
|:----------------|:---------------------------------------------|:--------------------------------------|:------------|
| x86_64          | Linux, macOS, Windows                        | AVX-512BW, AVX2, SSE4.2, SSSE3, SSE2  | 64-bit SWAR |
| i686 (x86)      | Linux, Windows                               | SSE2                                  | 32-bit SWAR |
| AArch64 (ARM64) | Apple Silicon, Linux ARM64, Android, FreeBSD | 128-bit ARM NEON                      | 64-bit SWAR |
| ARMv7           | Linux ARM, Android                           | 128-bit ARM NEON                      | 32-bit SWAR |

## Installation

Install `ashwa` using your preferred Python package manager:

```bash
pip install ashwa
```

## API Reference

### `search_one(haystack, needle)`

```py
def search_one(
    haystack: bytes | bytearray | memoryview,
    needle: int,
) -> int | None:
    ...
```

Searches for the first occurrence of `needle` (an `int` `0`–`255`) within `haystack`.

- Parameters:
  - `haystack`: `bytes | bytearray | memoryview` — The bytes-like object to search in.
  - `needle`: `int` — The target byte value (`0`–`255`) to locate.
- Returns:
  - The 0-based byte index (`int`) of the first occurrence of `needle`, or `None` if not found.

### `search_two(haystack, needle)`

```py
def search_two(
    haystack: bytes | bytearray | memoryview,
    needle: bytes | bytearray | memoryview | tuple[int, int] | list[int] | Sequence[int],
) -> int | None:
    ...
```

Searches for the first occurrence of a two-byte `needle` within `haystack`.

- Parameters:
  - `haystack`: `bytes | bytearray | memoryview` — The bytes-like object to search in.
  - `needle`: `bytes | bytearray | memoryview | tuple[int, int] | list[int] | Sequence[int]` — A 2-byte sequence to locate.
- Returns:
  - The 0-based byte index (`int`) of the first occurrence of `needle`, or `None` if not found.

## Benchmarks

- [`search_one`](#search_one-1)
- [`search_two`](#search_two-1)

> Benchmarks are evaluated across dedicated AWS EC2 hardware environments on Python `3.12.3`,
>
> * x86_64 (_x64_)
>   * Instance: Intel(R) Xeon(R) Platinum 8375C CPU @ 2.90GHz (8C/16T)
>   * ISA: _AVX-512BW_ (`+nightly`)
>   * Cache: L1d: 384 KiB · L1i: 256 KiB · L2: 10 MiB · L3: 54 MiB
>   * STREAM Triad: 20.94 GiB/s
>
> * AArch64 (_arm64_)
>   * Instance: AWS Graviton3 ARM Neoverse-V1 (16C/16T)
>   * ISA: _NEON_ (stable)
>   * Cache: L1d: 1 MiB · L1i: 1 MiB · L2: 16 MiB · L3: 32 MiB
>   * STREAM Triad: 77.35 GiB/s

### `search_one`

| Level      | Payload   | Latency (x64) | Latency (arm64) | Throughput (x64) | Throughput (arm64) |
|:-----------|:----------|:--------------|:----------------|:-----------------|:-------------------|
| L1 Cache   | 32 KiB    | 404.35 ns     | 779.68 ns       | 75.47 GiB/s      | 39.14 GiB/s        |
| L2 Cache   | 512 KiB   | 10.21 µs      | 10.93 µs        | 47.81 GiB/s      | 44.68 GiB/s        |
| L3 Cache   | 16 MiB    | 553.24 µs     | 364.89 µs       | 28.24 GiB/s      | 42.82 GiB/s        |
| RAM        | 256 MiB   | 20.17 ms      | 9.50 ms         | 12.39 GiB/s      | 26.33 GiB/s        |

### `search_two`

| Level      | Payload   | Latency (x64) | Latency (arm64) | Throughput (x64) | Throughput (arm64) |
|:-----------|:----------|:--------------|:----------------|:-----------------|:-------------------|
| L1 Cache   | 32 KiB    | 727.09 ns     | 1.67 µs         | 41.97 GiB/s      | 18.32 GiB/s        |
| L2 Cache   | 512 KiB   | 17.44 µs      | 23.83 µs        | 27.99 GiB/s      | 20.49 GiB/s        |
| L3 Cache   | 16 MiB    | 656.31 µs     | 782.46 µs       | 23.81 GiB/s      | 19.97 GiB/s        |
| RAM        | 256 MiB   | 22.26 ms      | 13.04 ms        | 11.23 GiB/s      | 19.17 GiB/s        |
