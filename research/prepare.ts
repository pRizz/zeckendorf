#!/usr/bin/env bun
// Deterministically generate the synthetic corpus into the user's cache.
// Idempotent: existing files with the expected size are kept as-is.
//
// Output: ~/.cache/zeckendorf-research/corpus/synthetic/<shape>_<size>.bin

import { existsSync, mkdirSync, statSync, writeFileSync } from "node:fs";
import { homedir } from "node:os";
import { join } from "node:path";

const CORPUS_DIR = join(
  homedir(),
  ".cache",
  "zeckendorf-research",
  "corpus",
  "synthetic",
);

const SIZES: readonly number[] = [256, 1024, 4096];
const SEED = 0x5a5a5a5a;

function fnv1a(s: string): number {
  let h = 2166136261;
  for (let i = 0; i < s.length; i++) {
    h ^= s.charCodeAt(i);
    h = Math.imul(h, 16777619);
  }
  return h >>> 0;
}

function xorshift32(seed: number): () => number {
  let s = seed >>> 0;
  if (s === 0) s = 0xdeadbeef;
  return () => {
    s ^= s << 13;
    s >>>= 0;
    s ^= s >>> 17;
    s ^= s << 5;
    s >>>= 0;
    return s;
  };
}

const nextByte = (rng: () => number): number => rng() & 0xff;

type Shape = (size: number, rng: () => number) => Uint8Array;

const shapes: Record<string, Shape> = {
  // High-entropy bytes — incompressible baseline.
  random: (n, rng) => {
    const a = new Uint8Array(n);
    for (let i = 0; i < n; i++) a[i] = nextByte(rng);
    return a;
  },
  // Counter: every byte differs from the next by exactly 1.
  monotonic: (n) => {
    const a = new Uint8Array(n);
    for (let i = 0; i < n; i++) a[i] = i & 0xff;
    return a;
  },
  // ~5% non-zero, the rest zero — common in serialized protobuf / sparse arrays.
  sparse_zeros: (n, rng) => {
    const a = new Uint8Array(n);
    for (let i = 0; i < n; i++) {
      if (rng() % 20 === 0) a[i] = 1 + (rng() & 0x7f);
    }
    return a;
  },
  // 6-byte repeating pattern.
  repetitive: (n) => {
    const pattern = new Uint8Array([0xde, 0xad, 0xbe, 0xef, 0xca, 0xfe]);
    const a = new Uint8Array(n);
    for (let i = 0; i < n; i++) a[i] = pattern[i % pattern.length];
    return a;
  },
  // ASCII letters with ~15% spaces — text-shaped but deterministic.
  ascii_text: (n, rng) => {
    const a = new Uint8Array(n);
    for (let i = 0; i < n; i++) {
      a[i] = rng() % 100 < 15 ? 0x20 : 0x61 + (rng() % 26);
    }
    return a;
  },
  // Pseudo-JSON: many quoted short keys/values, lots of structural punctuation.
  json_ish: (n, rng) => {
    const out: number[] = [0x7b];
    let i = 0;
    while (out.length < n) {
      if (i > 0) out.push(0x2c);
      const key = `"k${i}":`;
      for (const c of key) out.push(c.charCodeAt(0));
      out.push(0x22);
      const valLen = 2 + (rng() % 4);
      for (let j = 0; j < valLen; j++) out.push(0x61 + (rng() % 26));
      out.push(0x22);
      i++;
    }
    if (out[out.length - 1] !== 0x7d) out.push(0x7d);
    return new Uint8Array(out.slice(0, n));
  },
  // Log-line shaped text: timestamp, level, structured fields.
  log_lines: (n, rng) => {
    const out: number[] = [];
    while (out.length < n) {
      const ms = (rng() % 60).toString().padStart(2, "0");
      const ss = (rng() % 60).toString().padStart(2, "0");
      const line = `2026-04-25T10:${ms}:${ss}Z INFO event=${rng() % 1000} status=${rng() % 5}\n`;
      for (const c of line) out.push(c.charCodeAt(0));
    }
    return new Uint8Array(out.slice(0, n));
  },
  // Synthetic struct: tag byte + u32 LE counter, packed every 5 bytes.
  low_entropy_struct: (n) => {
    const a = new Uint8Array(n);
    let v = 0;
    for (let i = 0; i + 4 < n; i += 5) {
      a[i] = 0xa5;
      a[i + 1] = v & 0xff;
      a[i + 2] = (v >>> 8) & 0xff;
      a[i + 3] = (v >>> 16) & 0xff;
      a[i + 4] = (v >>> 24) & 0xff;
      v += 1;
    }
    return a;
  },
};

mkdirSync(CORPUS_DIR, { recursive: true });

let written = 0;
let skipped = 0;
for (const [shapeName, shape] of Object.entries(shapes)) {
  for (const size of SIZES) {
    const rngSeed = (SEED ^ fnv1a(shapeName) ^ Math.imul(size, 2654435761)) >>> 0;
    const rng = xorshift32(rngSeed);
    const path = join(CORPUS_DIR, `${shapeName}_${size}.bin`);
    if (existsSync(path) && statSync(path).size === size) {
      skipped++;
      continue;
    }
    const data = shape(size, rng);
    writeFileSync(path, data);
    written++;
  }
}

console.log(
  `zeckendorf-research synthetic corpus: ${written} written, ${skipped} skipped`,
);
console.log(`  dir: ${CORPUS_DIR}`);
