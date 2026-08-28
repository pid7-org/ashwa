[![PyPI](https://img.shields.io/pypi/v/ashwa?style=flat-square&logo=pypi)](https://pypi.org/project/ashwa/)
[![Crates.io](https://img.shields.io/crates/v/ashwa?style=flat-square&logo=rust)](https://crates.io/crates/ashwa)
[![Tests](https://img.shields.io/github/actions/workflow/status/pid7-org/ashwa/tests.yaml?style=flat-square&logo=github&label=tests)](https://github.com/pid7-org/ashwa/actions/workflows/tests.yaml)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue?style=flat-square)](../LICENSE-MIT)

# Ashwa 🐎

Hardware accelerated routines for single substring search

> [!NOTE]
> The best available SIMD instruction set is detected at runtime

## Supported Platforms

| Architecture    | Target Platform                              | Hardware Acceleration                 | Fallback    |
|:----------------|:---------------------------------------------|:--------------------------------------|:------------|
| x86_64          | Linux, macOS, Windows                        | AVX-512BW, AVX2, SSE4.2, SSSE3, SSE2  | 64-bit SWAR |
| i686 (x86)      | Linux                                        | SSE2                                  | 32-bit SWAR |
| AArch64 (ARM64) | Apple Silicon, Linux ARM64                   | 128-bit ARM NEON                      | 64-bit SWAR |
| ARMv7           | Linux ARM                                    | 128-bit ARM NEON                      | 32-bit SWAR |

## Install

```bash
pip install ashwa
```

## Example

```python
import ashwa

haystack = b"The quick brown fox jumps over the lazy dog"

match ashwa.search_one(haystack, ord("f")):
    case int(index):
        print(f"Found 'f' at byte index: {index}")
    case None:
        print("Not found")

assert ashwa.search_one(haystack, ord("z")) == 0x25
assert ashwa.search_one(haystack, ord("!")) is None
```

## API

#### `ashwa.search_one(haystack, needle) -> int | None`

Search for the first occurrence of `needle` (an `int` 0–255) in `haystack` (`bytes`, `bytearray`,
or `memoryview`).

Returns the **0-based byte index** of the first match, or `None` if not found.

#### `ashwa.search_two(haystack, needle) -> int | None`

Search for the first occurrence of a two-byte `needle` (a 2-byte sequence such as `bytes`, `bytearray`,
`memoryview`, `tuple[int, int]`, or `list[int]`) in `haystack` (`bytes`, `bytearray`, or `memoryview`).

Returns the **0-based byte index** of the first match, or `None` if not found.

## Benchmarks

- [`search_one`](#search_one)

### `search_one`

For _x86_64_ machine targeting _AVX-512BW_ SIMD ISA,

| Level      | Payload   | Latency   | Throughput   |
|:-----------|:----------|:----------|:-------------|
| L1 Cache   | 32 KiB    | 682.87 ns | 44.69 GiB/s  |
| L2 Cache   | 512 KiB   | 10.08 µs  | 48.45 GiB/s  |
| L3 Cache   | 16 MiB    | 524.71 µs | 29.78 GiB/s  |
| RAM        | 256 MiB   | 22.40 ms  | 11.16 GiB/s  |

Benchmarked using Intel(R) Xeon(R) Platinum 8375C CPU @ 2.90GHz (8C/16T) · L1d: 384 KiB, L1i: 256 KiB, L2: 10 MiB, L3: 54 MiB · STREAM Triad: 21.07 GB/s · Python 3.12.3

For _aarch64_ machine targeting _NEON_ SIMD ISA,

| Level      | Payload   | Latency   | Throughput   |
|:-----------|:----------|:----------|:-------------|
| L1 Cache   | 32 KiB    | 779.68 ns | 39.14 GiB/s  |
| L2 Cache   | 512 KiB   | 10.93 µs  | 44.68 GiB/s  |
| L3 Cache   | 16 MiB    | 364.89 µs | 42.82 GiB/s  |
| RAM        | 256 MiB   | 9.50 ms   | 26.33 GiB/s  |

Benchmarked using ARM Neoverse-V1 (16C/16T) · L1d: 1 MiB, L1i: 1 MiB, L2: 16 MiB, L3: 32 MiB · STREAM Triad: 77.35 GB/s · Python 3.12.3
