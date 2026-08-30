[![npm](https://img.shields.io/npm/v/@pid7/ashwa?style=flat-square&logo=npm)](https://www.npmjs.com/package/@pid7/ashwa)
[![Tests](https://img.shields.io/github/actions/workflow/status/pid7-org/ashwa/tests.yaml?style=flat-square&logo=github&label=tests)](https://github.com/pid7-org/ashwa/actions/workflows/tests.yaml)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue?style=flat-square)](../LICENSE-MIT)

# Ashwa 🐎

Hardware accelerated routines for single substring search

> 💡 TIP:
> `@pid7/ashwa` runs seamlessly across Node.js, Bun, Deno, and modern browser / WebWorker environments.

> ℹ️ NOTE:
> The optimal execution backend (native N-API SIMD bindings for Node / Bun / Deno or WebAssembly SIMD for browsers)
> is automatically detected and selected at runtime with zero manual configuration.

## Index

- [Supported Platforms](#supported-platforms)
- [Installation](#installation)
- [Usage](#usage)
  - [Node](#node)
  - [Browser](#browser)
- [API Reference](#api-reference)
  - [`searchOne`](#searchone)
  - [`searchTwo`](#searchtwo)
  - [`init`](#init)
  - [`initSync`](#initsync)
  - [`isNative`](#isnative)
- [Benchmarks](#benchmarks)
  - [`searchOne`](#searchone-1)
  - [`searchTwo`](#searchtwo-1)

## Supported Platforms

| Architecture    | Target Platform                        | Target ISA                           | Fallback    |
|:----------------|:---------------------------------------|:-------------------------------------|:------------|
| x86_64          | Linux, macOS, Windows                  | AVX-512BW, AVX2, SSE4.2, SSSE3, SSE2 | 64-bit SWAR |
| AArch64 (ARM64) | Apple Silicon, Linux ARM64             | 128-bit ARM NEON                     | 64-bit SWAR |
| WebAssembly     | Browsers, WebWorkers, Node.js (wasm32) | WASM SIMD128 (`simd128`)             | 32-bit SWAR |

## Installation

Install `@pid7/ashwa` using your preferred package manager:

```bash
npm install @pid7/ashwa
```

## Usage

### Node

Native N-API bindings execute _synchronously_ with zero overhead and full hardware SIMD acceleration,

```js
import { searchOne, searchTwo } from '@pid7/ashwa';

const haystack = new TextEncoder().encode("The quick brown fox jumps over the lazy dog");
const indexOne = searchOne(haystack, "f".charCodeAt(0));
console.log(`Found 'f' at byte index: ${indexOne}`); // 16
```

### Browser

In browser and WebWorker runtimes, WebAssembly SIMD (`simd128`) is used. The WASM module automatically initializes asynchronously on the first search call, or can be pre-warmed during application startup:

```js
import { init, searchOne, searchTwo } from '@pid7/ashwa';

// Optional pre-init
await init();

const haystack = new TextEncoder().encode("The quick brown fox jumps over the lazy dog");
const index = await searchOne(haystack, "f".charCodeAt(0));
console.log(`Found 'f' at byte index: ${index}`); // 16
```

## API Reference

### `searchOne(haystack, needle)`

```ts
function searchOne(
  haystack: Uint8Array,
  needle: number,
): number | null | Promise<number | null>;
```

Searches for the first occurrence of `needle` (byte value `0`–`255`) within `haystack`.

- Parameters:
  - `haystack`: `Uint8Array` / `Buffer` — The byte sequence to search.
  - `needle`: `number` — The target byte value (`0`–`255`) to locate.
- Returns:
  - In Node.js / Bun / Deno: `number | null` (synchronous 0-based byte index, or `null` if not found).
  - In Browser / WebWorker: `Promise<number | null>` (resolves to 0-based byte index, or `null` if not found).

### `searchTwo(haystack, needle)`

```ts
function searchTwo(
  haystack: Uint8Array,
  needle: Uint8Array | [number, number] | number[],
): number | null | Promise<number | null>;
```

Searches for the first occurrence of a two-byte `needle` within `haystack`.

- Parameters:
  - `haystack`: `Uint8Array` / `Buffer` — The byte sequence to search.
  - `needle`: `Uint8Array | [number, number] | number[]` — A 2-byte sequence to locate.
- Returns:
  - In Node.js / Bun / Deno: `number | null` (synchronous 0-based byte index, or `null` if not found).
  - In Browser / WebWorker: `Promise<number | null>` (resolves to 0-based byte index, or `null` if not found).

### `init(moduleOrPath?)`

```typescript
function init(moduleOrPath?: any): Promise<void>;
```

Asynchronously pre-initializes the WebAssembly module backend for Browser / WebWorker environments. In native Node.js environments (`isNative === true`), this is a no-op that resolves immediately.

- Parameters:
  - `moduleOrPath` *(optional)*: `WebAssembly.Module | Response | ArrayBuffer | string` — Custom WASM source or URL.

### `initSync(bytesOrModule?)`

```ts
function initSync(bytesOrModule?: any): void;
```

Synchronously initializes the WebAssembly module backend with pre-compiled bytes or a `WebAssembly.Module`. In native Node.js environments, this is a no-op.

- Parameters:
  - `bytesOrModule` *(optional)*: `ArrayBuffer | Uint8Array | WebAssembly.Module` — Pre-loaded WASM binary or module.

### `isNative`

```typescript
const isNative: boolean;
```

Boolean flag indicating whether `@pid7/ashwa` is executing via native N-API bindings (`true`) or WebAssembly SIMD (`false`).

## Benchmarks

- [`searchOne`](#searchone-1)
- [`searchTwo`](#searchtwo-1)

> Benchmarks are evaluated across dedicated AWS EC2 hardware environments on Node.js `v22.23.2`,
>
> * x86_64 (_x64_)
>   * Instance: Intel(R) Xeon(R) Platinum 8488C (8C/16T)
>   * ISA: _AVX-512BW_ · _WASM SIMD128_
>   * Cache: L1d: 384 KiB · L1i: 256 KiB · L2: 16 MiB · L3: 105 MiB
>   * STREAM Triad: 25.76 GiB/s
>
> * AArch64 (_arm64_)
>   * Instance: AWS Graviton3 ARM Neoverse-V1 (16C/16T)
>   * ISA: _NEON_
>   * Cache: L1d: 1 MiB · L1i: 1 MiB · L2: 16 MiB · L3: 32 MiB
>   * STREAM Triad: 75.48 GiB/s

### `searchOne`

#### Native Node.js (V8 N-API)

| Level      | Payload   | Latency (x64) | Latency (arm64) | Throughput (x64) | Throughput (arm64) |
|:-----------|:----------|:--------------|:----------------|:-----------------|:-------------------|
| L1 Cache   | 32 KiB    | 259.88 ns     | 730.67 ns       | 117.43 GiB/s     | 41.77 GiB/s        |
| L2 Cache   | 512 KiB   | 6.90 µs       | 11.15 µs        | 70.76 GiB/s      | 43.78 GiB/s        |
| L3 Cache   | 16 MiB    | 488.06 µs     | 376.89 µs       | 32.01 GiB/s      | 41.46 GiB/s        |
| RAM        | 256 MiB   | 20.19 ms      | 9.84 ms         | 12.38 GiB/s      | 25.40 GiB/s        |
| RAM        | 512 MiB   | 41.79 ms      | 19.60 ms        | 11.96 GiB/s      | 25.52 GiB/s        |
| RAM        | 1 GiB     | 83.68 ms      | 39.11 ms        | 11.95 GiB/s      | 25.57 GiB/s        |

#### WebAssembly (WASM SIMD128)

| Level      | Payload   | Latency (x64) | Throughput (x64) |
|:-----------|:----------|:--------------|:-----------------|
| L1 Cache   | 32 KiB    | 1.28 µs       | 23.88 GiB/s      |
| L2 Cache   | 512 KiB   | 20.77 µs      | 23.51 GiB/s      |
| L3 Cache   | 16 MiB    | 1.53 ms       | 10.22 GiB/s      |
| RAM        | 256 MiB   | 48.19 ms      | 5.19 GiB/s       |
| RAM        | 512 MiB   | 96.64 ms      | 5.17 GiB/s       |
| RAM        | 1 GiB     | 192.52 ms     | 5.19 GiB/s       |

### `searchTwo`

#### Native Node.js (V8 N-API)

| Level      | Payload   | Latency (x64) | Latency (arm64) | Throughput (x64) | Throughput (arm64) |
|:-----------|:----------|:--------------|:----------------|:-----------------|:-------------------|
| L1 Cache   | 32 KiB    | 563.60 ns     | 1.57 µs         | 54.15 GiB/s      | 19.38 GiB/s        |
| L2 Cache   | 512 KiB   | 12.14 µs      | 23.93 µs        | 40.22 GiB/s      | 20.40 GiB/s        |
| L3 Cache   | 16 MiB    | 558.19 µs     | 771.07 µs       | 27.99 GiB/s      | 20.26 GiB/s        |
| RAM        | 256 MiB   | 20.05 ms      | 13.03 ms        | 12.47 GiB/s      | 19.18 GiB/s        |
| RAM        | 512 MiB   | 41.59 ms      | 26.07 ms        | 12.02 GiB/s      | 19.18 GiB/s        |
| RAM        | 1 GiB     | 83.70 ms      | 52.08 ms        | 11.95 GiB/s      | 19.20 GiB/s        |

#### WebAssembly (WASM SIMD128)

| Level      | Payload   | Latency (x64) | Throughput (x64) |
|:-----------|:----------|:--------------|:-----------------|
| L1 Cache   | 32 KiB    | 1.87 µs       | 16.31 GiB/s      |
| L2 Cache   | 512 KiB   | 28.72 µs      | 17.00 GiB/s      |
| L3 Cache   | 16 MiB    | 1.80 ms       | 8.69 GiB/s       |
| RAM        | 256 MiB   | 53.44 ms      | 4.68 GiB/s       |
| RAM        | 512 MiB   | 106.71 ms     | 4.69 GiB/s       |
| RAM        | 1 GiB     | 213.08 ms     | 4.69 GiB/s       |
