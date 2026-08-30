[![PyPI](https://img.shields.io/pypi/v/ashwa?style=flat-square&logo=python&logoColor=white)](https://pypi.org/project/ashwa/)
[![Tests](https://img.shields.io/github/actions/workflow/status/pid7-org/ashwa/tests.yaml?style=flat-square&logo=github&label=tests)](https://github.com/pid7-org/ashwa/actions/workflows/tests.yaml)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue?style=flat-square)](../LICENSE-MIT)

# Ashwa 🐎

Hardware accelerated routines for single substring search

> 💡 TIP:
> `ashwa` provides native CPython extension bindings powered by Rust and SIMD vectorization.

> ℹ️ NOTE:
> The best available SIMD instruction set (AVX-512BW, AVX2, SSE4.2, SSSE3, SSE2, ARM NEON) is detected
> automatically at runtime on the host CPU with seamless fallback to SWAR routines.

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
>   * Instance: Intel(R) Xeon(R) Platinum 8488C (8C/16T)
>   * ISA: _AVX-512BW_
>   * Cache: L1d: 384 KiB · L1i: 256 KiB · L2: 16 MiB · L3: 105 MiB
>   * STREAM Triad: 25.76 GiB/s
>
> * AArch64 (_arm64_)
>   * Instance: AWS Graviton3 ARM Neoverse-V1 (16C/16T)
>   * ISA: _NEON_
>   * Cache: L1d: 1 MiB · L1i: 1 MiB · L2: 16 MiB · L3: 32 MiB
>   * STREAM Triad: 75.48 GiB/s

### `search_one`

| Level      | Payload   | Latency (x64) | Latency (arm64) | Throughput (x64) | Throughput (arm64) |
|:-----------|:----------|:--------------|:----------------|:-----------------|:-------------------|
| L1 Cache   | 32 KiB    | 304.09 ns     | 864.09 ns       | 100.36 GiB/s     | 35.32 GiB/s        |
| L2 Cache   | 512 KiB   | 6.89 µs       | 11.93 µs        | 70.82 GiB/s      | 40.94 GiB/s        |
| L3 Cache   | 16 MiB    | 488.96 µs     | 397.55 µs       | 31.96 GiB/s      | 39.30 GiB/s        |
| RAM        | 256 MiB   | 20.35 ms      | 10.33 ms        | 12.28 GiB/s      | 24.21 GiB/s        |
| RAM        | 512 MiB   | 41.72 ms      | 20.94 ms        | 11.98 GiB/s      | 23.88 GiB/s        |
| RAM        | 1 GiB     | 83.63 ms      | 39.15 ms        | 11.96 GiB/s      | 25.54 GiB/s        |

### `search_two`

| Level      | Payload   | Latency (x64) | Latency (arm64) | Throughput (x64) | Throughput (arm64) |
|:-----------|:----------|:--------------|:----------------|:-----------------|:-------------------|
| L1 Cache   | 32 KiB    | 611.86 ns     | 1.67 µs         | 49.88 GiB/s      | 18.31 GiB/s        |
| L2 Cache   | 512 KiB   | 10.77 µs      | 24.31 µs        | 45.33 GiB/s      | 20.08 GiB/s        |
| L3 Cache   | 16 MiB    | 558.87 µs     | 768.98 µs       | 27.96 GiB/s      | 20.32 GiB/s        |
| RAM        | 256 MiB   | 19.94 ms      | 13.03 ms        | 12.54 GiB/s      | 19.18 GiB/s        |
| RAM        | 512 MiB   | 41.67 ms      | 26.06 ms        | 12.00 GiB/s      | 19.18 GiB/s        |
| RAM        | 1 GiB     | 83.56 ms      | 52.04 ms        | 11.97 GiB/s      | 19.22 GiB/s        |
