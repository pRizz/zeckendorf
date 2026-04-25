# Handoff: Autoresearch-style harness for zeckendorf-rs (in progress)

> Self-contained handoff prompt. Paste the body of this file into a fresh agent
> session and it should be able to continue the work without further context.
> User-specific absolute paths are kept verbatim; adjust to your environment.

## Mission

Build an autoresearch-style parameter-sweep harness on top of the `zeckendorf-rs` Rust compression library, modeled on Karpathy's `karpathy/autoresearch` pattern. An autonomous agent edits a TOML config, the harness compresses a fixed corpus, prints one primary metric (total compression ratio), the agent appends a row to `results.tsv`, and either keeps or git-resets the change. Iterates indefinitely.

## Repo

- Path: `/Users/peterryszkiewicz/Repos/zeckendorf-rs`
- Branch: `main`
- The library implements Zeckendorf (Fibonacci-sum) compression. Today's only knob exposed to callers is endianness (BE/LE/best). No preprocessing, no chunking, no parameter sweeps existed before this work.

## The plan

The approved plan lives at `/Users/peterryszkiewicz/.claude/plans/for-our-zeckendorf-rs-project-joyful-fountain.md`. **Read it first.** It contains the autoresearch ↔ zeckendorf concept mapping, the file-by-file design, the ordered implementation steps, and the eight verification checks. Do not redesign — execute against it.

## Decisions baked in (do not relitigate)

1. **Agent scope: parameter sweep only.** The autonomous agent edits TOML configs. It does NOT modify Rust source. New preprocessing transforms must be added by a human/code change, not by the loop agent.
2. **Corpus: synthetic shapes (deterministic) + a small vendored real-file set.**
3. **Primary metric: total compression ratio = `sum(compressed) / sum(uncompressed)` across the corpus.** Lower is better. Round-trip failure on any item forces ratio = 1.0 (penalty).
4. **Scripts in TypeScript, run with Bun.** This is a durable user preference (memo: `~/.claude/projects/-Users-peterryszkiewicz-Repos-zeckendorf-rs/memory/feedback_scripts_typescript_bun.md`). Default to `.ts` + `bun run`, not bash or Python. Existing `scripts/*.sh` predate this preference and stay as-is.
5. **The Rust harness binary `zeck-research` stays Rust** (it has to link the library). Corpus generation, run-log parsing, and result aggregation are TS/Bun.
6. **No changes to production code paths.** The new module is feature-gated under `research`; default `cargo build` excludes it. `padless_zeckendorf_compress_*_dangerous` and `ZeckFile` are reused, never modified.

## What's done

Steps 1–4 complete and tested. Step 5 partially done. 22 research-module unit tests passing.

### Step 1: feature flag + skeleton ✅

- `Cargo.toml`: added `research = ["dep:clap", "dep:toml", "dep:thiserror"]` feature, optional deps, and `[[bin]] zeck-research` entry gated by it.
- `src/lib.rs:84`: added `#[cfg(feature = "research")] pub mod research;`.
- `src/research/mod.rs`: declares `pub mod {config, pipeline, preprocessors};`.
- `src/bin/zeck-research.rs`: stub binary that exits 2 with "skeleton — CLI not yet implemented". Real CLI lands in step 7.

### Step 2: preprocessors ✅

- `src/research/preprocessors.rs`: `Preprocessor` enum with `Identity` and `Rle` variants, serde-tagged `kind` snake_case for TOML. `apply` (Vec<u8>) and `invert` (Result<Vec<u8>, PreprocessError>). RLE is simple `(count, byte)` pairs, splits runs >255. 7 round-trip tests, all passing.

### Step 3: pipeline ✅

