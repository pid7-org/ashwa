/**
 * Browser ESM entry point for `@pid7/ashwa` using WebAssembly SIMD bindings.
 *
 * @module @pid7/ashwa/browser
 */

import fs from "node:fs";
import { fileURLToPath } from "node:url";
import initWasm, {
  initSync as wasmInitSync,
  searchOne as wasmSearchOne,
  searchTwo as wasmSearchTwo,
  searchThree as wasmSearchThree,
  searchN as wasmSearchN,
} from "./wasm/pkg/ashwa_wasm.js";

let isInitialized = false;
let initPromise = null;

/**
 * `false` indicating WebAssembly execution mode.
 */
export const isNative = false;

/**
 * Asynchronously initializes the WebAssembly module.
 *
 * @param {any} [moduleOrPath] - Optional WebAssembly.Module, ArrayBuffer, or URL to load WASM from.
 * @returns {Promise<void>} Resolves when initialization completes.
 */
export async function init(moduleOrPath) {
  if (isInitialized) return;
  if (!initPromise) {
    let input = moduleOrPath;
    if (
      !input &&
      typeof process !== "undefined" &&
      process.versions != null &&
      process.versions.node != null
    ) {
      const wasmPath = fileURLToPath(
        new URL("./wasm/pkg/ashwa_wasm_bg.wasm", import.meta.url),
      );
      input = fs.readFileSync(wasmPath);
    }

    const options = input ? { module_or_path: input } : undefined;
    initPromise = initWasm(options).then(() => {
      isInitialized = true;
    });
  }

  await initPromise;
}

/**
 * Synchronously initializes the WebAssembly module with bytes or module.
 *
 * @param {any} [bytesOrModule] - ArrayBuffer, Uint8Array, or WebAssembly.Module.
 */
export function initSync(bytesOrModule) {
  let input = bytesOrModule;
  if (
    !input &&
    typeof process !== "undefined" &&
    process.versions != null &&
    process.versions.node != null
  ) {
    const wasmPath = fileURLToPath(
      new URL("./wasm/pkg/ashwa_wasm_bg.wasm", import.meta.url),
    );

    input = fs.readFileSync(wasmPath);
  }

  const options = input ? (input.module != null ? input : { module: input }) : undefined;
  wasmInitSync(options);
  isInitialized = true;
}

/**
 * Searches for the first occurrence of `needle` in `haystack` via WebAssembly SIMD.
 * Automatically initializes WASM if not already loaded.
 *
 * @param {Uint8Array} haystack - Byte array to search.
 * @param {number} needle - Target byte (0-255).
 * @returns {Promise<number|null>} Resolves to 0-based matching index or null if not found.
 */
export async function searchOne(haystack, needle) {
  if (!isInitialized) {
    await init();
  }

  const res = wasmSearchOne(haystack, needle);
  return res !== undefined && res !== null ? Number(res) : null;
}

/**
 * Searches for the first occurrence of a two-byte `needle` in `haystack` via WebAssembly SIMD.
 * Automatically initializes WASM if not already loaded.
 *
 * @param {Uint8Array} haystack - Byte array to search.
 * @param {Uint8Array|number[]} needle - 2-byte sequence to locate.
 * @returns {Promise<number|null>} Resolves to 0-based matching index or null if not found.
 */
export async function searchTwo(haystack, needle) {
  const n = Array.isArray(needle) ? new Uint8Array(needle) : needle;
  if (n == null || n.length !== 2) {
    throw new TypeError("needle must be a 2-byte sequence");
  }

  if (!isInitialized) {
    await init();
  }

  const res = wasmSearchTwo(haystack, n);
  return res !== undefined && res !== null ? Number(res) : null;
}

/**
 * Searches for the first occurrence of a three-byte `needle` in `haystack` via WebAssembly SIMD.
 * Automatically initializes WASM if not already loaded.
 *
 * @param {Uint8Array} haystack - Byte array to search.
 * @param {Uint8Array|number[]} needle - 3-byte sequence to locate.
 * @returns {Promise<number|null>} Resolves to 0-based matching index or null if not found.
 */
export async function searchThree(haystack, needle) {
  const n = Array.isArray(needle) ? new Uint8Array(needle) : needle;
  if (n == null || n.length !== 3) {
    throw new TypeError("needle must be a 3-byte sequence");
  }

  if (!isInitialized) {
    await init();
  }

  const res = wasmSearchThree(haystack, n);
  return res !== undefined && res !== null ? Number(res) : null;
}

/**
 * Searches for the first occurrence of an arbitrary byte sequence `needle` in `haystack` via WebAssembly SIMD.
 * Automatically initializes WASM if not already loaded.
 *
 * @param {Uint8Array} haystack - Byte array to search.
 * @param {Uint8Array|number[]} needle - Byte sequence to locate.
 * @returns {Promise<number|null>} Resolves to 0-based matching index or null if not found.
 */
export async function searchN(haystack, needle) {
  const n = Array.isArray(needle) ? new Uint8Array(needle) : needle;
  if (
    n == null ||
    !(
      n instanceof Uint8Array ||
      (typeof Buffer !== "undefined" && Buffer.isBuffer(n))
    )
  ) {
    throw new TypeError("needle must be a Uint8Array, Buffer, or byte array");
  }

  if (!isInitialized) {
    await init();
  }

  const res = wasmSearchN(haystack, n);
  return res !== undefined && res !== null ? Number(res) : null;
}

