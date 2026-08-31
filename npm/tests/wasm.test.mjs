import { test, describe } from "node:test";
import assert from "node:assert/strict";
import { searchOne, searchTwo, searchThree, searchN, isNative, init } from "@pid7/ashwa/browser";

describe("WebAssembly Backend ESM (browser.mjs)", () => {
  test("ESM exports and basic search", async () => {
    assert.strictEqual(isNative, false);
    assert.strictEqual(typeof searchOne, "function");
    assert.strictEqual(typeof searchTwo, "function");
    assert.strictEqual(typeof searchThree, "function");
    assert.strictEqual(typeof searchN, "function");

    await init();

    const buf = Buffer.from("ESM WASM SIMD128 test suite");
    assert.strictEqual(await searchOne(buf, "W".charCodeAt(0)), 4);
    assert.strictEqual(await searchOne(buf, "Z".charCodeAt(0)), null);

    assert.strictEqual(await searchTwo(buf, Buffer.from("WA")), 4);
    assert.strictEqual(await searchTwo(buf, [0x57, 0x41]), 4);
    assert.strictEqual(await searchTwo(buf, Buffer.from("ZZ")), null);

    assert.strictEqual(await searchThree(buf, Buffer.from("WAS")), 4);
    assert.strictEqual(await searchThree(buf, [0x57, 0x41, 0x53]), 4);
    assert.strictEqual(await searchThree(buf, Buffer.from("ZZZ")), null);

    assert.strictEqual(await searchN(buf, Buffer.from("WASM")), 4);
    assert.strictEqual(await searchN(buf, [0x57, 0x41, 0x53, 0x4d]), 4);
    assert.strictEqual(await searchN(buf, Buffer.from("SIMD128")), 9);
    assert.strictEqual(await searchN(buf, Buffer.from("ZZZZ")), null);
    assert.strictEqual(await searchN(buf, []), 0);
  });
});
