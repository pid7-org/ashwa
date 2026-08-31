/**
 * Ashwa searchN Throughput & Latency Microbenchmark Suite (WebAssembly SIMD128)
 */

import { initSync, searchN } from "@pid7/ashwa/browser";

initSync();

const KB = 0x400;
const MB = KB * KB;
const GB = 0x400 * MB;
const SAMPLES = 0x200;

export const TIERS = [
  { name: "L1", size: 0x20 * KB },
  { name: "L2", size: 0x200 * KB },
  { name: "L3", size: 0x10 * MB },
  { name: "RAM", size: 0x100 * MB },
  { name: "RAM", size: 0x200 * MB },
  { name: "RAM", size: 1 * GB },
];

// Anti-DCE (Dead Code Elimination) optimization barrier sink
let blackHole = null;

export function formatSize(bytes) {
  if (bytes >= GB) {
    return `${Math.floor(bytes / GB)} GiB`;
  } else if (bytes >= MB) {
    return `${Math.floor(bytes / MB)} MiB`;
  } else if (bytes >= KB) {
    return `${Math.floor(bytes / KB)} KiB`;
  }

  return `${bytes} B`;
}

export function formatLatency(secs) {
  const nanos = secs * 1e9;

  if (nanos < 1_000.0) {
    return `${nanos.toFixed(2)} ns`;
  } else if (nanos < 1_000_000.0) {
    return `${(nanos / 1_000.0).toFixed(2)} µs`;
  } else if (nanos < 1_000_000_000.0) {
    return `${(nanos / 1_000_000.0).toFixed(2)} ms`;
  }

  return `${secs.toFixed(2)} s`;
}

export async function benchmarkTier(tier, haystack, needle) {
  const size = tier.size;
  const slice = haystack.subarray(0, size);

  // Warmup
  const warmupStart = process.hrtime.bigint();
  let warmupIters = 0;

  while (
    (Number(process.hrtime.bigint() - warmupStart) < 0x5f5e100 && warmupIters < 0x40) ||
    warmupIters < 2
  ) {
    blackHole = await searchN(slice, needle);
    warmupIters++;
  }

  const probeStart = process.hrtime.bigint();
  const probeIters = Math.max(0x0a, Math.floor(warmupIters / 0x0a));

  for (let i = 0; i < probeIters; i++) {
    blackHole = await searchN(slice, needle);
  }

  const probeElapsedSecs = Number(process.hrtime.bigint() - probeStart) / 1e9;
  const timePerSingleIter = Math.max(probeElapsedSecs / probeIters, 1e-9);

  const batchSize = Math.max(1, Math.round(0.001 / timePerSingleIter));
  const sampleDurations = new Array(SAMPLES);

  for (let s = 0; s < SAMPLES; s++) {
    const sampleStart = process.hrtime.bigint();

    for (let b = 0; b < batchSize; b++) {
      blackHole = await searchN(slice, needle);
    }

    const elapsedSecs = Number(process.hrtime.bigint() - sampleStart) / 1e9;
    sampleDurations[s] = elapsedSecs / batchSize;
  }

  sampleDurations.sort((a, b) => a - b);

  const medianSecs = sampleDurations[Math.floor(sampleDurations.length / 2)];
  const gibPerSec = size / (0x400 * 0x400 * 0x400) / medianSecs;

  return {
    name: tier.name,
    size,
    latencySecs: medianSecs,
    throughputGiB: gibPerSec,
  };
}

export function printTable(results) {
  const colTier = "Tier / Level";
  const colSize = "Size";
  const colLat = "Latency (Median)";
  const colThrpt = "Throughput";

  const wTier = 0x16;
  const wSize = 0x0a;
  const wLat = 0x12;
  const wThrpt = 0x10;

  const divider = `+-${"-".repeat(wTier)}-+-${"-".repeat(wSize)}-+-${"-".repeat(wLat)}-+-${"-".repeat(wThrpt)}-+`;

  console.log(divider);
  console.log(
    `| ${colTier.padEnd(wTier)} | ${colSize.padStart(wSize)} | ${colLat.padStart(wLat)} | ${colThrpt.padStart(wThrpt)} |`,
  );
  console.log(divider);

  for (const r of results) {
    console.log(
      `| ${r.name.padEnd(wTier)} | ${formatSize(r.size).padStart(wSize)} | ${formatLatency(r.latencySecs).padStart(wLat)} | ${(r.throughputGiB.toFixed(2) + " GiB/s").padStart(wThrpt)} |`,
    );
  }

  console.log(divider);
}

export async function main() {
  const needle = new Uint8Array([0x0a, 0x0b, 0x0c, 0x0d, 0x0e]);
  const maxSize = Math.max(...TIERS.map((t) => t.size));

  const haystack = new Uint8Array(maxSize);

  for (let pageOffset = 0; pageOffset < maxSize; pageOffset += 0x1000) {
    haystack[pageOffset] = 0;
  }

  const results = [];
  for (const tier of TIERS) {
    results.push(await benchmarkTier(tier, haystack, needle));
  }

  printTable(results);
}

main();
