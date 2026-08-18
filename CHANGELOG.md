# Changelog

All notable changes to this project will be documented in this file.

## [0.1.5] - 2026-08-18

### Added

- **PyPI Package**: Published Python package [`ashwa`](https://pypi.org/project/ashwa/) on [PyPI](https://pypi.org/project/ashwa/) with pre-built manylinux x86_64 wheel and source distribution

## [0.1.4] - 2026-08-10

### Fixed

- **NPM Release Workflow**: Upgraded npm CLI to latest version in GitHub Actions release workflow to resolve OIDC provenance token exchange issues (`404 Not Found`), added `scope: '@pid7'` routing, and configured fallback auth for NPM package publishing.

## [0.1.3] - 2026-08-08

### Fixed

- Fixed gh action for package release

## [0.1.2] - 2026-08-09

### Added

- **Public API Documentation**: Comprehensive JSDoc doc-comments and TypeScript definitions (`index.d.ts`) across all NPM entrypoints, native bindings, and WebAssembly modules
- **Multi-Platform CI/CD Release Workflow**: Added GitHub Actions workflow (`release.yaml`) for automated multi-platform native builds (Linux x64, Linux ARM64, macOS ARM64, Windows x64, Windows ARM64) and publishing to [crates.io](https://crates.io/crates/ashwa) and [npm](https://www.npmjs.com/package/@pid7/ashwa)
- **Post-Publish Verification**: Added post-deployment testing in CI release workflow to download `@pid7/ashwa` directly from NPM registry and test native execution across all 5 target platform runners

### Fixed

- **Documentation**: Fixed links and formatting in root `README.md` and `npm/README.md`

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
