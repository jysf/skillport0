---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-005
  type: story                      # epic | story | task | bug | chore
  cycle: design                    # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: M                    # S | M | L  (L means split it)

project:
  id: PROJ-001
  stage: STAGE-002
repo:
  id: skillport

agents:
  architect: claude-opus-4-8      # design cycle (this orchestrator session)
  implementer: claude-sonnet-5    # build runs as a Sonnet subagent (cost); updated with the real model
  created_at: 2026-07-18

references:
  decisions:
    - DEC-003   # severities -> exit codes
    - DEC-005   # deterministic, stable --json schema is a public CI contract
    - DEC-001   # not-a-converter: lint only; no convert/push subcommands
  constraints:
    - deterministic-stable-output
    - no-new-top-level-deps-without-decision
    - test-before-implementation
  related_specs:
    - SPEC-002  # walk -> Collection
    - SPEC-003  # Report::from_collection + exit_code
    - SPEC-004  # lint_skill (the rule_fn)

value_link: "the payoff of STAGE-002 — turns the substrate into a real, runnable `skillport lint` command that drops into CI (the first user-facing capability)"

# Self-reported AI cost per cycle. Each cycle (design, build, verify,
# ship) appends one entry to sessions[]. Totals are computed at ship.
# Record a REAL tokens_total for metered cycles (build/verify) — the
# orchestrator fills it from the Agent result's subagent_tokens at ship
# (or /cost interactively). Only un-metered main-loop cycles (design/ship)
# may be null-with-note. `just cost-audit` enforces this on shipped specs.
# See AGENTS.md §4 and docs/cost-tracking.md. interface: claude-code |
# claude-ai | api | ollama | other.
cost:
  sessions:
    - cycle: design
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-07-18
      notes: "main-loop, not separately metered (design cycle)"
    - cycle: build
      agent: claude-sonnet-5
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-07-18
      notes: "metered subagent; orchestrator fills real tokens_total/estimated_usd/duration_minutes from the Agent result at ship"
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-005: lint command with human and json output

## Context

The payoff spec: it turns the STAGE-001/002 substrate into a **real, runnable
command**. Today `main.rs` is a stub and the only way to run skillport is the
`lint_demo` example. This spec builds the actual `skillport lint <path>` CLI
(clap) that walks a path, runs the rule engine, and emits a human report or
`--json`, with correct CI exit codes and `--strict`. The output contract is
already specified in [`docs/api-contract.md`](../../../docs/api-contract.md) — this
implements it.

- Parent stage: `STAGE-002`, pulled ahead of the remaining rule specs so there's a
  usable command sooner. The remaining rules (`metadata.*`, `allowed-tools.*`,
  `body.*`, `frontmatter.unknown`) are now SPEC-006 and layer into `lint_skill`
  without touching the CLI.
- Reuses: `walk` (SPEC-002), `Report::from_collection` + `exit_code` (SPEC-003),
  `lint_skill` (SPEC-004). The `lint_demo` example is a working reference for the
  wiring and the human format.

## Goal

Implement `skillport lint <PATH> [--json] [--strict]`: walk the path, build the
report via `Report::from_collection(&collection, lint_skill)`, print a
human-readable report (default) or the stable `--json` schema, and exit with the
CI contract code (0 clean / 1 findings-gate / 2 usage error).

## Inputs

- **Files to read (reuse):** `src/walk.rs`, `src/report.rs`, `src/rules.rs`,
  `src/lib.rs`; the working reference `examples/lint_demo.rs`.
- **Output contract (implement exactly):** [`docs/api-contract.md`](../../../docs/api-contract.md)
  — the `lint` flags, exit-code table, and the `--json` shape.
- **Related code paths:** `src/main.rs` (currently a stub — becomes the CLI).

## Outputs

- **Files created:** `src/emit.rs` — `human(&Report) -> String` and
  `json(&Report, target: Option<&str>) -> String` (the `--json` serializer).
