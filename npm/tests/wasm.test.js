const { test, describe } = require("node:test");
const assert = require("node:assert/strict");
const { searchOne, isNative, init, initSync } = require("../browser.js");

describe("WebAssembly Backend (wasm-bindgen SIMD128)", () => {
  test("Environment & exports verification", async () => {
    assert.strictEqual(isNative, false);
    assert.strictEqual(typeof searchOne, "function");
    assert.strictEqual(typeof init, "function");
    assert.strictEqual(typeof initSync, "function");

    await init();
  });

  test("Basic searchOne operations", async () => {
    const haystack = Buffer.from("abcdefghijklmnopqrstuvwxyz");

    assert.strictEqual(await searchOne(haystack, "a".charCodeAt(0)), 0);
    assert.strictEqual(await searchOne(haystack, "m".charCodeAt(0)), 12);
    assert.strictEqual(await searchOne(haystack, "z".charCodeAt(0)), 25);
    assert.strictEqual(await searchOne(haystack, "A".charCodeAt(0)), null);
  });

  test("Empty buffer search", async () => {
    const emptyUint8 = new Uint8Array(0);
    const emptyBuf = Buffer.alloc(0);

    assert.strictEqual(await searchOne(emptyUint8, 65), null);
    assert.strictEqual(await searchOne(emptyBuf, 65), null);
  });

  test("Multiple occurrences return first match", async () => {
    const haystack = Buffer.from("banana");
    assert.strictEqual(await searchOne(haystack, "a".charCodeAt(0)), 1);
    assert.strictEqual(await searchOne(haystack, "n".charCodeAt(0)), 2);
  });

  test("Binary data & extreme byte values (0x00 and 0xFF)", async () => {
    const data = new Uint8Array([0x10, 0x20, 0x00, 0x30, 0xff, 0x40]);

    assert.strictEqual(await searchOne(data, 0x00), 2);
    assert.strictEqual(await searchOne(data, 0xff), 4);
    assert.strictEqual(await searchOne(data, 0x99), null);
  });

  test("Buffer types compatibility (Buffer vs Uint8Array vs Subarray)", async () => {
    const rawArray = [10, 20, 30, 40, 50, 60, 70, 80];

    const uint8 = new Uint8Array(rawArray);
    const nodeBuf = Buffer.from(rawArray);
    const subarray = uint8.subarray(2, 6);

    assert.strictEqual(await searchOne(uint8, 50), 4);
    assert.strictEqual(await searchOne(nodeBuf, 50), 4);
    assert.strictEqual(await searchOne(subarray, 50), 2);
  });

  test("Unaligned byteOffset subarray searches", async () => {
    const memory = new ArrayBuffer(128);
    const view = new Uint8Array(memory, 7, 50);
    view.fill(0x55);
    view[19] = 0xaa;

    // NOTE: Unaligned buffer views must compute relative index correctly
    assert.strictEqual(await searchOne(view, 0xaa), 19);
  });

  test("WASM SIMD128 boundary & chunk sizes", async () => {
    const sizes = [1, 2, 3, 7, 8, 15, 16, 17, 31, 32, 33, 63, 64, 65, 127, 128, 255, 256];

    for (const size of sizes) {
      const buf = new Uint8Array(size);
      buf.fill(0xaa);

      buf[0] = 0xbb;
      assert.strictEqual(await searchOne(buf, 0xbb), 0);

      buf[0] = 0xaa;
      buf[size - 1] = 0xbb;
      assert.strictEqual(await searchOne(buf, 0xbb), size - 1);

      if (size > 2) {
        const mid = Math.floor(size / 2);
        buf[size - 1] = 0xaa;
        buf[mid] = 0xbb;
        assert.strictEqual(await searchOne(buf, 0xbb), mid);
      }
    }
  });

  test("Large payload search (1MB payload)", async () => {
    const size = 1024 * 1024;
    const buf = new Uint8Array(size);
    buf.fill(0x41);

    const targetIndices = [0, 15, 16, 31, 32, 63, 64, 5000, 50000, size - 1];

    for (const idx of targetIndices) {
      buf[idx] = 0x42;
      assert.strictEqual(await searchOne(buf, 0x42), idx);
      buf[idx] = 0x41;
    }
  });

  test("Randomized data correctness test", async () => {
    const size = 4096;
    const buf = new Uint8Array(size);

    for (let i = 0; i < size; i++) {
      buf[i] = (i * 31 + 7) % 255;
    }

    const testPositions = [0, 1, 15, 16, 30, 31, 32, 63, 64, 100, 511, 1023, 2047, 4095];
    for (const pos of testPositions) {
      buf[pos] = 255;
      assert.strictEqual(await searchOne(buf, 255), pos);
      buf[pos] = (pos * 31 + 7) % 255;
    }
  });
});
