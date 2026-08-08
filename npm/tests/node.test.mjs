import { test, describe } from "node:test";
import assert from "node:assert/strict";
import { searchOne, isNative, init, initSync } from "../index.mjs";

describe("Node Native Backend ESM (index.mjs)", () => {
  test("ESM exports and basic search", async () => {
    assert.strictEqual(isNative, true);
    assert.strictEqual(typeof searchOne, "function");

    await init();
    initSync();

    const buf = Buffer.from("ESM Node test suite");
    assert.strictEqual(searchOne(buf, "N".charCodeAt(0)), 4);
    assert.strictEqual(searchOne(buf, "Z".charCodeAt(0)), null);
  });
});
