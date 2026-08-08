/**
 * Searches for the first occurrence of `needle` in `haystack`.
 * 
 * - **Node.js**: Runs via `napi-rs` native addon (Synchronous CPU SIMD).
 * - **Browser / WASM**: Runs via `wasm-bindgen` WebAssembly (SIMD128 accelerated).
 *
 * @param haystack - The byte array/buffer to search.
 * @param needle - The byte value (0-255) to locate.
 * @returns The matching index, or `null` if not found.
 */
export function searchOne(
  haystack: Uint8Array,
  needle: number
): number | null | Promise<number | null>;

/**
 * Initializes the WebAssembly backend (Browser only).
 * In Node.js native mode, this returns immediately.
 */
export function init(moduleOrPath?: any): Promise<void>;

/**
 * Synchronously initializes the WebAssembly backend with a WebAssembly.Module or buffer.
 * In Node.js native mode, this is a no-op.
 */
export function initSync(bytesOrModule: any): void;

/**
 * `true` if running on Node native backend (napi-rs), `false` if running on WebAssembly (wasm-bindgen).
 */
export const isNative: boolean;
