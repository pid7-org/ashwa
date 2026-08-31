# Changelog

## [1.0.1] - 2026-08-31

- pypi: update development status classifier to `Production/Stable`

## [1.0.0] - 2026-08-31

- Initial stable release.

## [0.2.6] - 2026-08-31

- implement `search_n`, hardware-accelerated routine for arbitrary-length ($N$) substring search
  - SIMD vector backends: AVX-512BW, AVX2, SSE4.2, SSSE3, SSE2, ARM NEON, WASM SIMD128
  - 64-bit and 32-bit SWAR fallback routines
  - native Node.js (N-API), browser (WASM SIMD), and Python (CPython) bindings
  - comprehensive test suites, throughput benchmarks, and ILP profiling across all ecosystems
- bencher:
  - add `search_n` benchmark suite and hardware profiling support
- docs:
  - add `search_n` benchmark tables across core, npm, and PyPI READMEs
  - document single-core DRAM bandwidth saturation limits and memory-bound throughput characteristics

## [0.2.5] - 2026-08-30

- implement `search_three`, hardware-accelerated routine for 3-byte substring search
  - SIMD vector backends: AVX-512BW, AVX2, SSE4.2, SSSE3, SSE2, NEON, SIMD128
  - 64-bit and 32-bit SWAR fallback routines
  - native Node.js (N-API), browser (WASM SIMD), and Python (CPython) bindings
  - comprehensive test suites and throughput microbenchmarks across all ecosystems
- bencher:
  - add `search_three` benchmark suite and harness support
  - standardize on stable Rust toolchain for x86_64 runners

## [0.2.4] - 2026-08-30

- fix `Illegal instruction` crashes on CPUs with lesser ISA support:
  - npm native addons and pypi wheels were previously compiled with `-C target-cpu=native`, baking
    runner-specific vector instructions into the base binary and causing `Illegal instruction` crashes
    on older x86_64 CPUs
  - replaced static compile-time flags with dynamic runtime CPUID feature detection and dispatch for
    AVX-512BW kernels
  - standardized release CI builds on the stable Rust toolchain and portable baseline flags, guaranteeing
    universal CPU compatibility while dynamically dispatching to AVX-512BW on supported hardware
- core:
  - set MSRV to `1.89.0` (required for stable `#[target_feature(enable = "avx512bw")]`)
- npm:
  - add `./browser` and `./package.json` entrypoints to exports map
  - convert browser test and benchmark suites to native ES Modules (ESM)
  - pass options object to WebAssembly `initSync` to avoid deprecation warnings
- ci:
  - decouple post-publish verification into a standalone workflow (`verify-published.yml`) running isolated
    test suites across npm (Node.js native, WASM ESM, headless Chromium) and pypi across all platforms
- docs:
  - update core, npm, and pypi README(s) with MSRV notes and remove obsolete AVX-512 nightly toolchain requirements

## [0.2.3] - 2026-08-29

- npm: ensure native loader uses CommonJS exports for cross-bundler compatibility
- add `512 MiB` and `1 GiB` RAM payload tiers across core, npm, and pypi benchmark suites
- bencher:
  - upgrade default x86_64 instance to `m7i.4xlarge` (Intel Sapphire Rapids) for DDR5 parity with Graviton3 (`m7g.4xlarge`)
  - isolate benchmark target results by suite (`search_one` / `search_two`)
  - add AWS authentication setup guide
- update benchmark results and documentation across core, npm, and pypi packages

## [0.2.2] - 2026-08-29

- fix `Illegal instruction` crashes on x86_64 CPUs (machines w/o AVX-512BW ISA)
  - npm: build native addon with `target-cpu=native`
  - pypi: build wheels with `target-cpu=native`
- improvements in CI
  - rust: specify `-p ashwa` explicitly in publish step
  - npm:
    - pin package version during verification
    - run separate node and browser test suites
  - pypi:
    - pin package version during verification
    - add retry backoff for CDN indexing

## [0.2.1] - 2026-08-29

- Impl of `search_two`, the hardware-accelerated 2-byte substring search
  - Support for ISA targets: AVX-512BW, AVX2, SSE4.2, SSSE3, SSE2, NEON, SIMD128
  - 64-bit and 32-bit SWAR fallbacks for targets without vector ISAs
  - native Node.js (N-API), browser (WASM SIMD), and Python (CPython) bindings
  - throughput and latency profiling suites and tests across all supported ecosystems
- build x86_64 native npm and pypi release artifacts with AVX-512BW feature flags on nightly
- improved cross-platform testing in CI

## [0.2.0] - 2026-08-23

- optimize AArch64 NEON single-byte search with 256-byte loop unrolling across 16 vector registers
- replace criterion with custom high-precision throughput, latency, and ILP profiling harness
- add throughput benchmarks for npm (Node.js / WASM SIMD128) and Python packages
- add automated multi-architecture AWS EC2 benchmark runner with hardware PMU profiling

## [0.1.8] - 2026-08-19

- fix 32-bit i686 Linux Python wheel builds by enforcing manylinux2014 container environment

## [0.1.7] - 2026-08-19

- optimize release pipeline: sequence crates.io publish before parallel npm and pypi releases
- unify native addon and Python wheel builds into a single matrix runner per platform
- add `--find-interpreter` support to maturin for multi-arch wheel builds (including ARMv7 and i686 Linux)
- run full `node:test` and `pytest` suites on live published npm and pypi packages
- simplify and standardize CI job naming across release workflow

## [0.1.6] - 2026-08-19

- add multi-architecture pypi wheel build and release workflow (Linux x86_64/ARM64/ARMv7/i686, macOS
  Apple Silicon, Windows x86_64/ARM64)
- add Trusted Publishing (OIDC) with provenance attestations for pypi releases
- add post-publish verification across native platforms for pypi package

## [0.1.5] - 2026-08-18

- publish Python package to [pypi](https://pypi.org/project/ashwa/)

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
