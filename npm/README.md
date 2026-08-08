# Ashwa 🐎

[![license](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

Hardware accelerated routines for single substring search

## Usage

ESM & CommonJS,

```javascript
import { searchOne } from '@pid7/ashwa';

const haystack = new TextEncoder().encode("Hello, World! Fast SIMD Search.");
const needle = "W".charCodeAt(0);

const index = await searchOne(haystack, needle);
console.log(`Found 'W' at byte index: ${index}`);
```

WASM (In browser),

> [!NOTE]
> In browser/WebWorker environments, WASM automatically initializes on the first call.

```javascript
import { init, searchOne } from '@pid7/ashwa';

// optional pre-initialize WASM module during app load
await init();

const index = await searchOne(haystack, needle);
```
