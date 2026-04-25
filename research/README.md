# Zeckendorf research harness

This directory contains the experimental autoresearch-style harness for
parameter sweeps. It evaluates TOML compression configs against a fixed corpus
and logs one primary metric: total compression ratio. Lower is better.

The division of responsibility is intentionally narrow:

- Humans and autonomous agents may edit or create TOML configs in
  `research/configs/`.
- The evaluator in `src/research/`, the `zeck-research` binary, the Bun scripts,
  and the corpus are fixed during a run.
- Runtime outputs are local and ignored: `research/results.tsv` and
  `research/runs/`.

## Baseline evaluation

From the repository root:

```bash
bun run research/prepare.ts
mkdir -p research/runs
cargo run --release --features research --bin zeck-research -- --config research/configs/baseline.toml > research/runs/baseline.log 2>&1
grep "^ratio:\|^roundtrip_ok:" research/runs/baseline.log
bun run research/scripts/log_result.ts --run-log research/runs/baseline.log --description "baseline" --prev-ratio Infinity
bun run research/scripts/summarize.ts
```

The baseline config lives at `research/configs/baseline.toml`. The synthetic
corpus is generated into `~/.cache/zeckendorf-research/corpus/synthetic/`; the
small real-file corpus is vendored under `research/corpus/real/`.

## Real autonomous run

Create a dedicated branch before starting the loop:

```bash
git checkout main
git pull --rebase
git checkout -b zeck-research/<tag>
```

Then launch a fresh autonomous coding agent in this repository and give it this
prompt:

```text
You are running a real zeckendorf-rs autoresearch session.

Read research/PROGRAM.md and follow it exactly. Work on the current
zeck-research/<tag> branch. Only edit TOML configs under research/configs/.
Do not modify Rust source, research/prepare.ts, research/scripts/, or the
corpus. Run the fixed evaluator, log each result with
research/scripts/log_result.ts, keep commits only when the ratio improves and
roundtrip_ok is true, and otherwise reset the experiment commit. Continue until
manually interrupted.
```

Use `research/PROGRAM.md` as the canonical instruction sheet for the detailed
loop, TSV schema, crash policy, and keep/discard rule.
