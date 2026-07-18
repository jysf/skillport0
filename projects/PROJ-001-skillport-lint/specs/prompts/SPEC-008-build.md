# SPEC-008 — BUILD prompt (Sonnet subagent)

You are the **implementer** for `SPEC-008: SARIF output format`. You run as a
metered subagent on branch `feat/spec-008-sarif`, already created and checked out
— **commit to the current branch; do not create/switch branches, open a PR, or
merge.** The spec is your source of truth.

## Read first (in order)

1. `projects/PROJ-001-skillport-lint/specs/SPEC-008-sarif-output-format-for-code-scanning.md`
   — the **SARIF 2.1.0 shape (implement exactly)**, the level mapping, Acceptance
   Criteria, Failing Tests, Out of scope, Notes.
2. `src/emit.rs` (mirror the `json` DTO approach), `src/main.rs` (the `Lint` clap
   command + dispatch), `src/report.rs` (`Report`/`Finding`/`Severity`).
3. `docs/api-contract.md` (the `--sarif` placeholder to replace).
4. `decisions/DEC-003`, `DEC-005`.

## Your job

1. `src/emit.rs`: add `pub fn sarif(report: &Report) -> String` with emitter-local
   `#[derive(Serialize)]` DTOs (`SarifLog`/`Run`/`Tool`/`Driver`/`ReportingDescriptor`/
   `SarifResult`/`Message`/`Location`/`PhysicalLocation`/`ArtifactLocation`/`Region`),
   producing the **exact** SARIF 2.1.0 shape in the spec. `#[serde(rename = "$schema")]`
   for the schema field; `skip_serializing_if = "Option::is_none"` for `region`.
   Level map: `Error→"error"`, `Warning→"warning"`, `Info→"note"` (a small
   `sarif_level` fn — do NOT reuse `Severity::label`). `driver.rules` = distinct rule
   ids **sorted** (a `BTreeSet<&str>`); `results` in the report's existing order
   (do NOT reorder). `line → region.startLine` when present.
2. `src/main.rs`: add `sarif: bool` to `Lint` with `#[arg(long, conflicts_with =
   "json")]`; dispatch to `emit::sarif` before json/human. Exit code unchanged
   (`Report::exit_code(strict)`).
3. `src/lib.rs`: re-export `sarif` next to `human`/`json`.
4. `docs/api-contract.md`: replace the `--sarif` placeholder with the shipped shape.
5. Write **every** Failing Test in the spec (`src/emit.rs` unit tests parsing the
   SARIF back via `serde_json::Value`; `tests/cli.rs` integration incl. the
   `--sarif`+`--json` conflict → exit 2).

## Definition of done

- Every **Acceptance Criterion** met; every **Failing Test** passes.
- `cargo test` green · `cargo clippy --all-targets -- -D warnings` clean ·
  `cargo fmt --check` clean.
- `skillport lint lint-fixtures/bad --sarif` prints valid SARIF (parses), exit 1;
  good fixture → `results: []`, exit 0; `--sarif --json` → exit 2, empty stdout.
- **No new dependency** (serde_json already present). Human/`--json` output
  **byte-for-byte unchanged**. No new rule/analysis.
- Fill the spec's **## Build Completion**, append a **build** cost session (null
  numerics, per `projects/_templates/prompts/cost-snippet.md`), set
  `agents.implementer` to your model, commit to `feat/spec-008-sarif`
  (`feat(SPEC-008): …`). Do **not** advance cycle, PR, or merge.

## Return (final message = data for the orchestrator)

Concise + factual: files changed, all ACs/tests pass with exact `cargo test`/
`clippy`/`fmt` lines, confirm no dep and human/json unchanged, PASTE the
`skillport lint lint-fixtures/bad --sarif` output (or a trimmed excerpt), note any
deviations and follow-ups.
