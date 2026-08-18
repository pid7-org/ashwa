[![PyPI](https://img.shields.io/pypi/v/ashwa?style=flat-square&logo=pypi)](https://pypi.org/project/ashwa/)
[![Crates.io](https://img.shields.io/crates/v/ashwa?style=flat-square&logo=rust)](https://crates.io/crates/ashwa)
[![Tests](https://img.shields.io/github/actions/workflow/status/pid7-org/ashwa/tests.yaml?style=flat-square&logo=github&label=tests)](https://github.com/pid7-org/ashwa/actions/workflows/tests.yaml)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue?style=flat-square)](../LICENSE-MIT)

# Ashwa 🐎

Hardware accelerated routines for single substring search

> **NOTE:**
> The best available SIMD instruction set is detected at runtime — no configuration needed.

## Supported Platforms

| Architecture        | Target Platform                              | Hardware Acceleration                 | Fallback    |
|:--------------------|:---------------------------------------------|:--------------------------------------|:------------|
| **x86_64**          | Linux, macOS, Windows                        | AVX-512BW, AVX2, SSE4.2, SSSE3, SSE2  | 64-bit SWAR |
| **i686 (x86)**      | Linux                                        | SSE2                                   | 32-bit SWAR |
| **AArch64 (ARM64)** | Apple Silicon, Linux ARM64                   | 128-bit ARM NEON                      | 64-bit SWAR |
| **ARMv7**           | Linux ARM                                    | 128-bit ARM NEON                      | 32-bit SWAR |

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

Search for the first occurrence of `needle` (an `int` 0–255) in `haystack`
(`bytes`, `bytearray`, or `memoryview`).

Returns the **0-based byte index** of the first match, or `None` if not found.
