# Changelog

All notable changes to this project will be documented in this file.

## [0.1.1] - 2026-08-09

### Added

- **`search_one` impl**: Hardware-accelerated routine for single byte and single character substring
  search (`search_one`)
- **SIMD Vector Acceleration**:
  - **x86_64 / x86**: Support for AVX-512BW, AVX2, SSE4.2, SSSE3, and SSE2 instructions with dynamic runtime
    CPU feature detection
  - **AArch64 / ARM**: 128-bit ARM NEON vector instructions
  - **WebAssembly**: 128-bit WASM SIMD (`simd128`) instructions
  - **SWAR Fallback**: High-performance 64-bit and 32-bit SIMD-Within-A-Register algorithm when vector ISAs
    are unavailable
- **`no_std` Support**: Zero heap allocations, compatible with embedded and high-performance kernel environments
- **Distribution Packages**:
  - Published Rust crate [`ashwa`](https://crates.io/crates/ashwa) on [crates.io](https://crates.io/crates/ashwa)
  - Published JavaScript/TypeScript package [`@pid7/ashwa`](https://www.npmjs.com/package/@pid7/ashwa) on
    [npm](https://www.npmjs.com/package/@pid7/ashwa) with pre-built native binaries and WebAssembly fallback