- **Files modified:** `src/main.rs` — the clap CLI + dispatch + exit code;
  `src/lib.rs` — expose `emit` if useful for tests; `Cargo.toml` — new deps.
- **New deps (author `DEC-008` in the same build pass):** `clap` (derive) for arg
  parsing, `serde_json` for `--json` (and `serde` declared directly — it's already
  transitively present via the YAML crate). All permissive (MIT/Apache-2.0).
- **CLI surface (implement exactly — see api-contract.md):**
  ```
  skillport lint <PATH> [--json] [--strict]
  ```
  - `<PATH>`: a `SKILL.md` file, a skill folder, or a tree.
  - `--json`: emit the stable JSON schema instead of human output.
  - `--strict`: treat warnings as failures (affects exit code only).
  - `--json` and human are the only formats here; `--sarif` and `--target` are STAGE-003.
- **Database changes:** none.

## Exit codes (from api-contract.md / DEC-003)

| Code | When |
|---|---|
| `0` | no errors (and, under `--strict`, no warnings) |
| `1` | ≥1 error finding — or any warning under `--strict` (i.e. `Report::exit_code(strict)`) |
| `2` | usage error: `<PATH>` does not exist, or bad/missing args (clap) |

Info never affects the exit code. A malformed skill in a bulk run is a
`file.unreadable`/`frontmatter.*` finding, not an abort (already handled by the
substrate).

## `--json` schema (implement exactly; stable public contract, DEC-005)

```json
{
  "tool": "skillport",
  "version": "0.1.0",
  "schema": 1,
  "target": null,
  "summary": { "skills": 2, "errors": 4, "warnings": 1, "infos": 0 },
  "sections": [
    { "path": "lint-fixtures/bad/My-Skill/SKILL.md",
      "findings": [
        { "rule": "name.charset", "severity": "error",
          "message": "…", "field": "name", "line": null }
      ] },
    { "path": "lint-fixtures/good/data-analysis/SKILL.md", "findings": [] }
  ]
}
```
- `version` = the crate version (`env!("CARGO_PKG_VERSION")`); `schema` = `1`;
  `target` = `null` for now (STAGE-003 sets it).
- `severity` serializes as the lowercase string `"error"`/`"warning"`/`"info"`.
- `sections` path-sorted, findings deterministically ordered (the report layer
  already guarantees this — the emitter must not reorder).
- Paths render as display strings.

## Acceptance Criteria

- [x] `skillport lint <PATH>` walks the path, runs `lint_skill` over the
      collection via `Report::from_collection`, and prints a human report.
- [x] `--json` emits the exact schema above (parseable; `tool`/`version`/`schema`/
      `target`/`summary`/`sections` present; severities lowercase strings).
- [x] **Exit codes:** clean → 0; any error → 1; warning-only → 0 but → 1 under
      `--strict`; `<PATH>` missing → 2 (with a stderr message, nothing on stdout).
- [x] Results on **stdout**, diagnostics/usage errors on **stderr** (machine
      consumers read stdout only).
- [x] Deterministic: same input → byte-identical stdout (human and `--json`).
- [x] `--json` and human report the same findings/counts; `--strict` changes only
      the exit code, not the bytes emitted.
- [x] No `convert`/`push`/`profiles` subcommands (DEC-001) — only `lint` (leave the
      command structure open for `audit` later).
- [x] `DEC-008` authored for `clap` + `serde_json` (+ `serde`); deps permissive.
- [x] A path with no `SKILL.md` → 0 sections, summary all zero, exit 0 (human
      output notes "no skills found"; `--json` still valid).

## Failing Tests

Written now (design), before build. Two layers:

