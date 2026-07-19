---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-018
  type: story                      # epic | story | task | bug | chore
  cycle: build  # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: M                    # S | M | L  (L means split it)

project:
  id: PROJ-002
  stage: STAGE-005
repo:
  id: skillport

agents:
  architect: claude-opus-4-8      # design cycle (this orchestrator session)
  implementer: claude-sonnet-5    # build runs as a Sonnet subagent (cost); updated with the real model
  created_at: 2026-07-19

references:
  decisions:
    - DEC-003   # severity discipline — audit is advisory (a report), not a CI gate
    - DEC-004   # build-for-reuse — audit is an additive layer on the PROJ-001 substrate
    - DEC-005   # deterministic + stable output — audit gets its OWN versioned --json schema
  constraints:
    - collection-first-substrate
    - deterministic-stable-output
    - test-before-implementation
    - no-heuristic-error
  related_specs:
    - SPEC-002  # the walker `walk(root) -> Collection` audit consumes
    - SPEC-003  # the report/Severity model audit reuses for advisory flags
    - SPEC-005  # the clap CLI + emit conventions audit slots alongside
    - SPEC-010  # the real tokenizer (body_token_count) audit reuses for token counts

value_link: "the first `audit` capability: `audit <path>` walks a skill library and reports an inventory (name / path / tokens) + summary — the legibility read that makes a library auditable, and the audit report/emit substrate STAGE-005's overlap/health specs and STAGE-006/007 extend."

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
      recorded_at: 2026-07-19
      notes: "main-loop, not separately metered (design cycle); probed the CLI subcommand shape (audit was pre-anticipated), the Skill model + report/emit surface, and the private body_token_count fn"
    - cycle: build
      agent: claude-sonnet-5
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-07-19
      notes: "metered subagent build; orchestrator fills tokens_total/duration/estimated_usd from the Agent result at ship"
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-018: audit command + inventory

## Context

The first spec of PROJ-002 (STAGE-005) and the first `audit` capability. Where `lint`
answers "does this skill conform?" (per-file, CI-gating), `audit` answers "what's in this
skill *library* and how healthy is it?" (per-collection, an advisory report a human reads).
This spec stands up the `audit` subcommand and its MVP read — an **inventory** — plus the
**audit report + emit substrate** that STAGE-005's later specs (overlap, health heuristics)
and STAGE-006/007 (`--security`, provenance) extend. The CLI was built subcommand-shaped
for exactly this ("`audit` (PROJ-002) can be added later without reshaping `Lint`", per the
`main.rs` doc-comment), so this is additive (DEC-004), not a reshape.

