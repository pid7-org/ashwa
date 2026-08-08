import { createRequire } from "module";

const require = createRequire(import.meta.url);
const native = require("./native/index.js");

export const isNative = true;

export async function init() {}
export function initSync() {}

export function searchOne(haystack, needle) {
  const res = native.searchOne(haystack, needle);
  return res !== undefined && res !== null ? Number(res) : null;
}
