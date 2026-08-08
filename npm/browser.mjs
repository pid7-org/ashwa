import fs from "node:fs";
import { fileURLToPath } from "node:url";
import initWasm, {
  initSync as wasmInitSync,
  searchOne as wasmSearchOne,
} from "./wasm/pkg/ashwa_wasm.js";

let isInitialized = false;
let initPromise = null;

export const isNative = false;

export async function init(moduleOrPath) {
  if (isInitialized) return;
  if (!initPromise) {
    let input = moduleOrPath;
    if (
      !input &&
      typeof process !== "undefined" &&
      process.versions != null &&
      process.versions.node != null
    ) {
      const wasmPath = fileURLToPath(
        new URL("./wasm/pkg/ashwa_wasm_bg.wasm", import.meta.url)
      );
      input = fs.readFileSync(wasmPath);
    }
    const options = input ? { module_or_path: input } : undefined;
    initPromise = initWasm(options).then(() => {
      isInitialized = true;
    });
  }
  await initPromise;
}

export function initSync(bytesOrModule) {
  let input = bytesOrModule;
  if (
    !input &&
    typeof process !== "undefined" &&
    process.versions != null &&
    process.versions.node != null
  ) {
    const wasmPath = fileURLToPath(
      new URL("./wasm/pkg/ashwa_wasm_bg.wasm", import.meta.url)
    );
    input = fs.readFileSync(wasmPath);
  }
  wasmInitSync(input);
  isInitialized = true;
}

export async function searchOne(haystack, needle) {
  if (!isInitialized) {
    await init();
  }
  const res = wasmSearchOne(haystack, needle);
  return res !== undefined && res !== null ? Number(res) : null;
}
