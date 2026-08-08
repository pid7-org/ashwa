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
    const options = moduleOrPath ? { module_or_path: moduleOrPath } : undefined;
    initPromise = initWasm(options).then(() => {
      isInitialized = true;
    });
  }
  await initPromise;
}

export function initSync(bytesOrModule) {
  wasmInitSync(bytesOrModule);
  isInitialized = true;
}

export async function searchOne(haystack, needle) {
  if (!isInitialized) {
    await init();
  }
  const res = wasmSearchOne(haystack, needle);
  return res !== undefined && res !== null ? Number(res) : null;
}
