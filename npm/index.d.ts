/**
 * Hardware-accelerated search routines (`@pid7/ashwa`).
 *
 * @module @pid7/ashwa
 */

/**
 * Searches for the first occurrence of `needle` (byte value 0-255) within `haystack` (Uint8Array / Buffer).
 *
 * Execution backend:
 * - **Node.js / Bun / Deno**: Executes synchronously via N-API native bindings with SIMD vectorization (AVX-512BW, AVX2, SSE2, ARM NEON).
 * - **Browser / WebWorker**: Executes via WebAssembly SIMD (`simd128`), automatically initializing the WASM module on first invocation if not already loaded.
 *
 * @param haystack - The byte array/buffer to search.
 * @param needle - The target byte value (0–255) to locate.
 * @returns The 0-based byte index of the first occurrence of `needle`, or `null` if not found.
 *
 * @example
 * ```javascript
 * import { searchOne } from '@pid7/ashwa';
 *
 * const haystack = new TextEncoder().encode("Hello, World!");
 * const index = await searchOne(haystack, "W".charCodeAt(0));
 * console.log(index); // 7
 * ```
 */
export function searchOne(
  haystack: Uint8Array,
  needle: number,
): number | null | Promise<number | null>;

/**
 * Searches for the first occurrence of a two-byte `needle` within `haystack` (Uint8Array / Buffer).
 *
 * Execution backend:
 * - **Node.js / Bun / Deno**: Executes synchronously via N-API native bindings with SIMD vectorization (AVX-512BW, AVX2, SSE4.2, SSSE3, SSE2, ARM NEON).
 * - **Browser / WebWorker**: Executes via WebAssembly SIMD (`simd128`), automatically initializing the WASM module on first invocation if not already loaded.
 *
 * @param haystack - The byte array/buffer to search.
 * @param needle - A 2-byte sequence (`Uint8Array`, `Buffer`, or `[number, number]` / `number[]`) to locate.
 * @returns The 0-based byte index of the first occurrence of `needle`, or `null` if not found.
 *
 * @example
 * ```javascript
 * import { searchTwo } from '@pid7/ashwa';
 *
 * const haystack = new TextEncoder().encode("Hello, World!");
 * const needle = new Uint8Array(["W".charCodeAt(0), "o".charCodeAt(0)]);
 * const index = await searchTwo(haystack, needle);
 * console.log(index); // 7
 * ```
 */
export function searchTwo(
  haystack: Uint8Array,
  needle: Uint8Array | [number, number] | number[],
): number | null | Promise<number | null>;

/**
 * Asynchronously pre-initializes the WebAssembly module backend (Browser / WebWorker).
 *
 * In Node.js native mode (`isNative === true`), this is a no-op that resolves immediately.
 *
 * @param moduleOrPath - Optional WebAssembly.Module, Response, ArrayBuffer, or fetch URL path to load the WASM binary from.
 * @returns A promise that resolves when initialization completes.
 *
 * @example
 * ```javascript
 * import { init, searchOne } from '@pid7/ashwa';
 *
 * // Pre-initialize WASM during application bootstrap
 * await init();
 * ```
 */
export function init(moduleOrPath?: any): Promise<void>;

/**
 * Synchronously initializes the WebAssembly module backend using pre-loaded byte data or a compiled `WebAssembly.Module`.
 *
 * In Node.js native mode (`isNative === true`), this is a no-op.
 *
 * @param bytesOrModule - An `ArrayBuffer`, `Uint8Array`, or compiled `WebAssembly.Module`.
 *
 * @example
 * ```javascript
 * import { initSync } from '@pid7/ashwa';
 *
 * initSync(wasmBuffer);
 * ```
 */
export function initSync(bytesOrModule?: any): void;

/**
 * Boolean flag indicating whether `@pid7/ashwa` is running via native N-API bindings (`true`)
 * or WebAssembly SIMD (`false`).
 */
export const isNative: boolean;
