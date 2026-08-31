/**
 * Main ESM entry point for `@pid7/ashwa` in Node.js environments.
 *
 * @module @pid7/ashwa
 */

import { createRequire } from "module";

const require = createRequire(import.meta.url);
const native = require("./native/index.js");

/**
 * `true` when running on native N-API bindings, `false` when running on WebAssembly.
 */
export const isNative = true;

/**
 * Asynchronously pre-initializes the WebAssembly module. No-op on native Node.js.
 * @returns {Promise<void>}
 */
export async function init() {}

/**
 * Synchronously initializes the WebAssembly module. No-op on native Node.js.
 */
export function initSync() {}

/**
 * Searches for the first occurrence of `needle` in `haystack`.
 *
 * @param {Uint8Array} haystack - Byte array to search.
 * @param {number} needle - Target byte (0-255).
 * @returns {number|null} 0-based index or null if not found.
 */
export function searchOne(haystack, needle) {
  const res = native.searchOne(haystack, needle);
  return res !== undefined && res !== null ? Number(res) : null;
}

/**
 * Searches for the first occurrence of a two-byte `needle` in `haystack`.
 *
 * @param {Uint8Array} haystack - Byte array to search.
 * @param {Uint8Array|number[]} needle - 2-byte sequence to locate.
 * @returns {number|null} 0-based index or null if not found.
 */
export function searchTwo(haystack, needle) {
  const n = Array.isArray(needle) ? new Uint8Array(needle) : needle;
  if (n == null || n.length !== 2) {
    throw new TypeError("needle must be a 2-byte sequence");
  }

  const res = native.searchTwo(haystack, n);
  return res !== undefined && res !== null ? Number(res) : null;
}

/**
 * Searches for the first occurrence of a three-byte `needle` in `haystack`.
 *
 * @param {Uint8Array} haystack - Byte array to search.
 * @param {Uint8Array|number[]} needle - 3-byte sequence to locate.
 * @returns {number|null} 0-based index or null if not found.
 */
export function searchThree(haystack, needle) {
  const n = Array.isArray(needle) ? new Uint8Array(needle) : needle;
  if (n == null || n.length !== 3) {
    throw new TypeError("needle must be a 3-byte sequence");
  }

  const res = native.searchThree(haystack, n);
  return res !== undefined && res !== null ? Number(res) : null;
}

/**
 * Searches for the first occurrence of an arbitrary byte sequence `needle` in `haystack`.
 *
 * @param {Uint8Array} haystack - Byte array to search.
 * @param {Uint8Array|number[]} needle - Byte sequence to locate.
 * @returns {number|null} 0-based index or null if not found.
 */
export function searchN(haystack, needle) {
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

  const res = native.searchN(haystack, n);
  return res !== undefined && res !== null ? Number(res) : null;
}

