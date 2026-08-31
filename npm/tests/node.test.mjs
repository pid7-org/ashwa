import { test, describe } from "node:test";
import assert from "node:assert/strict";
import {
  searchOne,
  searchTwo,
  searchThree,
  searchN,
  isNative,
  init,
  initSync,
} from "@pid7/ashwa";

describe("Node Native Backend ESM (index.mjs)", () => {
  test("ESM exports and basic search", async () => {
    assert.strictEqual(isNative, true);
    assert.strictEqual(typeof searchOne, "function");
    assert.strictEqual(typeof searchTwo, "function");
    assert.strictEqual(typeof searchThree, "function");
    assert.strictEqual(typeof searchN, "function");

    await init();
    initSync();

    const buf = Buffer.from("ESM Node test suite");
    assert.strictEqual(searchOne(buf, "N".charCodeAt(0)), 4);
    assert.strictEqual(searchOne(buf, "Z".charCodeAt(0)), null);

    assert.strictEqual(searchTwo(buf, Buffer.from("No")), 4);
    assert.strictEqual(searchTwo(buf, [0x4e, 0x6f]), 4);
    assert.strictEqual(searchTwo(buf, Buffer.from("ZZ")), null);

    assert.strictEqual(searchThree(buf, Buffer.from("Nod")), 4);
    assert.strictEqual(searchThree(buf, [0x4e, 0x6f, 0x64]), 4);
    assert.strictEqual(searchThree(buf, Buffer.from("ZZZ")), null);

    assert.strictEqual(searchN(buf, Buffer.from("Node")), 4);
    assert.strictEqual(searchN(buf, [0x4e, 0x6f, 0x64, 0x65]), 4);
    assert.strictEqual(searchN(buf, Buffer.from("test suite")), 9);
    assert.strictEqual(searchN(buf, Buffer.from("ZZZZ")), null);
    assert.strictEqual(searchN(buf, []), 0);
  });
});
