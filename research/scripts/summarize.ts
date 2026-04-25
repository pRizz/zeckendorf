#!/usr/bin/env bun

import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";

type ResultRow = {
  commit: string;
  ratio: number;
  roundtripOk: boolean;
  wallSeconds: number;
  status: string;
  description: string;
};

const RESULTS_PATH = join("research", "results.tsv");

function parseRows(text: string): ResultRow[] {
  const lines = text.trim().split(/\r?\n/);
  if (lines.length <= 1) return [];

  return lines.slice(1).flatMap((line) => {
    const [commit, ratioRaw, roundtripRaw, wallRaw, status, ...descriptionParts] =
      line.split("\t");
    const ratio = Number.parseFloat(ratioRaw);
    const wallSeconds = Number.parseFloat(wallRaw);
    if (!Number.isFinite(ratio) || !Number.isFinite(wallSeconds)) return [];
    return [
      {
        commit,
        ratio,
        roundtripOk: roundtripRaw === "true",
        wallSeconds,
        status,
        description: descriptionParts.join(" "),
      },
    ];
  });
}

function formatRow(row: ResultRow): string {
  return `${row.commit}\t${row.ratio.toFixed(6)}\t${row.roundtripOk}\t${row.wallSeconds.toFixed(3)}\t${row.status}\t${row.description}`;
}

if (!existsSync(RESULTS_PATH)) {
  console.log("No research/results.tsv yet.");
  process.exit(0);
}

const rows = parseRows(readFileSync(RESULTS_PATH, "utf8"));
if (rows.length === 0) {
  console.log("No result rows yet.");
  process.exit(0);
}

const keptRows = rows
  .filter((row) => row.status === "keep" && row.roundtripOk)
  .sort((a, b) => a.ratio - b.ratio);
const crashCount = rows.filter((row) => row.status === "crash").length;
const crashRate = crashCount / rows.length;

console.log("Best kept ratio:");
if (keptRows.length === 0) {
  console.log("  none");
} else {
  console.log(`  ${formatRow(keptRows[0])}`);
}

console.log(`Crash rate: ${(crashRate * 100).toFixed(1)}% (${crashCount}/${rows.length})`);
console.log("Last 10 runs:");
for (const row of rows.slice(-10)) {
  console.log(`  ${formatRow(row)}`);
}