- `src/research/config.rs`: `ResearchConfig` (serde): `endianness` (`Be`/`Le`/`Best`, default `Best`), `block_size_bytes: Option<usize>`, `preprocessors: Vec<Preprocessor>`, `corpus_set: String` (default `"all"`).
- `src/research/pipeline.rs`: `compress(cfg, &[u8]) -> Vec<u8>` and `decompress(cfg, &[u8]) -> Result<Vec<u8>, PipelineError>`.
- **Frame format** (research-internal, NOT the on-disk `ZeckFile`): `[flags(1)] [preprocessed_len(8 BE u64)] [zeckendorf_payload...]`. `flags & 1` = endianness used (1=BE, 0=LE). Self-describing on decompress side.
- Empty input is special-cased on both sides (the underlying padless decompress doesn't round-trip empty payload). 10 pipeline tests passing.

### Step 4: block mode ✅

- Added to `pipeline.rs`. When `block_size_bytes = Some(N > 0)`, input is chunked into N-byte blocks and each block is framed independently with format `[u64 BE block_len] [block_frame]` repeated. `block_size_bytes = None` or `Some(0)` means single-frame mode. 5 block-mode round-trip tests passing.

### Step 5: corpus generator ⚠️ PARTIAL

- ✅ `package.json` at repo root: `"type": "module"`, `prepare-corpus`/`log-result`/`summarize` scripts pointing at TS files.
- ✅ `research/prepare.ts` (Bun): generates 8 shapes × 3 sizes = 24 synthetic files into `~/.cache/zeckendorf-research/corpus/synthetic/`. Shapes: `random`, `monotonic`, `sparse_zeros`, `repetitive`, `ascii_text`, `json_ish`, `log_lines`, `low_entropy_struct`. Sizes: 256, 1024, 4096. Idempotent (skips when file exists with matching size). Deterministic via xorshift32 seeded per (shape, size) tuple. **Not yet test-run** — please run `bun run research/prepare.ts` and verify it populates the cache and is idempotent on second invocation.
- ❌ Real-file corpus: `research/corpus/real/` does NOT yet exist. Plan calls for ~3 small public-domain files: a text excerpt (~10KB; e.g. Lincoln's Gettysburg Address), a small JSON sample (~5KB), a small CSV sample (~5KB). Total under 50KB. Vendor these as Step 5's remaining work.

## What's next

In strict order:

### Finish Step 5

1. Run `bun run research/prepare.ts` once — verify it creates `~/.cache/zeckendorf-research/corpus/synthetic/` with 24 files, second invocation reports `0 written, 24 skipped`.
2. Create `research/corpus/real/` with three small vendored files. Public-domain text only. No license risk.
3. Add a `research/.gitignore` ignoring `results.tsv`, `runs/`. Update root `.gitignore` to ignore `~/.cache/zeckendorf-research/` is N/A (outside repo) — just ignore `research/results.tsv` and `research/runs/` from the root if needed.

### Step 6: Rust eval harness (`src/research/eval.rs`)

- Walks corpus files: synthetic dir (`~/.cache/zeckendorf-research/corpus/synthetic/`) + vendored dir (`research/corpus/real/`). Filter by `cfg.corpus_set` (`"all"`, `"synthetic"`, `"real"`, or a specific shape prefix).
- For each file: time `pipeline::compress`, time `pipeline::decompress`, assert recovered bytes match original, accumulate uncompressed/compressed byte totals.
- On any round-trip failure: `roundtrip_ok = false`, ratio forced to `1.0`. Don't abort the harness — finish all files.
- Returns a `RunReport` struct.
- Prints the metric block to stdout:

  ```
  ---
  ratio:               0.812345
  uncompressed_bytes:  1234567
  compressed_bytes:    1003210
  roundtrip_ok:        true
  wall_seconds:        12.3
  peak_rss_mb:         48.2
  config_hash:         a1b2c3d4
  ```

- `peak_rss_mb` is best-effort; `getrusage`-ish on Unix is fine. If awkward, print `0.0` for now and circle back.
- `config_hash` is a short stable hash of the loaded config (e.g. first 8 hex of FNV/SHA over the TOML-serialized form).
- Add a unit test: tiny in-memory corpus (a temp dir with a couple files), identity config, asserts the printed `ratio:` line is parseable and equals what you compute by hand.

### Step 7: `src/bin/zeck-research.rs` real CLI

- Replace the stub. Use `clap` (already pulled in by the `research` feature).
- Args: `--config <path>` required.
- Loads TOML, runs `eval::run(&cfg)`, prints the metric block, exits 0 on harness success (compression failures yield ratio=1.0 but exit 0). Exit non-zero only on harness errors (config parse, missing corpus dir, IO error).

### Step 8: Bun result-logging scripts

- `research/scripts/log_result.ts`: args `--run-log <path>`, `--description <str>`, `--prev-ratio <float>`. Greps the `ratio:`, `roundtrip_ok:`, `compressed_bytes:`, `wall_seconds:` lines from the log. Appends one tab-separated row to `research/results.tsv`. Columns: `commit\tratio\troundtrip_ok\twall_seconds\tstatus\tdescription`. Status is `keep` if ratio strictly less than prev_ratio AND roundtrip_ok, else `discard`. Crash (no metric block) → `crash` status with ratio=1.0. Prints the verdict to stdout for the agent to act on.
- `research/scripts/summarize.ts`: reads `research/results.tsv`, prints best ratio kept, last 10 runs, crash rate. Cheap leaderboard.

### Step 9: PROGRAM.md + baseline config

- `research/configs/baseline.toml`: `endianness = "best"`, no preprocessors, no block size. Reproduces today's `padless_zeckendorf_compress_best_dangerous` behavior.
- `research/PROGRAM.md`: agent instruction sheet. Mirror the structure of [karpathy/autoresearch's `program.md`](https://raw.githubusercontent.com/karpathy/autoresearch/master/program.md): setup steps (create `zeck-research/<tag>` branch, run `bun run research/prepare.ts`, init `results.tsv`, run baseline), experiment loop (edit a config in `research/configs/`, commit, run `cargo run --release --features research --bin zeck-research -- --config <path> > run.log 2>&1`, run `bun run research/scripts/log_result.ts ...`, keep or git-reset), TSV schema, crash policy, never-stop directive.

### Verification (8 checks from the plan)

1. `cargo build` (no features) and `cargo build --all-targets --all-features` both succeed.
2. `cargo clippy --all-targets --all-features -- -D warnings` is clean.
3. `cargo test --all-features` — all preprocessor + pipeline + eval tests pass.
4. `bun run research/prepare.ts` populates the cache, byte-identical on re-run.
5. Baseline run: `cargo run --release --features research --bin zeck-research -- --config research/configs/baseline.toml > run.log 2>&1`, then `grep "^ratio:\|^roundtrip_ok:" run.log`. Expect `roundtrip_ok: true`.
6. Non-trivial config (e.g. `endianness="best", preprocessors=[{kind="rle"}], block_size_bytes=1024`) — round-trip OK, metric line parses, log_result.ts appends correct row.
7. Dry-run the agent loop one iteration on a `zeck-research/dryrun` branch.
8. Default-feature `cargo build` produces no `research` symbols (full gating).

## Reused library functions (do not modify)

- `padless_zeckendorf_compress_be_dangerous` — `src/lib.rs:1027`
- `padless_zeckendorf_compress_le_dangerous` — `src/lib.rs:1076`
- `padless_zeckendorf_compress_best_dangerous` — `src/lib.rs:1338` (returns `PadlessCompressionResult` with struct-style variants `BigEndianBest { compressed_data, le_size }`, `LittleEndianBest { compressed_data, be_size }`, `Neither { be_size, le_size }`)
- `padless_zeckendorf_decompress_be_dangerous` — `src/lib.rs:1239`
- `padless_zeckendorf_decompress_le_dangerous` — `src/lib.rs:1281`

## Project conventions to honor

The user has a global `~/.claude/CLAUDE.md` with strict Rust pre-commit requirements. Before any commit:

1. `cargo fmt --all`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo build --all-targets --all-features`
4. `cargo test --all-features`

All four must pass. Other rules: prefer `thiserror` for library errors, use `?` propagation (no `unwrap()`), prefix `Option`-typed bindings `maybe_`, organize tests one-concept-per-test with `// Arrange / Act / Assert` comments. The existing research code follows these.

## Quirks you'll hit

- The `PreToolUse:Edit` hook fires a "READ-BEFORE-EDIT" reminder repeatedly even when the file was just read in this turn. **Edits succeed regardless** — just re-Read then re-Edit when the hook fires. Don't take it as a hard failure.
- `let ... && ...` (let-chains) are used in `pipeline.rs` — works on this project's `edition = "2024"` Rust.
- The library has a soft 10KB warning per its docs. The corpus stays at 4KB max per file, so we're well under.

## Memory artifacts to be aware of

- `~/.claude/projects/-Users-peterryszkiewicz-Repos-zeckendorf-rs/memory/MEMORY.md` is the index. There's a feedback memory recording the TS+Bun-for-scripts preference — add to it (don't overwrite) if you learn anything new about how the user wants to work.

## One last thing

The user has been engaged and approves of progress so far. Don't pause to ask whether to continue between steps — just push through to step 9 + verification. If a verification check fails, fix the cause and re-run; only escalate if you hit something fundamentally ambiguous (e.g. the metric semantics).

Start by reading the plan file, glancing at `src/research/`, then continue from "Finish Step 5" above.
