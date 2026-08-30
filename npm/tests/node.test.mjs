import { test, describe } from "node:test";
import assert from "node:assert/strict";
import { searchOne, searchTwo, isNative, init, initSync } from "@pid7/ashwa";

describe("Node Native Backend ESM (index.mjs)", () => {
  test("ESM exports and basic search", async () => {
    assert.strictEqual(isNative, true);
    assert.strictEqual(typeof searchOne, "function");
    assert.strictEqual(typeof searchTwo, "function");

    await init();
    initSync();

    const buf = Buffer.from("ESM Node test suite");
    assert.strictEqual(searchOne(buf, "N".charCodeAt(0)), 4);
    assert.strictEqual(searchOne(buf, "Z".charCodeAt(0)), null);

    assert.strictEqual(searchTwo(buf, Buffer.from("No")), 4);
    assert.strictEqual(searchTwo(buf, [0x4e, 0x6f]), 4);
    assert.strictEqual(searchTwo(buf, Buffer.from("ZZ")), null);
  });
});
