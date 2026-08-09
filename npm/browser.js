/**
 * Browser CommonJS entry point for `@pid7/ashwa` using WebAssembly SIMD bindings.
 *
 * @module @pid7/ashwa/browser
 */

const fs = require("fs");
const path = require("path");

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
      const wasmPath = path.join(__dirname, "./wasm/pkg/ashwa_wasm_bg.wasm");
      input = fs.readFileSync(wasmPath);
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
    const wasmPath = path.join(__dirname, "./wasm/pkg/ashwa_wasm_bg.wasm");
    input = fs.readFileSync(wasmPath);
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

module.exports = {
  /**
   * `false` indicating WebAssembly execution mode.
   */
  isNative: false,
  init,
  initSync,
  searchOne,
};
