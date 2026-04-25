#!/usr/bin/env bun

import { existsSync, mkdirSync, readFileSync, appendFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { execFileSync } from "node:child_process";

type Args = {
  runLog: string;
  description: string;
  prevRatio: number;
};

type ParsedMetrics = {
  ratio: number;
  roundtripOk: boolean;
  compressedBytes: number;
  wallSeconds: number;
};

const RESULTS_PATH = join("research", "results.tsv");

function usage(): never {
  console.error(
    "usage: bun run research/scripts/log_result.ts --run-log <path> --description <str> --prev-ratio <float>",
  );
  process.exit(2);
}

function parseArgs(argv: string[]): Args {
  const values = new Map<string, string>();
  for (let i = 0; i < argv.length; i++) {
    const key = argv[i];
    if (!key.startsWith("--")) usage();
    const value = argv[i + 1];
    if (value === undefined || value.startsWith("--")) usage();
    values.set(key, value);
    i++;
  }

  const runLog = values.get("--run-log");
  const description = values.get("--description");
  const prevRatioRaw = values.get("--prev-ratio");
  if (runLog === undefined || description === undefined || prevRatioRaw === undefined) {
    usage();
  }

  const prevRatio = Number.parseFloat(prevRatioRaw);
  if (Number.isNaN(prevRatio)) {
    console.error(`invalid --prev-ratio: ${prevRatioRaw}`);
    process.exit(2);
  }

  return { runLog, description, prevRatio };
}

function parseMetrics(logText: string): ParsedMetrics | null {
  const metrics = new Map<string, string>();
  for (const line of logText.split(/\r?\n/)) {
    const match = line.match(/^([a-z_]+):\s+(.+)$/);
    if (match === null) continue;
    metrics.set(match[1], match[2].trim());
  }

  const ratioRaw = metrics.get("ratio");
  const roundtripRaw = metrics.get("roundtrip_ok");
  const compressedRaw = metrics.get("compressed_bytes");
  const wallRaw = metrics.get("wall_seconds");
  if (
    ratioRaw === undefined ||
    roundtripRaw === undefined ||
    compressedRaw === undefined ||
    wallRaw === undefined
  ) {
    return null;
  }

  const ratio = Number.parseFloat(ratioRaw);
  const compressedBytes = Number.parseInt(compressedRaw, 10);
  const wallSeconds = Number.parseFloat(wallRaw);
  if (
    !Number.isFinite(ratio) ||
    !Number.isFinite(compressedBytes) ||
    !Number.isFinite(wallSeconds)
  ) {
    return null;
  }

  return {
    ratio,
    roundtripOk: roundtripRaw === "true",
    compressedBytes,
    wallSeconds,
  };
}

function currentCommit(): string {
  try {
    return execFileSync("git", ["rev-parse", "--short", "HEAD"], {
      encoding: "utf8",
    }).trim();
  } catch {
    return "unknown";
  }
}

function cleanTsv(value: string): string {
  return value.replaceAll("\t", " ").replaceAll(/\r?\n/g, " ");
}

function appendResult(row: readonly string[]): void {
  mkdirSync(dirname(RESULTS_PATH), { recursive: true });
  if (!existsSync(RESULTS_PATH)) {
    appendFileSync(
      RESULTS_PATH,
      "commit\tratio\troundtrip_ok\twall_seconds\tstatus\tdescription\n",
    );
  }
  appendFileSync(RESULTS_PATH, `${row.join("\t")}\n`);
}

const args = parseArgs(process.argv.slice(2));
const logText = readFileSync(args.runLog, "utf8");
const maybeMetrics = parseMetrics(logText);
const status =
  maybeMetrics === null
    ? "crash"
    : maybeMetrics.ratio < args.prevRatio && maybeMetrics.roundtripOk
      ? "keep"
      : "discard";
const ratio = maybeMetrics?.ratio ?? 1.0;
const roundtripOk = maybeMetrics?.roundtripOk ?? false;
const wallSeconds = maybeMetrics?.wallSeconds ?? 0.0;
const compressedBytes = maybeMetrics?.compressedBytes ?? 0;

appendResult([
  currentCommit(),
  ratio.toFixed(6),
  String(roundtripOk),
  wallSeconds.toFixed(3),
  status,
  cleanTsv(args.description),
]);

console.log(
  `status=${status} ratio=${ratio.toFixed(6)} roundtrip_ok=${roundtripOk} compressed_bytes=${compressedBytes} wall_seconds=${wallSeconds.toFixed(3)}`,
);
