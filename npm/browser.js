/**
 * Browser CommonJS entry point for `@pid7/ashwa` using WebAssembly SIMD bindings.
 *
 * @module @pid7/ashwa/browser
 */

import { readFileSync } from "fs";
import { join } from "path";

let wasmModule = null;
let initPromise = null;

/**
 * Asynchronously initializes the WebAssembly module.
 *
 * @param {any} [moduleOrPath] - Optional WebAssembly.Module, ArrayBuffer, or URL to load WASM from.
 * @returns {Promise<any>} Resolves with the loaded WASM module instance.
 */
async function init(moduleOrPath) {
  if (wasmModule) return wasmModule;
  if (!initPromise) {
    const wasm = require("./wasm/pkg/ashwa_wasm.js");
    let input = moduleOrPath;
    if (
      !input &&
      typeof process !== "undefined" &&
      process.versions != null &&
      process.versions.node != null
    ) {
      const wasmPath = join(__dirname, "./wasm/pkg/ashwa_wasm_bg.wasm");
      input = readFileSync(wasmPath);
    }
    const initOptions = input ? { module_or_path: input } : undefined;
    initPromise = Promise.resolve(wasm.default(initOptions)).then(() => {
      wasmModule = wasm;
      return wasm;
    });
  }
  await initPromise;
  return wasmModule;
}

/**
 * Synchronously initializes the WebAssembly module with bytes or module.
 *
 * @param {any} [bytesOrModule] - ArrayBuffer, Uint8Array, or WebAssembly.Module.
 */
function initSync(bytesOrModule) {
  const wasm = require("./wasm/pkg/ashwa_wasm.js");
  let input = bytesOrModule;
  if (
    !input &&
    typeof process !== "undefined" &&
    process.versions != null &&
    process.versions.node != null
  ) {
    const wasmPath = join(__dirname, "./wasm/pkg/ashwa_wasm_bg.wasm");
    input = readFileSync(wasmPath);
  }
  wasm.initSync(input);
  wasmModule = wasm;
}

/**
 * Searches for the first occurrence of `needle` in `haystack` via WebAssembly SIMD.
 * Automatically initializes WASM if not already loaded.
 *
 * @param {Uint8Array} haystack - Byte array to search.
 * @param {number} needle - Target byte (0-255).
 * @returns {Promise<number|null>} Resolves to 0-based matching index or null if not found.
 */
async function searchOne(haystack, needle) {
  if (!wasmModule) {
    await init();
  }
  const res = wasmModule.searchOne(haystack, needle);
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
async function searchTwo(haystack, needle) {
  const n = Array.isArray(needle) ? new Uint8Array(needle) : needle;
  if (n == null || n.length !== 2) {
    throw new TypeError("needle must be a 2-byte sequence");
  }
  if (!wasmModule) {
    await init();
  }
  const res = wasmModule.searchTwo(haystack, n);
  return res !== undefined && res !== null ? Number(res) : null;
}

export default {
  /**
   * `false` indicating WebAssembly execution mode.
   */
  isNative: false,
  init,
  initSync,
  searchOne,
  searchTwo,
};
