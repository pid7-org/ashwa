[![npm](https://img.shields.io/npm/v/@pid7/ashwa?style=flat-square&logo=npm)](https://www.npmjs.com/package/@pid7/ashwa)
[![Crates.io](https://img.shields.io/crates/v/ashwa?style=flat-square&logo=rust)](https://crates.io/crates/ashwa)
[![Tests](https://img.shields.io/github/actions/workflow/status/pid7-org/ashwa/tests.yaml?style=flat-square&logo=github&label=tests)](https://github.com/pid7-org/ashwa/actions/workflows/tests.yaml)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue?style=flat-square)](../LICENSE-MIT)

# Ashwa 🐎

Hardware accelerated routines for single substring search

> [!TIP]
> Package is supported in both Node.js and browser environments.

> [!NOTE]
> The optimal backend (native SIMD bindings for Node / Bun / Deno or WebAssembly SIMD for browsers) is
> automatically selected at runtime.

## Supported Platforms

| Architecture    | Target Platform                              | Hardware Acceleration                | Fallback    |
|:----------------|:---------------------------------------------|:-------------------------------------|:------------|
| x86_64          | Linux, macOS, Windows                        | AVX-512BW, AVX2, SSE4.2, SSSE3, SSE2 | 64-bit SWAR |
| AArch64 (ARM64) | Apple Silicon, Linux ARM64                   | 128-bit ARM NEON                     | 64-bit SWAR |
| WebAssembly     | Browsers, Node.js (`wasm32`)                 | WASM SIMD128 (`simd128`)             | 32-bit SWAR |

## Usage

Install `@pid7/ashwa` via npm:

```bash
npm install @pid7/ashwa
```

## Example

### ESM & CommonJS (Node.js, Bun, Deno)

```javascript
import { searchOne, searchTwo } from '@pid7/ashwa';

const haystack = new TextEncoder().encode("The quick brown fox jumps over the lazy dog");

const indexOne = searchOne(haystack, "f".charCodeAt(0));
console.log(`Found 'f' at byte index: ${indexOne}`); // 16
```

### WASM (Browser / WebWorker)

> **INFO:**
> In browser/WebWorker environments, WASM automatically initializes on the first call.

```javascript
import { init, searchOne} from '@pid7/ashwa';

await init(); // optional pre-initialization of WASM module during app startup
const indexOne = await searchOne(haystack, needleByte);
```

## Benchmarks

- [`searchOne`](#searchone)
- [`searchTwo`](#searchtwo)

### `searchOne`

#### Native Node.js (V8 N-API)

For _x86_64_ machine targeting _AVX-512BW_ SIMD ISA,

| Level      | Payload   | Latency   | Throughput   |
|:-----------|:----------|:----------|:-------------|
| L1 Cache   | 32 KiB    | 346.28 ns | 88.13 GiB/s  |
| L2 Cache   | 512 KiB   | 10.17 µs  | 47.99 GiB/s  |
| L3 Cache   | 16 MiB    | 552.74 µs | 28.27 GiB/s  |
| RAM        | 256 MiB   | 20.22 ms  | 12.36 GiB/s  |

Benchmarked using Intel(R) Xeon(R) Platinum 8375C CPU @ 2.90GHz (8C/16T) · L1d: 384 KiB, L1i: 256 KiB, L2: 10 MiB, L3: 54 MiB ·
STREAM Triad: 20.94 GB/s · Node.js v22.23.2

For _aarch64_ machine targeting _NEON_ SIMD ISA,

| Level      | Payload   | Latency   | Throughput   |
|:-----------|:----------|:----------|:-------------|
| L1 Cache   | 32 KiB    | 743.33 ns | 41.06 GiB/s  |
| L2 Cache   | 512 KiB   | 11.25 µs  | 43.42 GiB/s  |
| L3 Cache   | 16 MiB    | 381.99 µs | 40.90 GiB/s  |
| RAM        | 256 MiB   | 9.96 ms   | 25.10 GiB/s  |

Benchmarked using ARM Neoverse-V1 (16C/16T) · L1d: 1 MiB, L1i: 1 MiB, L2: 16 MiB, L3: 32 MiB ·
STREAM Triad: 75.12 GB/s · Node.js v22.23.2

#### WebAssembly (WASM SIMD128)

For _x86_64_ machine targeting _WASM SIMD128_ ISA,

| Level      | Payload   | Latency   | Throughput   |
|:-----------|:----------|:----------|:-------------|
| L1 Cache   | 32 KiB    | 1.62 µs   | 18.82 GiB/s  |
| L2 Cache   | 512 KiB   | 27.88 µs  | 17.51 GiB/s  |
| L3 Cache   | 16 MiB    | 1.86 ms   | 8.38 GiB/s   |
| RAM        | 256 MiB   | 44.56 ms  | 5.61 GiB/s   |

Benchmarked using Intel(R) Xeon(R) Platinum 8375C CPU @ 2.90GHz (8C/16T) ·
L1d: 384 KiB, L1i: 256 KiB, L2: 10 MiB, L3: 54 MiB · STREAM Triad: 20.94 GB/s · Node.js v22.23.2

### `searchTwo`

#### Native Node.js (V8 N-API)

For _x86_64_ machine targeting _AVX-512BW_ SIMD ISA,

| Level      | Payload   | Latency   | Throughput   |
|:-----------|:----------|:----------|:-------------|
| L1 Cache   | 32 KiB    | 647.01 ns | 47.17 GiB/s  |
| L2 Cache   | 512 KiB   | 17.41 µs  | 28.04 GiB/s  |
| L3 Cache   | 16 MiB    | 657.09 µs | 23.78 GiB/s  |
| RAM        | 256 MiB   | 22.50 ms  | 11.11 GiB/s  |

Benchmarked using Intel(R) Xeon(R) Platinum 8375C CPU @ 2.90GHz (8C/16T) · L1d: 384 KiB, L1i: 256 KiB, L2: 10 MiB, L3: 54 MiB ·
STREAM Triad: 19.34 GB/s · Node.js v22.23.2

For _aarch64_ machine targeting _NEON_ SIMD ISA,

| Level      | Payload   | Latency   | Throughput   |
|:-----------|:----------|:----------|:-------------|
| L1 Cache   | 32 KiB    | 1.57 µs   | 19.44 GiB/s  |
| L2 Cache   | 512 KiB   | 24.37 µs  | 20.03 GiB/s  |
| L3 Cache   | 16 MiB    | 788.85 µs | 19.81 GiB/s  |
| RAM        | 256 MiB   | 13.13 ms  | 19.04 GiB/s  |

Benchmarked using ARM Neoverse-V1 (16C/16T) · L1d: 1 MiB, L1i: 1 MiB, L2: 16 MiB, L3: 32 MiB ·
STREAM Triad: 75.88 GB/s · Node.js v22.23.2

#### WebAssembly (WASM SIMD128)

For _x86_64_ machine targeting _WASM SIMD128_ ISA,

| Level      | Payload   | Latency   | Throughput   |
|:-----------|:----------|:----------|:-------------|
| L1 Cache   | 32 KiB    | 2.17 µs   | 14.06 GiB/s  |
| L2 Cache   | 512 KiB   | 35.10 µs  | 13.91 GiB/s  |
| L3 Cache   | 16 MiB    | 3.69 ms   | 4.23 GiB/s   |
| RAM        | 256 MiB   | 51.67 ms  | 4.84 GiB/s   |

Benchmarked using Intel(R) Xeon(R) Platinum 8375C CPU @ 2.90GHz (8C/16T) ·
L1d: 384 KiB, L1i: 256 KiB, L2: 10 MiB, L3: 54 MiB · STREAM Triad: 19.34 GB/s · Node.js v22.23.2
