import { test, describe } from "node:test";
import assert from "node:assert/strict";
import path from "node:path";
import fs from "node:fs";
import { createRequire } from "node:module";
import { chromium } from "playwright";

const require = createRequire(import.meta.url);

describe("Headless Browser WASM SIMD128 Execution", () => {
  let browser;
  let page;

  test("Initialize Headless Chromium", async () => {
    try {
      browser = await chromium.launch({
        headless: true,
        args: ["--no-sandbox", "--disable-setuid-sandbox"],
      });
      page = await browser.newPage();
    } catch (e) {
      // NOTE: Skip if system shared library is missing on local environment
    }
  });

  test("Execute WASM SIMD128 in Headless Browser context", async () => {
    if (!page) {
      return;
    }

    const packageDir = path.dirname(require.resolve("@pid7/ashwa/package.json"));
    const wasmPath = path.join(packageDir, "wasm/pkg/ashwa_wasm_bg.wasm");
    const wasmBytes = fs.readFileSync(wasmPath);
    const wasmBase64 = wasmBytes.toString("base64");

    const jsCode = fs.readFileSync(
      path.join(packageDir, "wasm/pkg/ashwa_wasm.js"),
      "utf8",
    );

    // NOTE: Injecting WASM module into real Browser V8 engine context
    const result = await page.evaluate(
      async ({ code, base64 }) => {
        const binaryString = atob(base64);
        const len = binaryString.length;
        const bytes = new Uint8Array(len);
        for (let i = 0; i < len; i++) {
          bytes[i] = binaryString.charCodeAt(i);
        }

        const blob = new Blob([code], { type: "application/javascript" });
        const url = URL.createObjectURL(blob);
        const mod = await import(url);

        await mod.default({ module_or_path: bytes.buffer });

        const haystack = new TextEncoder().encode(
          "Hello, World! SIMD128 Headless Chrome Test.",
        );
        const matchW = mod.searchOne(haystack, "W".charCodeAt(0));
        const matchZ = mod.searchOne(haystack, "Z".charCodeAt(0));

        const needleWo = new TextEncoder().encode("Wo");
        const needleZZ = new TextEncoder().encode("ZZ");
        const matchWo = mod.searchTwo(haystack, needleWo);
        const matchZZ = mod.searchTwo(haystack, needleZZ);

        const needleWor = new TextEncoder().encode("Wor");
        const needleZZZ = new TextEncoder().encode("ZZZ");
        const matchWor = mod.searchThree(haystack, needleWor);
        const matchZZZ = mod.searchThree(haystack, needleZZZ);

        const needleWorld = new TextEncoder().encode("World");
        const needleNotFound = new TextEncoder().encode("Not Found");
        const matchWorld = mod.searchN(haystack, needleWorld);
        const matchNotFound = mod.searchN(haystack, needleNotFound);

        return {
          matchW: Number(matchW),
          matchZ: matchZ != null ? Number(matchZ) : null,
          matchWo: matchWo != null ? Number(matchWo) : null,
          matchZZ: matchZZ != null ? Number(matchZZ) : null,
          matchWor: matchWor != null ? Number(matchWor) : null,
          matchZZZ: matchZZZ != null ? Number(matchZZZ) : null,
          matchWorld: matchWorld != null ? Number(matchWorld) : null,
          matchNotFound: matchNotFound != null ? Number(matchNotFound) : null,
        };
      },
      { code: jsCode, base64: wasmBase64 },
    );

    assert.strictEqual(result.matchW, 7);
    assert.strictEqual(result.matchZ, null);
    assert.strictEqual(result.matchWo, 7);
    assert.strictEqual(result.matchZZ, null);
    assert.strictEqual(result.matchWor, 7);
    assert.strictEqual(result.matchZZZ, null);
    assert.strictEqual(result.matchWorld, 7);
    assert.strictEqual(result.matchNotFound, null);
  });

  test("Teardown Headless Chromium", async () => {
    if (browser) {
      await browser.close();
    }
  });
});
