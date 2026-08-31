const { test, describe } = require("node:test");
const { strictEqual, rejects } = require("node:assert/strict");
const { searchOne, searchTwo, searchThree, searchN, isNative, init, initSync } = require("@pid7/ashwa/browser");

describe("WebAssembly Backend (wasm-bindgen SIMD128)", () => {
  test("Environment & exports verification", async () => {
    strictEqual(isNative, false);
    strictEqual(typeof searchOne, "function");
    strictEqual(typeof searchTwo, "function");
    strictEqual(typeof searchThree, "function");
    strictEqual(typeof searchN, "function");
    strictEqual(typeof init, "function");
    strictEqual(typeof initSync, "function");

    await init();
  });

  test("Basic searchOne operations", async () => {
    const haystack = Buffer.from("abcdefghijklmnopqrstuvwxyz");

    strictEqual(await searchOne(haystack, "a".charCodeAt(0)), 0);
    strictEqual(await searchOne(haystack, "m".charCodeAt(0)), 12);
    strictEqual(await searchOne(haystack, "z".charCodeAt(0)), 25);
    strictEqual(await searchOne(haystack, "A".charCodeAt(0)), null);
  });

  test("Basic searchTwo operations", async () => {
    const haystack = Buffer.from("abcdefghijklmnopqrstuvwxyz");

    strictEqual(await searchTwo(haystack, Buffer.from("ab")), 0);
    strictEqual(await searchTwo(haystack, Buffer.from("mn")), 12);
    strictEqual(await searchTwo(haystack, Buffer.from("yz")), 24);
    strictEqual(await searchTwo(haystack, Buffer.from("AB")), null);
    strictEqual(await searchTwo(haystack, [0x61, 0x62]), 0);
  });

  test("Basic searchThree operations", async () => {
    const haystack = Buffer.from("abcdefghijklmnopqrstuvwxyz");

    strictEqual(await searchThree(haystack, Buffer.from("abc")), 0);
    strictEqual(await searchThree(haystack, Buffer.from("mno")), 12);
    strictEqual(await searchThree(haystack, Buffer.from("xyz")), 23);
    strictEqual(await searchThree(haystack, Buffer.from("ABC")), null);
    strictEqual(await searchThree(haystack, [0x61, 0x62, 0x63]), 0);
  });

  test("Basic searchN operations", async () => {
    const haystack = Buffer.from("abcdefghijklmnopqrstuvwxyz");

    strictEqual(await searchN(haystack, Buffer.from("a")), 0);
    strictEqual(await searchN(haystack, Buffer.from("ab")), 0);
    strictEqual(await searchN(haystack, Buffer.from("abc")), 0);
    strictEqual(await searchN(haystack, Buffer.from("cde")), 2);
    strictEqual(await searchN(haystack, Buffer.from("mnop")), 12);
    strictEqual(await searchN(haystack, Buffer.from("vwxyz")), 21);
    strictEqual(await searchN(haystack, Buffer.from("abcdefghijklmnopqrstuvwxyz")), 0);
    strictEqual(await searchN(haystack, Buffer.from("abcdefghijklmnopqrstuvwxyz!")), null);
    strictEqual(await searchN(haystack, Buffer.from("XYZ")), null);
    strictEqual(await searchN(haystack, [0x61, 0x62, 0x63, 0x64]), 0);
    strictEqual(await searchN(haystack, []), 0);
    strictEqual(await searchN(haystack, new Uint8Array(0)), 0);
  });

  test("Pangram spot-checks for searchTwo, searchThree, and searchN", async () => {
    const haystack = Buffer.from("the quick brown fox jumps over the lazy dog");

    strictEqual(await searchTwo(haystack, Buffer.from("th")), 0);
    strictEqual(await searchTwo(haystack, Buffer.from("he")), 1);
    strictEqual(await searchTwo(haystack, Buffer.from("qu")), 4);
    strictEqual(await searchTwo(haystack, Buffer.from("ox")), 17);
    strictEqual(await searchTwo(haystack, Buffer.from("do")), 40);
    strictEqual(await searchTwo(haystack, Buffer.from("og")), 41);
    strictEqual(await searchTwo(haystack, Buffer.from("ZZ")), null);
    strictEqual(await searchTwo(haystack, Buffer.from("!!")), null);

    strictEqual(await searchThree(haystack, Buffer.from("the")), 0);
    strictEqual(await searchThree(haystack, Buffer.from("qui")), 4);
    strictEqual(await searchThree(haystack, Buffer.from("fox")), 16);
    strictEqual(await searchThree(haystack, Buffer.from("dog")), 40);
    strictEqual(await searchThree(haystack, Buffer.from("ZZZ")), null);
    strictEqual(await searchThree(haystack, Buffer.from("!!!")), null);

    strictEqual(await searchN(haystack, Buffer.from("quick")), 4);
    strictEqual(await searchN(haystack, Buffer.from("brown fox")), 10);
    strictEqual(await searchN(haystack, Buffer.from("lazy dog")), 35);
    strictEqual(await searchN(haystack, Buffer.from("the quick brown fox jumps over the lazy dog")), 0);
    strictEqual(await searchN(haystack, Buffer.from("lazy dog!")), null);
    strictEqual(await searchN(haystack, Buffer.from("!!!")), null);
  });

  test("Empty buffer search", async () => {
    const emptyUint8 = new Uint8Array(0);
    const emptyBuf = Buffer.alloc(0);

    strictEqual(await searchOne(emptyUint8, 65), null);
    strictEqual(await searchOne(emptyBuf, 65), null);

    strictEqual(await searchTwo(emptyUint8, Buffer.from("ab")), null);
    strictEqual(await searchTwo(emptyBuf, Buffer.from("ab")), null);

    const singleByte = Buffer.from("a");
    strictEqual(await searchTwo(singleByte, Buffer.from("ab")), null);
    strictEqual(await searchThree(emptyUint8, Buffer.from("abc")), null);
    strictEqual(await searchThree(emptyBuf, Buffer.from("abc")), null);
    strictEqual(await searchThree(singleByte, Buffer.from("abc")), null);
    strictEqual(await searchThree(Buffer.from("ab"), Buffer.from("abc")), null);

    strictEqual(await searchN(emptyUint8, Buffer.from("abcd")), null);
    strictEqual(await searchN(emptyBuf, Buffer.from("abcd")), null);
    strictEqual(await searchN(singleByte, Buffer.from("abcd")), null);
    strictEqual(await searchN(emptyUint8, Buffer.alloc(0)), 0);
    strictEqual(await searchN(emptyBuf, Buffer.alloc(0)), 0);
  });

  test("Multiple occurrences return first match", async () => {
    const haystack = Buffer.from("banana");
    strictEqual(await searchOne(haystack, "a".charCodeAt(0)), 1);
    strictEqual(await searchOne(haystack, "n".charCodeAt(0)), 2);

    strictEqual(await searchTwo(haystack, Buffer.from("an")), 1);
    strictEqual(await searchTwo(haystack, Buffer.from("na")), 2);

    strictEqual(await searchThree(haystack, Buffer.from("ana")), 1);
    strictEqual(await searchThree(haystack, Buffer.from("nan")), 2);

    strictEqual(await searchN(haystack, Buffer.from("an")), 1);
    strictEqual(await searchN(haystack, Buffer.from("ana")), 1);
    strictEqual(await searchN(haystack, Buffer.from("anan")), 1);
    strictEqual(await searchN(haystack, Buffer.from("nana")), 2);
  });

  test("Binary data & extreme byte values (0x00 and 0xFF)", async () => {
    const data = new Uint8Array([0x10, 0x20, 0x00, 0x30, 0xff, 0x40]);

    strictEqual(await searchOne(data, 0x00), 2);
    strictEqual(await searchOne(data, 0xff), 4);
    strictEqual(await searchOne(data, 0x99), null);

    const binaryData = new Uint8Array([
      0x10, 0x00, 0x00, 0x30, 0xfe, 0xff, 0x40,
    ]);
    strictEqual(await searchTwo(binaryData, new Uint8Array([0x00, 0x00])), 1);
    strictEqual(await searchTwo(binaryData, new Uint8Array([0xfe, 0xff])), 4);
    strictEqual(
      await searchTwo(binaryData, new Uint8Array([0xff, 0xff])),
      null,
    );

    const binaryData3 = new Uint8Array([
      0x10, 0x00, 0x00, 0x00, 0x30, 0xfd, 0xfe, 0xff, 0x40,
    ]);
    strictEqual(await searchThree(binaryData3, new Uint8Array([0x00, 0x00, 0x00])), 1);
    strictEqual(await searchThree(binaryData3, new Uint8Array([0xfd, 0xfe, 0xff])), 5);
    strictEqual(
      await searchThree(binaryData3, new Uint8Array([0xff, 0xff, 0xff])),
      null,
    );

    const binaryDataN = new Uint8Array([
      0x10, 0x00, 0x00, 0x00, 0x00, 0x30, 0xfc, 0xfd, 0xfe, 0xff, 0x40,
    ]);
    strictEqual(await searchN(binaryDataN, new Uint8Array([0x00, 0x00, 0x00, 0x00])), 1);
    strictEqual(await searchN(binaryDataN, new Uint8Array([0xfc, 0xfd, 0xfe, 0xff])), 6);
    strictEqual(
      await searchN(binaryDataN, new Uint8Array([0xff, 0xff, 0xff, 0xff])),
      null,
    );
  });

  test("Overlapping & repeating patterns", async () => {
    strictEqual(await searchTwo(Buffer.from("aaaaaa"), Buffer.from("aa")), 0);
    strictEqual(await searchTwo(Buffer.from("baaaaa"), Buffer.from("aa")), 1);
    strictEqual(await searchTwo(Buffer.from("bbaaaa"), Buffer.from("aa")), 2);
    strictEqual(await searchTwo(Buffer.from("ababab"), Buffer.from("ab")), 0);
    strictEqual(await searchTwo(Buffer.from("bababa"), Buffer.from("ab")), 1);

    strictEqual(await searchThree(Buffer.from("aaaaaa"), Buffer.from("aaa")), 0);
    strictEqual(await searchThree(Buffer.from("baaaaa"), Buffer.from("aaa")), 1);
    strictEqual(await searchThree(Buffer.from("bbaaaa"), Buffer.from("aaa")), 2);
    strictEqual(await searchThree(Buffer.from("abcabc"), Buffer.from("abc")), 0);
    strictEqual(await searchThree(Buffer.from("zabcabc"), Buffer.from("abc")), 1);

    strictEqual(await searchN(Buffer.from("aaaaaaaa"), Buffer.from("aaaa")), 0);
    strictEqual(await searchN(Buffer.from("baaaaaaa"), Buffer.from("aaaa")), 1);
    strictEqual(await searchN(Buffer.from("bbaaaaaa"), Buffer.from("aaaa")), 2);
    strictEqual(await searchN(Buffer.from("abcdabcd"), Buffer.from("abcd")), 0);
    strictEqual(await searchN(Buffer.from("zabcdabcd"), Buffer.from("abcd")), 1);

    const allA = Buffer.alloc(256, "A");
    strictEqual(await searchTwo(allA, Buffer.from("AB")), null);
    allA[120] = "B".charCodeAt(0);
    strictEqual(await searchTwo(allA, Buffer.from("AB")), 119);

    const allA3 = Buffer.alloc(256, "A");
    allA3[120] = "B".charCodeAt(0);
    allA3[121] = "C".charCodeAt(0);
    strictEqual(await searchThree(allA3, Buffer.from("ABC")), 119);

    const allAN = Buffer.alloc(256, "A");
    allAN[120] = "B".charCodeAt(0);
    allAN[121] = "C".charCodeAt(0);
    allAN[122] = "D".charCodeAt(0);
    allAN[123] = "E".charCodeAt(0);
    strictEqual(await searchN(allAN, Buffer.from("ABCDE")), 119);
  });

  test("Buffer types compatibility (Buffer vs Uint8Array vs Subarray)", async () => {
    const rawArray = [10, 20, 30, 40, 50, 60, 70, 80];

    const uint8 = new Uint8Array(rawArray);
    const nodeBuf = Buffer.from(rawArray);
    const subarray = uint8.subarray(2, 7);

    strictEqual(await searchOne(uint8, 50), 4);
    strictEqual(await searchOne(nodeBuf, 50), 4);
    strictEqual(await searchOne(subarray, 50), 2);

    strictEqual(await searchTwo(uint8, new Uint8Array([50, 60])), 4);
    strictEqual(await searchTwo(nodeBuf, Buffer.from([50, 60])), 4);
    strictEqual(await searchTwo(subarray, [50, 60]), 2);

    strictEqual(await searchThree(uint8, new Uint8Array([50, 60, 70])), 4);
    strictEqual(await searchThree(nodeBuf, Buffer.from([50, 60, 70])), 4);
    strictEqual(await searchThree(subarray, [50, 60, 70]), 2);

    strictEqual(await searchN(uint8, new Uint8Array([50, 60, 70, 80])), 4);
    strictEqual(await searchN(nodeBuf, Buffer.from([50, 60, 70, 80])), 4);
    strictEqual(await searchN(subarray, [50, 60, 70]), 2);
  });

  test("Unaligned byteOffset subarray searches", async () => {
    const memory = new ArrayBuffer(128);
    const view = new Uint8Array(memory, 7, 50);
    view.fill(0x55);
    view[19] = 0xaa;
    view[20] = 0xbb;
    view[21] = 0xcc;
    view[22] = 0xdd;

    // NOTE: Unaligned buffer views must compute relative index correctly
    strictEqual(await searchOne(view, 0xaa), 19);
    strictEqual(await searchTwo(view, new Uint8Array([0xaa, 0xbb])), 19);
    strictEqual(await searchThree(view, new Uint8Array([0xaa, 0xbb, 0xcc])), 19);
    strictEqual(await searchN(view, new Uint8Array([0xaa, 0xbb, 0xcc, 0xdd])), 19);
  });

  test("WASM SIMD128 boundary & chunk sizes", async () => {
    const sizes = [
      1, 2, 3, 7, 8, 9, 15, 16, 17, 24, 25, 31, 32, 33, 34, 63, 64, 65, 127,
      128, 255, 256,
    ];

    for (const size of sizes) {
      const buf = new Uint8Array(size);
      buf.fill(0xaa);

      buf[0] = 0xbb;
      strictEqual(await searchOne(buf, 0xbb), 0);

      buf[0] = 0xaa;
      buf[size - 1] = 0xbb;
      strictEqual(await searchOne(buf, 0xbb), size - 1);

      if (size > 2) {
        const mid = Math.floor(size / 2);
        buf[size - 1] = 0xaa;
        buf[mid] = 0xbb;
        strictEqual(await searchOne(buf, 0xbb), mid);
      }
    }

    for (const size of sizes.filter((s) => s >= 2)) {
      const buf = new Uint8Array(size);
      buf.fill(0xaa);

      buf[0] = 0xbb;
      buf[1] = 0xcc;
      strictEqual(await searchTwo(buf, [0xbb, 0xcc]), 0);

      buf[0] = 0xaa;
      buf[1] = 0xaa;
      buf[size - 2] = 0xbb;
      buf[size - 1] = 0xcc;
      strictEqual(await searchTwo(buf, [0xbb, 0xcc]), size - 2);

      if (size > 3) {
        const mid = Math.floor(size / 2);
        buf[size - 2] = 0xaa;
        buf[size - 1] = 0xaa;
        buf[mid] = 0xbb;
        buf[mid + 1] = 0xcc;
        strictEqual(await searchTwo(buf, [0xbb, 0xcc]), mid);
      }
    }

    for (const size of sizes.filter((s) => s >= 3)) {
      const buf = new Uint8Array(size);
      buf.fill(0xaa);

      buf[0] = 0xbb;
      buf[1] = 0xcc;
      buf[2] = 0xdd;
      strictEqual(await searchThree(buf, [0xbb, 0xcc, 0xdd]), 0);

      buf[0] = 0xaa;
      buf[1] = 0xaa;
      buf[2] = 0xaa;
      buf[size - 3] = 0xbb;
      buf[size - 2] = 0xcc;
      buf[size - 1] = 0xdd;
      strictEqual(await searchThree(buf, [0xbb, 0xcc, 0xdd]), size - 3);

      if (size > 4) {
        const mid = Math.floor(size / 2);
        buf[size - 3] = 0xaa;
        buf[size - 2] = 0xaa;
        buf[size - 1] = 0xaa;
        buf[mid] = 0xbb;
        buf[mid + 1] = 0xcc;
        buf[mid + 2] = 0xdd;
        strictEqual(await searchThree(buf, [0xbb, 0xcc, 0xdd]), mid);
      }
    }

    for (const size of sizes.filter((s) => s >= 5)) {
      const buf = new Uint8Array(size);
      buf.fill(0xaa);

      buf[0] = 0x11;
      buf[1] = 0x22;
      buf[2] = 0x33;
      buf[3] = 0x44;
      buf[4] = 0x55;
      strictEqual(await searchN(buf, [0x11, 0x22, 0x33, 0x44, 0x55]), 0);

      buf[0] = 0xaa;
      buf[1] = 0xaa;
      buf[2] = 0xaa;
      buf[3] = 0xaa;
      buf[4] = 0xaa;
      buf[size - 5] = 0x11;
      buf[size - 4] = 0x22;
      buf[size - 3] = 0x33;
      buf[size - 2] = 0x44;
      buf[size - 1] = 0x55;
      strictEqual(await searchN(buf, [0x11, 0x22, 0x33, 0x44, 0x55]), size - 5);
    }
  });

  test("Straddling chunk boundaries for searchTwo, searchThree, and searchN", async () => {
    const crossPositions = [
      0x03, 0x04, 0x07, 0x08, 0x0b, 0x0c, 0x0f, 0x10, 0x13, 0x14, 0x17, 0x18,
      0x1b, 0x1c, 0x1f, 0x20, 0x27, 0x28, 0x3f, 0x40,
    ];
    const crossBuf = Buffer.alloc(0x80, "-");
    for (const pos of crossPositions) {
      crossBuf[pos] = "Y".charCodeAt(0);
      crossBuf[pos + 1] = "Z".charCodeAt(0);
      strictEqual(await searchTwo(crossBuf, Buffer.from("YZ")), pos);
      crossBuf[pos] = "-".charCodeAt(0);
      crossBuf[pos + 1] = "-".charCodeAt(0);
    }

    const crossBuf3 = Buffer.alloc(0x80, "-");
    for (const pos of crossPositions) {
      crossBuf3[pos] = "X".charCodeAt(0);
      crossBuf3[pos + 1] = "Y".charCodeAt(0);
      crossBuf3[pos + 2] = "Z".charCodeAt(0);
      strictEqual(await searchThree(crossBuf3, Buffer.from("XYZ")), pos);
      crossBuf3[pos] = "-".charCodeAt(0);
      crossBuf3[pos + 1] = "-".charCodeAt(0);
      crossBuf3[pos + 2] = "-".charCodeAt(0);
    }

    const crossBufN = Buffer.alloc(0x80, "-");
    for (const pos of crossPositions) {
      crossBufN[pos] = "W".charCodeAt(0);
      crossBufN[pos + 1] = "X".charCodeAt(0);
      crossBufN[pos + 2] = "Y".charCodeAt(0);
      crossBufN[pos + 3] = "Z".charCodeAt(0);
      strictEqual(await searchN(crossBufN, Buffer.from("WXYZ")), pos);
      crossBufN[pos] = "-".charCodeAt(0);
      crossBufN[pos + 1] = "-".charCodeAt(0);
      crossBufN[pos + 2] = "-".charCodeAt(0);
      crossBufN[pos + 3] = "-".charCodeAt(0);
    }
  });

  test("Large payload search (1MB payload)", async () => {
    const size = 1024 * 1024;
    const buf = new Uint8Array(size);
    buf.fill(0x41);

    const targetIndices = [0, 15, 16, 31, 32, 63, 64, 5000, 50000, size - 5];

    for (const idx of targetIndices) {
      buf[idx] = 0x42;
      strictEqual(await searchOne(buf, 0x42), idx);
      buf[idx + 1] = 0x43;
      strictEqual(await searchTwo(buf, [0x42, 0x43]), idx);
      buf[idx + 2] = 0x44;
      strictEqual(await searchThree(buf, [0x42, 0x43, 0x44]), idx);
      buf[idx + 3] = 0x45;
      buf[idx + 4] = 0x46;
      strictEqual(await searchN(buf, [0x42, 0x43, 0x44, 0x45, 0x46]), idx);
      buf[idx] = 0x41;
      buf[idx + 1] = 0x41;
      buf[idx + 2] = 0x41;
      buf[idx + 3] = 0x41;
      buf[idx + 4] = 0x41;
    }
  });

  test("Randomized data correctness test", async () => {
    const size = 4096;
    const buf = new Uint8Array(size);

    for (let i = 0; i < size; i++) {
      buf[i] = (i * 31 + 7) % 255;
    }

    const testPositions = [
      0, 1, 15, 16, 30, 31, 32, 63, 64, 100, 511, 1023, 2047, 4090,
    ];
    for (const pos of testPositions) {
      buf[pos] = 255;
      strictEqual(await searchOne(buf, 255), pos);
      buf[pos + 1] = 254;
      strictEqual(await searchTwo(buf, [255, 254]), pos);
      buf[pos + 2] = 253;
      strictEqual(await searchThree(buf, [255, 254, 253]), pos);
      buf[pos + 3] = 252;
      strictEqual(await searchN(buf, [255, 254, 253, 252]), pos);
      buf[pos] = (pos * 31 + 7) % 255;
      buf[pos + 1] = ((pos + 1) * 31 + 7) % 255;
      buf[pos + 2] = ((pos + 2) * 31 + 7) % 255;
      buf[pos + 3] = ((pos + 3) * 31 + 7) % 255;
    }
  });

  test("Argument validation and type handling", async () => {
    await rejects(async () => searchTwo(null, [65, 66]));
    await rejects(async () => searchTwo(undefined, [65, 66]));
    await rejects(async () => searchTwo(new Uint8Array([10, 20]), null));
    await rejects(async () => searchTwo(new Uint8Array([10, 20]), undefined));
    await rejects(async () => searchTwo(new Uint8Array([10, 20]), []));
    await rejects(async () => searchTwo(new Uint8Array([10, 20]), [65]));
    await rejects(async () =>
      searchTwo(new Uint8Array([10, 20]), [65, 66, 67]),
    );
    await rejects(async () =>
      searchTwo(new Uint8Array([10, 20]), new Uint8Array([65])),
    );
    await rejects(async () =>
      searchTwo(new Uint8Array([10, 20]), new Uint8Array([65, 66, 67])),
    );

    await rejects(async () => searchThree(null, [65, 66, 67]));
    await rejects(async () => searchThree(undefined, [65, 66, 67]));
    await rejects(async () => searchThree(new Uint8Array([10, 20]), null));
    await rejects(async () => searchThree(new Uint8Array([10, 20]), undefined));
    await rejects(async () => searchThree(new Uint8Array([10, 20]), []));
    await rejects(async () => searchThree(new Uint8Array([10, 20]), [65]));
    await rejects(async () => searchThree(new Uint8Array([10, 20]), [65, 66]));
    await rejects(async () =>
      searchThree(new Uint8Array([10, 20]), [65, 66, 67, 68]),
    );
    await rejects(async () =>
      searchThree(new Uint8Array([10, 20]), new Uint8Array([65, 66])),
    );
    await rejects(async () =>
      searchThree(new Uint8Array([10, 20]), new Uint8Array([65, 66, 67, 68])),
    );

    await rejects(async () => searchN(null, [65, 66, 67, 68]));
    await rejects(async () => searchN(undefined, [65, 66, 67, 68]));
    await rejects(async () => searchN(new Uint8Array([10, 20]), null));
    await rejects(async () => searchN(new Uint8Array([10, 20]), undefined));
    await rejects(async () => searchN(new Uint8Array([10, 20]), "not_a_needle"));
  });
});
