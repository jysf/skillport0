# SPEC-018 — BUILD prompt (Sonnet subagent)

You are the **implementer** for `SPEC-018: audit command + inventory` — the first spec of
PROJ-002 (the `audit` wave). You run as a metered subagent on branch
`feat/spec-018-audit-inventory`, already created and checked out — **commit to the current
branch; do not create/switch branches, open a PR, or merge.** The spec is your source of
truth.

> **`audit` is a new, additive command — do NOT change `lint`.** No change to `lint`'s
> output, its `--json`/SARIF schema, its exit codes, or any rule id (DEC-005). `audit` is a
> **report, not a CI gate** (DEC-003): exit 0 on a normal run, 2 on a usage error, no
> `--strict`/gating. No new dependency. This spec is **inventory only** — no overlap, no
> health flags (those are SPEC-019/020).

## Read first (in order)

1. `projects/PROJ-002-skillport-audit/specs/SPEC-018-audit-command-and-inventory.md` —
   Outputs, Acceptance Criteria, Failing Tests, Notes, Out of scope.
2. `src/main.rs` (the `Commands` subcommand enum + `Lint` arm + exit-code/emit wiring),
   `src/walk.rs` (`walk` / `Collection` / `CollectionItem`), `src/skill.rs` (`Skill`:
   `path`, `dir_name`, `get("name")`, `body`), `src/emit.rs` (the `--json` `Envelope`/DTO
   pattern + `schema` const), `src/rules.rs` (`body_token_count` ~line 60, private),
   `src/report.rs` (`Severity`), `src/lib.rs`.
3. `decisions/DEC-003`, `DEC-004`, `DEC-005`.

## Your job

1. **Expose the tokenizer:** make `body_token_count` `pub` (a neutral `pub fn
   token_count(text: &str) -> usize` is nicer since it's not rule-specific — keep the
   `body.size` rule using the same underlying count) and re-export from `lib.rs`. Do NOT
   change its behavior or the `body.size` threshold/rule.
2. **`src/audit.rs` (new):** `InventoryRow { name, path, tokens, bytes, lines }`
   (`name` = frontmatter `name` if a string, else `dir_name`, else file stem; `tokens` =
   `token_count(&skill.body)`); an `AuditReport` **designed to grow** (e.g. `{ inventory:
   Vec<InventoryRow>, summary: AuditSummary /*, sections later */ }`) with `AuditSummary {
   skills, tokens_total, unreadable }`; `pub fn audit_collection(&Collection) ->
   AuditReport` — one row per readable skill (path-sorted, deterministic), unreadable
   files/dirs **counted in the summary, not dropped** (never panic/abort).
3. **`src/emit.rs`:** `pub fn audit_human(&AuditReport) -> String` (inventory table:
   name, path, `~<tokens> tokens` as the headline metric; then a summary line) and
   `pub fn audit_json(&AuditReport) -> String` with a **separate** `AUDIT_SCHEMA: u32 = 1`
   const, an audit discriminator (e.g. `"kind":"audit"`), `tool`/`version`, `summary`, and
   an `inventory` array of `{name, path, tokens, bytes, lines}`. Emitter-local
   `#[derive(Serialize)]` DTOs, same style as lint's `Envelope`. Never panics. Do NOT touch
   the lint envelope/schema.
4. **`src/main.rs`:** add `Commands::Audit { path: PathBuf, #[arg(long)] json: bool }`; the
   arm walks the path → `audit_collection` → prints `audit_json`/`audit_human` to stdout →
   exit 0; a missing path is a usage error → exit 2, message on stderr (mirror `lint`).
5. **`src/lib.rs`:** re-export `audit_collection`, `AuditReport`, `InventoryRow`, the audit
   emitters, `token_count`.
6. Write **every Failing Test** in the spec (unit tests in `src/audit.rs`, CLI tests in
   `tests/cli.rs`), including the tokenizer-pin (`tokens_use_the_real_tokenizer`), the
   unreadable-counted test, and the determinism + exit-code CLI tests.

## Definition of done

- Every **Acceptance Criterion** met; every **Failing Test** passes.
- `cargo test` green · `cargo clippy --all-targets -- -D warnings` clean ·
  `cargo fmt --check` clean. **No new dependency.**
- **`lint` is unchanged** — its tests, output, `--json`/SARIF schema, exit codes, and rule
  ids are untouched (`git diff main` on `src/main.rs`/`src/emit.rs` should show only
  *additions* for audit, and the `body.size` rule/tests still pass).
- PASTE real output of `audit lint-fixtures/good` (human) and `audit lint-fixtures/good
  --json`.
- Fill the spec's **## Build Completion**, append a **build** cost session (null numerics,
  per `projects/_templates/prompts/cost-snippet.md`), set `agents.implementer` to your
  model, commit to `feat/spec-018-audit-inventory` (`feat(SPEC-018): …`). Do **not** advance
  cycle, PR, or merge.

## Return (final message = data for the orchestrator)

Concise + factual: files changed; confirm all ACs/tests pass with exact `cargo test`/
`clippy`/`fmt` lines; PASTE the `audit` human + `--json` output for `lint-fixtures/good`;
state the audit `schema` value + how it's discriminated from lint; confirm `lint` is
unchanged + no new dep + determinism (json run twice identical); note how unreadable items
are counted; any deviations/follow-ups.
