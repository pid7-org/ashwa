import { test, describe } from "node:test";
import assert from "node:assert/strict";
import { searchOne, searchTwo, isNative, init } from "../browser.mjs";

describe("WebAssembly Backend ESM (browser.mjs)", () => {
  test("ESM exports and basic search", async () => {
    assert.strictEqual(isNative, false);
    assert.strictEqual(typeof searchOne, "function");
    assert.strictEqual(typeof searchTwo, "function");

    await init();

    const buf = Buffer.from("ESM WASM SIMD128 test suite");
    assert.strictEqual(await searchOne(buf, "W".charCodeAt(0)), 4);
    assert.strictEqual(await searchOne(buf, "Z".charCodeAt(0)), null);

    assert.strictEqual(await searchTwo(buf, Buffer.from("WA")), 4);
    assert.strictEqual(await searchTwo(buf, [0x57, 0x41]), 4);
    assert.strictEqual(await searchTwo(buf, Buffer.from("ZZ")), null);
  });
});
