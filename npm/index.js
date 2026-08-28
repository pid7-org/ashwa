/**
 * Main CommonJS entry point for `@pid7/ashwa`.
 * Automatically detects Node.js environment to load native bindings or fallback to browser WASM implementation.
 *
 * @module @pid7/ashwa
 */

const isNode =
  typeof process !== "undefined" &&
  process.versions != null &&
  process.versions.node != null;

if (isNode) {
  const native = require("./native/index.js");

  module.exports = {
    /**
     * `true` when running on native N-API bindings, `false` when running on WebAssembly.
     */
    isNative: true,

    /**
     * Asynchronously pre-initializes the WebAssembly module. No-op on native Node.js.
     * @returns {Promise<void>}
     */
    init: async () => {},

    /**
     * Synchronously initializes the WebAssembly module. No-op on native Node.js.
     */
    initSync: () => {},

    /**
     * Searches for the first occurrence of `needle` in `haystack`.
     *
     * @param {Uint8Array} haystack - Byte array to search.
     * @param {number} needle - Target byte (0-255).
     * @returns {number|null} 0-based index or null if not found.
     */
    searchOne(haystack, needle) {
      const res = native.searchOne(haystack, needle);
      return res !== undefined && res !== null ? Number(res) : null;
    },

    /**
     * Searches for the first occurrence of a two-byte `needle` in `haystack`.
     *
     * @param {Uint8Array} haystack - Byte array to search.
     * @param {Uint8Array|number[]} needle - 2-byte sequence to locate.
     * @returns {number|null} 0-based index or null if not found.
     */
    searchTwo(haystack, needle) {
      const n = Array.isArray(needle) ? new Uint8Array(needle) : needle;
      if (n == null || n.length !== 2) {
        throw new TypeError("needle must be a 2-byte sequence");
      }

      const res = native.searchTwo(haystack, n);
      return res !== undefined && res !== null ? Number(res) : null;
    },
  };
} else {
  module.exports = require("./browser.js");
}