- **`src/emit.rs` (mod tests)** — unit-test the emitters on a hand-built `Report`:
  - `"json has the documented envelope"` — parse `json(&report, None)` back
    (via serde_json::Value) and assert `tool=="skillport"`, `schema==1`,
    `target==null`, `summary.errors==<n>`, and a finding's `severity=="error"`.
  - `"json severities are lowercase strings"`.
  - `"json sections preserve report order"` (path-sorted as the report gives them).
  - `"human output contains rule id + severity + message for a finding"` and
    `"human marks a clean section as having no findings"`.
- **`tests/cli.rs`** (integration, runs the built binary via
  `env!("CARGO_BIN_EXE_skillport")` — no extra dep):
  - `"lint good fixture → exit 0"` — `lint lint-fixtures/good` exits 0.
  - `"lint bad fixture → exit 1"` — `lint lint-fixtures/bad` exits 1, stdout
    mentions `name.charset`.
  - `"good fixture --strict → exit 0"` (no warnings in the good fixture).
  - `"lint --json → valid JSON, exit code reflects findings"` — `lint lint-fixtures
    --json` stdout parses as JSON with `summary.errors > 0`, exit 1.
  - `"missing path → exit 2, stderr message, empty stdout"`.
  - `"--strict flips warning-only to exit 1"` — construct/point at a fixture whose
    only finding is a warning (e.g. a skill whose `name != dir` but is otherwise
    valid) → exit 0 without `--strict`, exit 1 with it. (Add a fixture under
    `lint-fixtures/` if needed for this — a warning-only skill.)

## Implementation Context

### Decisions that apply

- `DEC-003` — severity → exit code mapping is `Report::exit_code(strict)`; reuse it.
- `DEC-005` — the `--json` schema, exit codes, and rule ids are a stable public
  contract; `schema: 1` is the version marker. Deterministic stdout.
- `DEC-001` — `lint` only; do not add converter subcommands. Structure the clap
  command so `audit` can be added later (a subcommand enum).

### Constraints that apply

- `deterministic-stable-output` — stdout is byte-identical for identical input;
  don't iterate `HashMap`s; rely on the report layer's ordering.
- `no-new-top-level-deps-without-decision` — `clap` + `serde_json` (+ `serde`) are
  runtime deps → author `DEC-008` in the same pass (sanctioned) and keep them
  permissive-licensed.
- `test-before-implementation` — the Failing Tests are the contract.

### Prior related work

- `SPEC-002/003/004` (shipped) — `walk`, `Report::from_collection` + `exit_code`,
  `lint_skill`. The pipeline is `walk(path)` → `Report::from_collection(&c,
  lint_skill)` → emit → exit.
