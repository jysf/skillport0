# SPEC-005 — BUILD prompt (Sonnet subagent)

You are the **implementer** for `SPEC-005: lint command with human and json
output`. You run as a metered subagent on branch `feat/spec-005-cli`, already
created and checked out — **commit to the current branch; do not create/switch
branches, open a PR, or merge.** The spec is your source of truth.

## Read first (in order)

1. `projects/PROJ-001-skillport-lint/specs/SPEC-005-lint-command-with-human-and-json-output.md`
   — the CLI surface, the **exit-code table**, the exact **`--json` schema**,
   Acceptance Criteria, Failing Tests, Out of scope, Notes.
2. `docs/api-contract.md` — the authoritative output contract you implement.
3. `src/walk.rs`, `src/report.rs` (`Report::from_collection`, `exit_code`),
   `src/rules.rs` (`lint_skill`), `src/lib.rs`, and `examples/lint_demo.rs` (the
   working wiring + human format to refine — do NOT delete the example).
4. `decisions/DEC-001` (lint only, no converter subcommands), `DEC-003`, `DEC-005`.
5. `guidance/constraints.yaml` (`deterministic-stable-output`,
   `no-new-top-level-deps-without-decision`, `test-before-implementation`),
   `guidance/toolchain-brief.md`. Reference: `initial_stuff/main.rs`,
   `initial_stuff/emit.rs` (take only the `lint` + emit parts).

## Your job

1. Add deps and author **`decisions/DEC-008`** in the same pass (the deps
   constraint sanctions dep + DEC together): `clap` (derive), `serde_json`, and
   declare `serde` directly. Keep them permissive-licensed; explain the choices.
2. Create `src/emit.rs`: `human(&Report) -> String` and `json(&Report, target:
   Option<&str>) -> String`. Implement the **exact** `--json` schema from the spec
   (envelope `tool`/`version`/`schema`/`target`/`summary`/`sections`; `version` =
   `env!("CARGO_PKG_VERSION")`; `schema` = 1; severities as lowercase strings).
   **Prefer emitter-local `#[derive(Serialize)]` DTO structs** so `report.rs` stays
   serde-free. Do NOT reorder — the report is already sorted.
3. Rewrite `src/main.rs` as the clap CLI: `skillport lint <PATH> [--json]
   [--strict]` (a `Commands` enum with a `Lint` variant, leaving room for `audit`).
   Pipeline: check `<PATH>` exists (else stderr + **exit 2**) → `walk(path)` →
   `Report::from_collection(&c, lint_skill)` → `emit::human`/`emit::json` to
   **stdout** → return `report.exit_code(strict)` as the process exit code.
4. Write **every** test in the spec's **Failing Tests** — `src/emit.rs` unit tests
   + `tests/cli.rs` integration tests that run the built binary via
   `env!("CARGO_BIN_EXE_skillport")`. If the `--strict` warning-only test needs a
   warning-only fixture, add one under `lint-fixtures/` (a skill valid except
   `name != dir`).

## Definition of done

- Every **Acceptance Criterion** met; every **Failing Test** passes.
- `cargo test` green · `cargo clippy --all-targets -- -D warnings` clean ·
  `cargo fmt --check` clean.
- `skillport lint lint-fixtures/bad` prints findings and exits 1;
  `skillport lint lint-fixtures/good` exits 0; `--json` emits the documented schema;
  a missing path exits 2 with an empty stdout.
- **In scope:** only `lint` + human/`--json`. NO `--sarif`, NO `--target`, NO new
  rules, NO `convert`/`push`, NO color. DEC-008 written; deps permissive.
- Fill the spec's **## Build Completion**, append a **build** cost session (null
  numerics, per `projects/_templates/prompts/cost-snippet.md`), set
  `agents.implementer` to your model, commit to `feat/spec-005-cli`
  (`feat(SPEC-005): …`). Do **not** advance cycle, PR, or merge.

## Return (final message = data for the orchestrator)

Concise + factual: files changed, deps added + DEC-008, all ACs/tests pass with
exact `cargo test`/`clippy`/`fmt` lines, the actual `skillport lint` output on a
fixture, any deviations, and any follow-ups. Confirm scope (no sarif/target/new
rules) and determinism.
