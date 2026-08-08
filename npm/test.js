const assert = require("assert");
const { searchOne, isNative } = require("./index.js");

console.log(`Running test on backend: ${isNative ? "Node Native (napi-rs)" : "WebAssembly (wasm-bindgen)"}`);

const buf = Buffer.from("Hello, World! Welcome to Ashwa Rust SIMD search.");
const matchIndex = searchOne(buf, "W".charCodeAt(0));

console.log(`Searching for 'W': found at index ${matchIndex}`);
assert.strictEqual(matchIndex, 7, "Match index should be 7");

const notFound = searchOne(buf, "Z".charCodeAt(0));
console.log(`Searching for 'Z': result is ${notFound}`);
assert.strictEqual(notFound, null, "Result should be null when not found");

console.log("✅ All tests passed successfully!");
