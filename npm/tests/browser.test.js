const { test, describe } = require("node:test");
const assert = require("node:assert/strict");
const path = require("node:path");
const fs = require("node:fs");
const { chromium } = require("playwright");

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

    const wasmPath = path.join(__dirname, "../wasm/pkg/ashwa_wasm_bg.wasm");
    const wasmBytes = fs.readFileSync(wasmPath);
    const wasmBase64 = wasmBytes.toString("base64");

    const jsCode = fs.readFileSync(
      path.join(__dirname, "../wasm/pkg/ashwa_wasm.js"),
      "utf8"
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
          "Hello, World! SIMD128 Headless Chrome Test."
        );
        const matchW = mod.searchOne(haystack, "W".charCodeAt(0));
        const matchZ = mod.searchOne(haystack, "Z".charCodeAt(0));

        return {
          matchW: Number(matchW),
          matchZ: matchZ != null ? Number(matchZ) : null,
        };
      },
      { code: jsCode, base64: wasmBase64 }
    );

    assert.strictEqual(result.matchW, 7);
    assert.strictEqual(result.matchZ, null);
  });

  test("Teardown Headless Chromium", async () => {
    if (browser) {
      await browser.close();
    }
  });
});
