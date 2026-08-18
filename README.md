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
| **WebAssembly**     | Browsers, Node.js (`wasm32`)                 | WASM SIMD128 (`simd128`)             | 32-bit SWAR |

## Usage

Add `ashwa` to your `Cargo.toml`:

```toml
[dependencies]
ashwa = "0.1.6"
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