**Framing decisions applied (2026-07-19):** one `audit` command (focus flags like
`--security` come later); the inventory's primary size metric is the **real token count**
(what actually costs an agent's context); this spec is **inventory-only** — overlap
(SPEC-019) and health flags (SPEC-020) layer on next.

## Goal

Add an `audit <path>` subcommand that walks a skill collection and reports a per-skill
**inventory** (name, relative path, token count; bytes + lines in `--json`) plus a summary,
as a sectioned human report and a **new, stable, versioned `--json` audit schema** —
advisory (exit 0; usage → 2), reusing the walker + tokenizer + emit conventions, with no
new dependency and no change to `lint`.

## Inputs

- **Files to read:** `src/main.rs` (the `Commands` subcommand enum + the `Lint` arm +
  `emit`/exit-code wiring), `src/walk.rs` (`walk(root) -> Collection`, `CollectionItem`),
  `src/skill.rs` (`Skill`: `path`, `dir_name`, `get("name")`, `body`), `src/report.rs`
  (`Severity` + how sections carry paths), `src/emit.rs` (the `--json` `Envelope`/DTO
  pattern + `schema` marker), `src/rules.rs` (`body_token_count` at ~line 60 — currently
  private), `src/lib.rs` (re-exports).
- **Related code paths:** `lint-fixtures/` (reuse `good/`, `good-claude/`, `bad/` as audit
  test corpora).
- **No new dependency.**

## Outputs

- **`src/rules.rs` + `src/lib.rs`:** make `body_token_count` `pub` (rename to something
  neutral like `pub fn token_count(text: &str) -> usize` is optional but nice — it's not
  rule-specific) and re-export it from `lib.rs` so `audit` reuses the one real tokenizer.
  Do **not** change its behavior or the `body.size` rule.
- **`src/audit.rs` (new):** the audit model + analysis, reusing the substrate.
  - `pub struct InventoryRow { name: String, path: PathBuf, tokens: usize, bytes: usize,
    lines: usize }` (or similar). `name` = the frontmatter `name` if a string, else
    `dir_name`, else the file stem — always populated. `path` = the skill's path as the
    walker recorded it (same paths `lint` prints). `tokens` = `token_count(&skill.body)`;
    `bytes`/`lines` from the body.
  - `pub struct AuditReport { rows: Vec<InventoryRow>, summary: AuditSummary, /* unreadable
    count etc. */ }` with an `AuditSummary { skills: usize, tokens_total: usize,
    unreadable: usize }`. Rows **path-sorted** (the collection is already path-sorted —
    keep it stable/deterministic; DEC-005).
  - `pub fn audit_collection(collection: &Collection) -> AuditReport` — walks the
    collection items, builds one `InventoryRow` per readable `Skill`, counts unreadable
    files/dirs into the summary (don't drop them silently — a library read must not hide
    coverage gaps; cf. `walk-unreadable-dirs`). This is the seam later specs extend (they
    add sections to `AuditReport`).
- **`src/emit.rs`:** two new audit emitters, mirroring the `lint` emit style but with a
  **separate schema**:
  - `pub fn audit_human(report: &AuditReport) -> String` — a readable inventory table:
    per skill a line with name, path, and `~<tokens> tokens` (tokens is the headline
    metric), then a summary line (`N skill(s), ~T tokens total[, U unreadable]`). Stable,
    path-sorted.
  - `pub fn audit_json(report: &AuditReport) -> String` — a **new** DTO envelope with its
    own `schema` marker (an `AUDIT_SCHEMA: u32 = 1` const, independent of the lint schema)
    and a discriminator so consumers can tell it apart (e.g. `"kind": "audit"` or
    `"report": "audit"`), `tool`/`version`, a `summary` object, and an `inventory` array of
    `{name, path, tokens, bytes, lines}`. Emitter-local `#[derive(Serialize)]` DTOs (same
    pattern as lint's `Envelope`). Never panics.
- **`src/main.rs`:** add `Commands::Audit { path: PathBuf, #[arg(long)] json: bool }`;
  in `main`, the `Audit` arm walks the path, builds the report, prints `audit_json` if
  `--json` else `audit_human` to **stdout**, and returns an exit code: **0** for a normal
  run (audit is a report, not a gate — DEC-003), **2** for a usage error (path does not
  exist), mirroring `lint`'s stderr-for-usage convention. Do **not** add `--strict`/gating.
- **`src/lib.rs`:** re-export the audit surface (`audit_collection`, `AuditReport`,
  `InventoryRow`, the emitters, `token_count`).
- **No change to `lint`, its `--json`/SARIF schema, its exit codes, or any rule id
  (DEC-005).**

## Acceptance Criteria

- [ ] `audit <path>` runs over a single `SKILL.md`, a folder, and a tree, printing a
      per-skill inventory (name, path, token count) + a summary line, to **stdout**, exit
      **0**. `audit <missing-path>` is a usage error → exit **2**, message on **stderr**.
- [ ] The inventory's headline size metric is the **real token count** (via the shared
      tokenizer, not chars/4); `bytes` and `lines` are present in `--json`.
- [ ] `audit --json <path>` emits a **valid, deterministic** JSON document with its **own**
      `schema` marker (distinct from the lint schema) and an audit discriminator, a
      `summary` (skills, tokens_total, unreadable), and an `inventory` array; running twice
      is byte-identical; rows are path-sorted.
- [ ] Unreadable files/dirs in the tree are **counted in the summary** (not silently
      dropped); the walk never aborts on one bad item (reuses `walk`'s guarantees).
- [ ] `body_token_count`/`token_count` is `pub` + re-exported; the `body.size` lint rule
      and its threshold are **unchanged** (its tests still pass).
- [ ] `lint` is **entirely unchanged** — same output, `--json`/SARIF schema, exit codes,
      rule ids. `cargo test`/`clippy -D warnings`/`fmt --check` green; **no new dependency**.

## Failing Tests

Written during **design**, before build. Paths indicative (unit tests may live in
`src/audit.rs`'s `#[cfg(test)]`; CLI tests in `tests/cli.rs`).

- **`src/audit.rs` (unit) — `inventory_row_per_readable_skill`**
  - Build a `Collection` (via `walk` over a temp tree, or hand-constructed) with 2 skills;
    `audit_collection` returns 2 rows, path-sorted, each with the right `name` (frontmatter
    name; falls back to dir name when absent) and a non-zero `tokens` for a non-empty body.
    `summary.skills == 2`.
- **`src/audit.rs` (unit) — `unreadable_items_counted_not_dropped`**
  - A collection containing a `CollectionItem::Unreadable` (and/or `UnreadableDir`) →
    `summary.unreadable >= 1`, and it does not appear as an inventory row / does not panic.
- **`src/audit.rs` (unit) — `tokens_use_the_real_tokenizer`**
  - `token_count` of a known string equals the real tokenizer's count (≠ `len()/4`),
    mirroring the SPEC-010 tokenizer-pin guard — proves audit reuses the real tokenizer.
- **`tests/cli.rs` — `audit_runs_and_reports_inventory`**
  - `audit lint-fixtures/good` exits 0 and stdout contains the skill's name + a token
    count + a summary line; `audit lint-fixtures/good --json` is valid JSON with the audit
    `schema`, a `summary`, and an `inventory` array whose entries have `name/path/tokens`.
- **`tests/cli.rs` — `audit_usage_error_and_determinism`**
  - `audit <missing>` → exit 2, message on stderr, stdout empty. `audit --json
    lint-fixtures/good` run twice → byte-identical. `lint` behavior on the same fixtures
    is unchanged (a smoke assertion).

## Implementation Context

*Read this section before starting the build cycle.*

### Decisions that apply

- `DEC-003` — audit is **advisory**: a report, never a CI gate. Exit 0 on a normal run;
  no `--strict`, no error-severity-fails-CI semantics. (Later health flags are advisory
  too.)
- `DEC-004` — build on the shipped substrate: reuse `walk`, the `Skill` model, the report
  `Severity`, the tokenizer, and the emit pattern. `audit_collection`/`AuditReport` is the
  new seam that STAGE-005's overlap/health specs and STAGE-006/007 extend — design it to
  grow (sections can be added) rather than as a one-off inventory dump.
- `DEC-005` — deterministic + stable: path-sorted rows; a **separate** `AUDIT_SCHEMA`
  (do not overload or bump the lint schema). The audit `--json` is a new public contract
  from v1. (The `tool` field value + any future lockfile name will be swept by the pending
  project rename — don't block on it.)

### Constraints that apply

- `collection-first-substrate` — audit is inherently per-collection; it consumes the
  `Collection` the walker returns.
- `deterministic-stable-output` — stable ordering + a versioned schema; a bad item never
  aborts the run.
- `no-heuristic-error` — nothing here is error-level (it's inventory); keep it that way.
- `test-before-implementation` — the tests above are the spec.

### Prior related work

- `SPEC-002` (walker), `SPEC-003` (report/Severity), `SPEC-005` (CLI + emit), `SPEC-010`
  (the real tokenizer) — all shipped; audit reuses them. `SPEC-008` (SARIF) is **not**
  relevant here (audit is a human/JSON report; no SARIF).

### Out of scope (for this spec specifically)

- **Overlap/collision detection** (SPEC-019) and **health heuristics** (SPEC-020) — this
  spec is inventory + the substrate only.
- **`--security` / permissions manifest** (STAGE-006), **provenance / lockfile**
  (STAGE-007), **AGENTS.md** (parked).
- **No SARIF for audit**, no `--strict`/gating, no autofix, no change to `lint`.

## Notes for the Implementer

- Keep `audit`'s stdout/stderr split identical to `lint`: report → stdout, usage errors →
  stderr, so `audit --json` is pipe-safe.
- Design `AuditReport` to **grow**: overlap (pairwise) and health (per-skill advisory
  `Finding`s) are coming, so a shape like `{ inventory: Vec<Row>, sections: Vec<...>,
  summary }` where this spec fills only `inventory` + `summary` is better than a bare
  `Vec<Row>`. Don't over-build the empty section machinery, but don't paint it into a
  corner either.
- Reuse `Severity`/`Finding` from `report.rs` for later advisory flags — but this spec
  emits no findings yet, so you may not touch them at all.
- The token fn is `body`-named in `rules.rs` but is a generic tokenizer — exposing it as
  `pub fn token_count(text: &str)` (with `body_token_count` kept or aliased if simpler)
  reads better from `audit`. Keep the `body.size` rule calling the same underlying count.
- Mirror the lint `--json` envelope style (emitter-local `#[derive(Serialize)]` DTOs, a
  `schema` const) so the two schemas are consistent in shape but independent in version.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:**
- **PR (if applicable):**
- **All acceptance criteria met?** yes/no
- **New decisions emitted:**
  - `DEC-NNN` — <title> (if any)
- **Deviations from spec:**
  - [list]
- **Follow-up work identified:**
  - [any new specs for the stage's backlog]

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — <answer>

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — <answer>

3. **If you did this task again, what would you do differently?**
   — <answer>

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
