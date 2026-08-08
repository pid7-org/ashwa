import { test, describe } from "node:test";
import assert from "node:assert/strict";
import { searchOne, isNative, init, initSync } from "../browser.mjs";

describe("WebAssembly Backend ESM (browser.mjs)", () => {
  test("ESM exports and basic search", async () => {
    assert.strictEqual(isNative, false);
    assert.strictEqual(typeof searchOne, "function");

    await init();

    const buf = Buffer.from("ESM WASM SIMD128 test suite");
    assert.strictEqual(await searchOne(buf, "W".charCodeAt(0)), 4);
    assert.strictEqual(await searchOne(buf, "Z".charCodeAt(0)), null);
  });
});