- `examples/lint_demo.rs` — the exact wiring + a working human format to refine.
  Once the CLI exists, the example can stay (it's a library demo) — do not delete it.
- `initial_stuff/main.rs` / `initial_stuff/emit.rs` — the prototype's clap CLI and
  emitter as a reference (it had convert/push too — take only the `lint` + emit parts).

### JSON serialization approach

Prefer **emitter-local `#[derive(Serialize)]` DTO structs** (built by borrowing a
`&Report`) so `report.rs` stays serde-free and the entire JSON schema lives in
`emit.rs`. Map `Severity` → lowercase string. Duplicating a few fields into DTOs is
fine and keeps the format concern in one place. (If you instead derive `Serialize`
directly on the report types, the schema must still match exactly — but the DTO
approach is preferred for separation.)

### Out of scope (for this spec specifically)

- `--sarif` output and `--target <platform>` — STAGE-003.
- The remaining rules (`metadata.*`, `allowed-tools.*`, `body.*`,
  `frontmatter.unknown`) — SPEC-006; `lint_skill` grows there, the CLI is unchanged.
- Color/TTY styling, progress, config files, `--quiet`/`-v` — later DX.
- The `audit` command — PROJ-002 (just leave the subcommand structure open).

## Notes for the Implementer

- **Missing path → exit 2 BEFORE walking:** `walk` returns an empty collection for
  a missing path (SPEC-002), so the CLI must check `path` existence itself and
  emit a usage error (stderr, exit 2). Everything that exists (even an empty dir)
  goes through the normal path → exit 0.
- **stdout vs stderr:** the report (human or JSON) → stdout; usage errors and any
  diagnostics → stderr. Tests assert stdout is empty on the exit-2 path.
- **Exit:** compute `report.exit_code(strict)` and return it as the process code
  (`std::process::ExitCode`), 2 for usage errors.
- **clap:** derive API; a `Commands` enum with a `Lint { path, json, strict }`
  variant leaves room for `audit`. `#[command(name = "skillport")]`.
- **Determinism:** don't sort or reorder in the emitter — the report is already
  ordered; just render it.
- Keep the human format close to `lint_demo.rs` (it's already readable and tested
  by eye) but you own the exact wording; pin it with the emit unit tests.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-005-cli`
- **PR (if applicable):** not opened this cycle (build cycle only, per prompt)
- **All acceptance criteria met?** yes
- **New decisions emitted:**
  - `DEC-008` — CLI arg parsing with `clap` (derive), `--json` output with
    `serde` + `serde_json`
- **Deviations from spec:**
  - Added one new fixture, `lint-fixtures/warn-only/mismatched-name/SKILL.md`
    (name valid but `!= dir`, otherwise a fully valid skill), as anticipated by
    the spec's Notes/Failing Tests for the `--strict` warning-only test. No
    existing fixture was warning-only-clean, so this was required, not
    optional.
  - `src/lib.rs`'s top-of-file doc comment was left unchanged (still says
    "Later specs add ... the CLI on top") — cosmetic only, not part of any
    acceptance criterion; flagged as a trivial follow-up below rather than
    touched out of caution against unrelated churn.
- **Follow-up work identified:**
  - SPEC-006 (already slated): `metadata.*`, `allowed-tools.*`, `body.*`,
    `frontmatter.unknown` rules — `lint_skill` grows, CLI/emitter unchanged.
  - Minor: refresh `src/lib.rs`'s module doc comment now that the CLI exists
    (cosmetic; not gating).

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — Nothing blocking. The one soft ambiguity was which existing fixture(s)
   the `--strict` warning-only test should point at — none of
   `lint-fixtures/good` or `lint-fixtures/bad` is warning-only-clean (good has
   zero findings at all; bad has several errors alongside its one warning), so
   a new fixture was clearly needed, exactly as the spec's Notes anticipated
   ("Add a fixture under `lint-fixtures/` if needed").

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. `deterministic-stable-output`, `no-new-top-level-deps-without-decision`,
   and `test-before-implementation` fully covered the design space; DEC-001/
   DEC-003/DEC-005 gave unambiguous answers for scope, exit codes, and the
   JSON contract.

3. **If you did this task again, what would you do differently?**
   — Nothing structural. I'd consider naming the new fixture directory
   `lint-fixtures/warn-only/` from the start with an eye toward SPEC-006 adding
   more warning-only cases there (already done), so it reads as a deliberate
   fixture category rather than a one-off.

---

## Reflection (Ship)

*Appended during the **ship** cycle. Outcome-focused reflection, distinct
from the process-focused build reflection above.*

1. **What would I do differently next time?**
   — <answer>

2. **Does any template, constraint, or decision need updating?**
   — <answer — if yes but not done this session, record it in
   `/guidance/signals.yaml`: `type: lesson` (with its N-count) for a recurring
   coding pattern, `type: process-debt` for tooling/process friction. A close
   then forces the decision. See `docs/signals.md`.>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>

4. **Where was the worst defect caught?** — one word from a fixed vocabulary so
   the defect-escape distribution is greppable across specs:
   `design` | `build` | `verify` | `ship` | `escaped` (reached prod/runtime) |
   `none` (clean first try).
   — <one word>
   *(Runtime/operational defects — the escape-prone class — only exist once the
   artifact meets its real host. `escaped` here is a signal to strengthen the
   §12 behavioral pre-flight for that surface.)*
