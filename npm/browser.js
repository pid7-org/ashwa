const fs = require("fs");
const path = require("path");

let wasmModule = null;
let initPromise = null;

async function init(moduleOrPath) {
  if (wasmModule) return wasmModule;
  if (!initPromise) {
    const wasm = require("./wasm/pkg/ashwa_wasm.js");
    let input = moduleOrPath;
    if (
      !input &&
      typeof process !== "undefined" &&
      process.versions != null &&
      process.versions.node != null
    ) {
      const wasmPath = path.join(__dirname, "./wasm/pkg/ashwa_wasm_bg.wasm");
      input = fs.readFileSync(wasmPath);
    }
    const initOptions = input ? { module_or_path: input } : undefined;
    initPromise = Promise.resolve(wasm.default(initOptions)).then(() => {
      wasmModule = wasm;
      return wasm;
    });
  }
  await initPromise;
  return wasmModule;
}

function initSync(bytesOrModule) {
  const wasm = require("./wasm/pkg/ashwa_wasm.js");
  let input = bytesOrModule;
  if (
    !input &&
    typeof process !== "undefined" &&
    process.versions != null &&
    process.versions.node != null
  ) {
    const wasmPath = path.join(__dirname, "./wasm/pkg/ashwa_wasm_bg.wasm");
    input = fs.readFileSync(wasmPath);
  }
  wasm.initSync(input);
  wasmModule = wasm;
}

async function searchOne(haystack, needle) {
  if (!wasmModule) {
    await init();
  }
  const res = wasmModule.searchOne(haystack, needle);
  return res !== undefined && res !== null ? Number(res) : null;
}

module.exports = {
  isNative: false,
  init,
  initSync,
  searchOne,
};
