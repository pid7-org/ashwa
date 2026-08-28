[![npm](https://img.shields.io/npm/v/@pid7/ashwa?style=flat-square&logo=npm)](https://www.npmjs.com/package/@pid7/ashwa)
[![Tests](https://img.shields.io/github/actions/workflow/status/pid7-org/ashwa/tests.yaml?style=flat-square&logo=github&label=tests)](https://github.com/pid7-org/ashwa/actions/workflows/tests.yaml)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue?style=flat-square)](../LICENSE-MIT)

# Ashwa 🐎

Hardware accelerated routines for single substring search

> 💡 TIP: `@pid7/ashwa` runs seamlessly across Node.js, Bun, Deno, and modern browser / WebWorker environments.

> ℹ️ NOTE: The optimal execution backend (native N-API SIMD bindings for Node / Bun / Deno or WebAssembly
> SIMD for browsers) is automatically detected and selected at runtime with zero manual configuration.

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
>   * Instance: Intel(R) Xeon(R) Platinum 8375C CPU @ 2.90GHz (8C/16T)
>   * ISA: _AVX-512BW_ (`+nightly`) · _WASM SIMD128_ (stable)
>   * Cache: L1d: 384 KiB · L1i: 256 KiB · L2: 10 MiB · L3: 54 MiB
>   * STREAM Triad: 20.94 GiB/s
>
> * AArch64 (_arm64_)
>   * Instance: AWS Graviton3 ARM Neoverse-V1 (16C/16T)
>   * ISA: _NEON_ (stable)
>   * Cache: L1d: 1 MiB · L1i: 1 MiB · L2: 16 MiB · L3: 32 MiB
>   * STREAM Triad: 75.12 GiB/s

### `searchOne`

#### Native Node.js (V8 N-API)

| Level      | Payload   | Latency (x64) | Latency (arm64) | Throughput (x64) | Throughput (arm64) |
|:-----------|:----------|:--------------|:----------------|:-----------------|:-------------------|
| L1 Cache   | 32 KiB    | 346.28 ns     | 743.33 ns       | 88.13 GiB/s      | 41.06 GiB/s        |
| L2 Cache   | 512 KiB   | 10.17 µs      | 11.25 µs        | 47.99 GiB/s      | 43.42 GiB/s        |
| L3 Cache   | 16 MiB    | 552.74 µs     | 381.99 µs       | 28.27 GiB/s      | 40.90 GiB/s        |
| RAM        | 256 MiB   | 20.22 ms      | 9.96 ms         | 12.36 GiB/s      | 25.10 GiB/s        |

#### WebAssembly (WASM SIMD128)

| Level      | Payload   | Latency (x64) | Throughput (x64) |
|:-----------|:----------|:--------------|:-----------------|
| L1 Cache   | 32 KiB    | 1.62 µs       | 18.82 GiB/s      |
| L2 Cache   | 512 KiB   | 27.88 µs      | 17.51 GiB/s      |
| L3 Cache   | 16 MiB    | 1.86 ms       | 8.38 GiB/s       |
| RAM        | 256 MiB   | 44.56 ms      | 5.61 GiB/s       |

### `searchTwo`

#### Native Node.js (V8 N-API)

| Level      | Payload   | Latency (x64) | Latency (arm64) | Throughput (x64) | Throughput (arm64) |
|:-----------|:----------|:--------------|:----------------|:-----------------|:-------------------|
| L1 Cache   | 32 KiB    | 647.01 ns     | 1.57 µs         | 47.17 GiB/s      | 19.44 GiB/s        |
| L2 Cache   | 512 KiB   | 17.41 µs      | 24.37 µs        | 28.04 GiB/s      | 20.03 GiB/s        |
| L3 Cache   | 16 MiB    | 657.09 µs     | 788.85 µs       | 23.78 GiB/s      | 19.81 GiB/s        |
| RAM        | 256 MiB   | 22.50 ms      | 13.13 ms        | 11.11 GiB/s      | 19.04 GiB/s        |

#### WebAssembly (WASM SIMD128)

| Level      | Payload   | Latency (x64) | Throughput (x64) |
|:-----------|:----------|:--------------|:-----------------|
| L1 Cache   | 32 KiB    | 2.17 µs       | 14.06 GiB/s      |
| L2 Cache   | 512 KiB   | 35.10 µs      | 13.91 GiB/s      |
| L3 Cache   | 16 MiB    | 3.69 ms       | 4.23 GiB/s       |
| RAM        | 256 MiB   | 51.67 ms      | 4.84 GiB/s       |
