const { test, describe } = require("node:test");
const { strictEqual, throws } = require("node:assert/strict");
const { searchOne, searchTwo, isNative, init, initSync } = require("../index.js");

describe("Node Native Backend (napi-rs)", () => {
  test("Environment & exports verification", async () => {
    strictEqual(isNative, true);
    strictEqual(typeof searchOne, "function");
    strictEqual(typeof searchTwo, "function");
    strictEqual(typeof init, "function");
    strictEqual(typeof initSync, "function");

    await init();
    initSync();
  });

  test("Basic searchOne operations", () => {
    const haystack = Buffer.from("abcdefghijklmnopqrstuvwxyz");

    strictEqual(searchOne(haystack, "a".charCodeAt(0)), 0);
    strictEqual(searchOne(haystack, "m".charCodeAt(0)), 12);
    strictEqual(searchOne(haystack, "z".charCodeAt(0)), 25);
    strictEqual(searchOne(haystack, "A".charCodeAt(0)), null);
  });

  test("Basic searchTwo operations", () => {
    const haystack = Buffer.from("abcdefghijklmnopqrstuvwxyz");

    strictEqual(searchTwo(haystack, Buffer.from("ab")), 0);
    strictEqual(searchTwo(haystack, Buffer.from("mn")), 12);
    strictEqual(searchTwo(haystack, Buffer.from("yz")), 24);
    strictEqual(searchTwo(haystack, Buffer.from("AB")), null);
    strictEqual(searchTwo(haystack, [0x61, 0x62]), 0);
  });

  test("Pangram spot-checks for searchTwo", () => {
    const haystack = Buffer.from("the quick brown fox jumps over the lazy dog");

    strictEqual(searchTwo(haystack, Buffer.from("th")), 0);
    strictEqual(searchTwo(haystack, Buffer.from("he")), 1);
    strictEqual(searchTwo(haystack, Buffer.from("qu")), 4);
    strictEqual(searchTwo(haystack, Buffer.from("ox")), 17);
    strictEqual(searchTwo(haystack, Buffer.from("do")), 40);
    strictEqual(searchTwo(haystack, Buffer.from("og")), 41);
    strictEqual(searchTwo(haystack, Buffer.from("ZZ")), null);
    strictEqual(searchTwo(haystack, Buffer.from("!!")), null);
  });

  test("Empty buffer search", () => {
    const emptyUint8 = new Uint8Array(0);
    const emptyBuf = Buffer.alloc(0);

    strictEqual(searchOne(emptyUint8, 65), null);
    strictEqual(searchOne(emptyBuf, 65), null);

    strictEqual(searchTwo(emptyUint8, Buffer.from("ab")), null);
    strictEqual(searchTwo(emptyBuf, Buffer.from("ab")), null);

    const singleByte = Buffer.from("a");
    strictEqual(searchTwo(singleByte, Buffer.from("ab")), null);
  });

  test("Multiple occurrences return first match", () => {
    const haystack = Buffer.from("banana");
    strictEqual(searchOne(haystack, "a".charCodeAt(0)), 1);
    strictEqual(searchOne(haystack, "n".charCodeAt(0)), 2);

    strictEqual(searchTwo(haystack, Buffer.from("an")), 1);
    strictEqual(searchTwo(haystack, Buffer.from("na")), 2);
  });

  test("Binary data & extreme byte values (0x00 and 0xFF)", () => {
    const data = new Uint8Array([0x10, 0x20, 0x00, 0x30, 0xff, 0x40]);

    strictEqual(searchOne(data, 0x00), 2);
    strictEqual(searchOne(data, 0xff), 4);
    strictEqual(searchOne(data, 0x99), null);

    const binaryData = new Uint8Array([
      0x10, 0x00, 0x00, 0x30, 0xfe, 0xff, 0x40,
    ]);
    strictEqual(searchTwo(binaryData, new Uint8Array([0x00, 0x00])), 1);
    strictEqual(searchTwo(binaryData, new Uint8Array([0xfe, 0xff])), 4);
    strictEqual(searchTwo(binaryData, new Uint8Array([0xff, 0xff])), null);
  });

  test("Overlapping & repeating patterns", () => {
    strictEqual(searchTwo(Buffer.from("aaaaaa"), Buffer.from("aa")), 0);
    strictEqual(searchTwo(Buffer.from("baaaaa"), Buffer.from("aa")), 1);
    strictEqual(searchTwo(Buffer.from("bbaaaa"), Buffer.from("aa")), 2);
    strictEqual(searchTwo(Buffer.from("ababab"), Buffer.from("ab")), 0);
    strictEqual(searchTwo(Buffer.from("bababa"), Buffer.from("ab")), 1);

    const allA = Buffer.alloc(256, "A");
    strictEqual(searchTwo(allA, Buffer.from("AB")), null);
    allA[120] = "B".charCodeAt(0);
    strictEqual(searchTwo(allA, Buffer.from("AB")), 119);
  });

  test("Buffer types compatibility (Buffer vs Uint8Array vs Subarray)", () => {
    const rawArray = [10, 20, 30, 40, 50, 60, 70, 80];

    const uint8 = new Uint8Array(rawArray);
    const nodeBuf = Buffer.from(rawArray);
    const subarray = uint8.subarray(2, 6);

    strictEqual(searchOne(uint8, 50), 4);
    strictEqual(searchOne(nodeBuf, 50), 4);
    strictEqual(searchOne(subarray, 50), 2);

    strictEqual(searchTwo(uint8, new Uint8Array([50, 60])), 4);
    strictEqual(searchTwo(nodeBuf, Buffer.from([50, 60])), 4);
    strictEqual(searchTwo(subarray, [50, 60]), 2);
  });

  test("Unaligned byteOffset subarray searches", () => {
    const memory = new ArrayBuffer(128);
    const view = new Uint8Array(memory, 7, 50);
    view.fill(0x55);
    view[19] = 0xaa;
    view[20] = 0xbb;

    // NOTE: Unaligned buffer views must compute relative index correctly
    strictEqual(searchOne(view, 0xaa), 19);
    strictEqual(searchTwo(view, new Uint8Array([0xaa, 0xbb])), 19);
  });

  test("SIMD boundary & chunk sizes", () => {
    const sizes = [
      1, 2, 3, 7, 8, 9, 15, 16, 17, 24, 25, 31, 32, 33, 34, 63, 64, 65, 127,
      128, 255, 256,
    ];

    for (const size of sizes) {
      const buf = new Uint8Array(size);
      buf.fill(0xaa);

      buf[0] = 0xbb;
      strictEqual(searchOne(buf, 0xbb), 0);

      buf[0] = 0xaa;
      buf[size - 1] = 0xbb;
      strictEqual(searchOne(buf, 0xbb), size - 1);

      if (size > 2) {
        const mid = Math.floor(size / 2);
        buf[size - 1] = 0xaa;
        buf[mid] = 0xbb;
        strictEqual(searchOne(buf, 0xbb), mid);
      }
    }

    for (const size of sizes.filter((s) => s >= 2)) {
      const buf = new Uint8Array(size);
      buf.fill(0xaa);

      buf[0] = 0xbb;
      buf[1] = 0xcc;
      strictEqual(searchTwo(buf, [0xbb, 0xcc]), 0);

      buf[0] = 0xaa;
      buf[1] = 0xaa;
      buf[size - 2] = 0xbb;
      buf[size - 1] = 0xcc;
      strictEqual(searchTwo(buf, [0xbb, 0xcc]), size - 2);

      if (size > 3) {
        const mid = Math.floor(size / 2);
        buf[size - 2] = 0xaa;
        buf[size - 1] = 0xaa;
        buf[mid] = 0xbb;
        buf[mid + 1] = 0xcc;
        strictEqual(searchTwo(buf, [0xbb, 0xcc]), mid);
      }
    }
  });

  test("Straddling chunk boundaries for searchTwo", () => {
    const crossPositions = [
      0x03, 0x04, 0x07, 0x08, 0x0b, 0x0c, 0x0f, 0x10, 0x13, 0x14, 0x17, 0x18,
      0x1b, 0x1c, 0x1f, 0x20, 0x27, 0x28, 0x3f, 0x40,
    ];
    const crossBuf = Buffer.alloc(0x80, "-");
    for (const pos of crossPositions) {
      crossBuf[pos] = "Y".charCodeAt(0);
      crossBuf[pos + 1] = "Z".charCodeAt(0);
      strictEqual(searchTwo(crossBuf, Buffer.from("YZ")), pos);
      crossBuf[pos] = "-".charCodeAt(0);
      crossBuf[pos + 1] = "-".charCodeAt(0);
    }
  });

  test("Large payload search (1MB payload)", () => {
    const size = 1024 * 1024;
    const buf = new Uint8Array(size);
    buf.fill(0x41);

    const targetIndices = [0, 15, 16, 31, 32, 63, 64, 5000, 50000, size - 2];

    for (const idx of targetIndices) {
      buf[idx] = 0x42;
      strictEqual(searchOne(buf, 0x42), idx);
      buf[idx + 1] = 0x43;
      strictEqual(searchTwo(buf, [0x42, 0x43]), idx);
      buf[idx] = 0x41;
      buf[idx + 1] = 0x41;
    }
  });

  test("Randomized data correctness test", () => {
    const size = 4096;
    const buf = new Uint8Array(size);

    for (let i = 0; i < size; i++) {
      buf[i] = (i * 31 + 7) % 255;
    }

    const testPositions = [
      0, 1, 15, 16, 30, 31, 32, 63, 64, 100, 511, 1023, 2047, 4094,
    ];
    for (const pos of testPositions) {
      buf[pos] = 255;
      strictEqual(searchOne(buf, 255), pos);
      buf[pos + 1] = 254;
      strictEqual(searchTwo(buf, [255, 254]), pos);
      buf[pos] = (pos * 31 + 7) % 255;
      buf[pos + 1] = ((pos + 1) * 31 + 7) % 255;
    }
  });

  test("Argument validation and type handling", () => {
    throws(() => searchOne(null, 65));
    throws(() => searchOne(undefined, 65));
    throws(() => searchOne("not_a_buffer", 65));
    throws(() => searchOne(new Uint8ClampedArray([10, 20]), 20));

    throws(() => searchTwo(null, [65, 66]));
    throws(() => searchTwo(undefined, [65, 66]));
    throws(() => searchTwo("not_a_buffer", [65, 66]));
    throws(() => searchTwo(new Uint8Array([10, 20]), null));
    throws(() => searchTwo(new Uint8Array([10, 20]), undefined));
    throws(() => searchTwo(new Uint8Array([10, 20]), []));
    throws(() => searchTwo(new Uint8Array([10, 20]), [65]));
    throws(() => searchTwo(new Uint8Array([10, 20]), [65, 66, 67]));
    throws(() => searchTwo(new Uint8Array([10, 20]), new Uint8Array([65])));
    throws(() =>
      searchTwo(new Uint8Array([10, 20]), new Uint8Array([65, 66, 67])),
    );
  });
});
