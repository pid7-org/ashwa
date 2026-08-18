# Changelog

## [0.1.6] - 2026-08-19

- add multi-architecture PyPI wheel build and release workflow (Linux x86_64/ARM64/ARMv7/i686, macOS Apple Silicon, Windows x86_64/ARM64)
- add Trusted Publishing (OIDC) with provenance attestations for PyPI releases
- add post-publish verification across native platforms for PyPI package

## [0.1.5] - 2026-08-18

- publish Python package to [PyPI](https://pypi.org/project/ashwa/)

## [0.1.4] - 2026-08-10

- fix npm release workflow (OIDC token exchange, scoped auth)

## [0.1.3] - 2026-08-08

- fix release action

## [0.1.2] - 2026-08-09

- add JSDoc + TypeScript definitions across all npm entrypoints
- add multi-platform release workflow (linux x64/arm64, macOS arm64, windows x64/arm64)
- add post-publish verification against the npm registry
- fix links in `README.md`

## [0.1.1] - 2026-08-09

- `search_one` — hardware-accelerated single-byte substring search
- SIMD: AVX-512BW, AVX2, SSE4.2, SSSE3, SSE2 (x86_64/x86), NEON (AArch64/ARMv7), SIMD128 (wasm32)
- SWAR fallback for targets without vector ISAs
- `no_std`, zero allocations
- publish [`ashwa`](https://crates.io/crates/ashwa) to crates.io
- publish [`@pid7/ashwa`](https://www.npmjs.com/package/@pid7/ashwa) to npm
