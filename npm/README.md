[![npm](https://img.shields.io/npm/v/@pid7/ashwa?style=flat-square&logo=npm)](https://www.npmjs.com/package/@pid7/ashwa)
[![Crates.io](https://img.shields.io/crates/v/ashwa?style=flat-square&logo=rust)](https://crates.io/crates/ashwa)
[![Tests](https://img.shields.io/github/actions/workflow/status/pid7-org/ashwa/tests.yaml?style=flat-square&logo=github&label=tests)](https://github.com/pid7-org/ashwa/actions/workflows/tests.yaml)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue?style=flat-square)](../LICENSE-MIT)

# Ashwa 🐎

Hardware accelerated routines for single substring search

> **NOTE:**
> `@pid7/ashwa` is supported in both Node.js and browser environments. The optimal backend (native SIMD bindings for
> Node / Bun / Deno or WebAssembly SIMD for browsers) is automatically selected at runtime.

## Supported Platforms

| Architecture        | Target Platform                              | Hardware Acceleration                | Fallback    |
|:--------------------|:---------------------------------------------|:-------------------------------------|:------------|
| **x86_64**          | Linux, macOS, Windows                        | AVX-512BW, AVX2, SSE4.2, SSSE3, SSE2 | 64-bit SWAR |
| **AArch64 (ARM64)** | Apple Silicon, Linux ARM64                   | 128-bit ARM NEON                     | 64-bit SWAR |
| **WebAssembly**     | Browsers, Node.js (`wasm32`)                 | WASM SIMD128 (`simd128`)             | 32-bit SWAR |

## Usage

Install `@pid7/ashwa` via npm:

```bash
npm install @pid7/ashwa
```

## Example

### ESM & CommonJS (Node.js, Bun, Deno)

```javascript
import { searchOne } from '@pid7/ashwa';

const haystack = new TextEncoder().encode("Hello, World! Fast SIMD Search.");
const needle = "W".charCodeAt(0);

const index = searchOne(haystack, needle);
console.log(`Found 'W' at byte index: ${index}`);
```

### WASM (Browser / WebWorker)

> **INFO:**
> In browser/WebWorker environments, WASM automatically initializes on the first call.

```javascript
import { init, searchOne } from '@pid7/ashwa';

// Optional pre-initialization of WASM module during app startup
await init();

const index = searchOne(haystack, needle);
```
