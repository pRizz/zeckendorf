# zeckendorf-rs autoresearch program

This program runs autonomous compression experiments against `zeckendorf-rs`.
The fixed evaluator prints one primary metric: total compression ratio across the
fixed corpus. Lower is better.

## Invocation

A real run starts by creating a dedicated `zeck-research/<tag>` branch and
launching an autonomous coding agent in this repository. Tell the agent to read
and follow this file exactly. The agent should edit TOML configs only, run the
fixed evaluator, log each result, keep commits only when the ratio improves and
round-trip succeeds, and otherwise reset the experiment commit.

## Setup

To set up a new run:

1. Agree on a run tag based on the date or machine, for example `apr25` or
   `apr25-mac`.

1. Create a fresh branch from `main`:

   ```bash
   git checkout main
   git pull --rebase
   git checkout -b zeck-research/<tag>
   ```

1. Read the in-scope files:

   - `README.md` - project context and compression caveats.
   - `research/PROGRAM.md` - this experiment loop.
   - `research/configs/*.toml` - configs the loop may edit or copy.
   - `research/prepare.ts` - fixed corpus generator. Do not modify.
   - `src/research/` - fixed evaluator and pipeline. Do not modify during a run.

1. Prepare the synthetic corpus:

   ```bash
   bun run research/prepare.ts
   ```

1. Initialize `research/results.tsv` if it does not exist:

   ```text
   commit	ratio	roundtrip_ok	wall_seconds	status	description
   ```

1. Run the baseline once:

   ```bash
   mkdir -p research/runs
   cargo run --release --features research --bin zeck-research -- --config research/configs/baseline.toml > research/runs/baseline.log 2>&1
   grep "^ratio:\|^roundtrip_ok:" research/runs/baseline.log
   bun run research/scripts/log_result.ts --run-log research/runs/baseline.log --description "baseline" --prev-ratio Infinity
   ```

## Experimentation

Each experiment evaluates one TOML config against the fixed corpus.

What you can do:

- Edit or create configs under `research/configs/`.
- Change `endianness`, `block_size_bytes`, `preprocessors`, and `corpus_set`.
- Commit config changes before running the evaluator.

What you cannot do:

- Do not modify Rust source during the autonomous loop.
- Do not modify `research/prepare.ts`, `src/research/`, or `src/bin/zeck-research.rs`.
- Do not add dependencies.
- Do not edit the corpus to improve the score.

The goal is the lowest `ratio`. A config must round-trip successfully to count
as a keep. Simpler configs are preferred when ratios are equal or nearly equal.

## Output format

The evaluator prints:

```text
---
ratio:               0.812345
uncompressed_bytes:  1234567
compressed_bytes:    1003210
roundtrip_ok:        true
wall_seconds:        12.300
peak_rss_mb:         0.0
config_hash:         a1b2c3d4
```

Extract the primary lines with:

```bash
grep "^ratio:\|^roundtrip_ok:" research/runs/<name>.log
```

## Logging results

Use `research/results.tsv`. It is tab-separated and gitignored. Columns:

```text
commit	ratio	roundtrip_ok	wall_seconds	status	description
```

Status values:

- `keep` - ratio is strictly lower than the previous best and `roundtrip_ok` is true.
- `discard` - ratio did not improve, or round-trip failed.
- `crash` - no metric block was produced.

Use the logger instead of hand-formatting rows:

```bash
bun run research/scripts/log_result.ts --run-log research/runs/<name>.log --description "<short description>" --prev-ratio <best-ratio>
```

Use the summarizer to inspect progress:

```bash
bun run research/scripts/summarize.ts
```

## Experiment loop

Loop forever on the dedicated `zeck-research/<tag>` branch:

1. Check the current git state and best logged ratio.

1. Create or edit one config under `research/configs/`.

1. Commit the config change.

1. Run the evaluator, redirecting all output to a log:

   ```bash
   cargo run --release --features research --bin zeck-research -- --config research/configs/<name>.toml > research/runs/<name>.log 2>&1
   ```

1. Read the metric lines:

   ```bash
   grep "^ratio:\|^roundtrip_ok:" research/runs/<name>.log
   ```

1. Log the result:

   ```bash
   bun run research/scripts/log_result.ts --run-log research/runs/<name>.log --description "<short description>" --prev-ratio <best-ratio>
   ```

1. If status is `keep`, leave the commit in place and continue from it.

1. If status is `discard` or `crash`, reset back to the commit before the
   experiment and try a different config.

If a run exceeds 10 minutes, kill it and treat it as a failed experiment. If a
crash is a simple config typo, fix the config and rerun. If the idea is broken,
log the crash and move on.

Never stop once the loop starts. Do not ask whether to keep going. Continue
trying configs until manually interrupted.
