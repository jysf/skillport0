---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-003
  type: story                      # epic | story | task | bug | chore
  cycle: ship  # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: M                    # S | M | L  (L means split it)

project:
  id: PROJ-001
  stage: STAGE-001
repo:
  id: skillport

agents:
  architect: claude-opus-4-8      # design cycle (this orchestrator session)
  implementer: claude-sonnet-4-8  # build ran as a Sonnet subagent (cost)
  created_at: 2026-07-18

references:
  decisions:
    - DEC-003   # severity taxonomy -> exit codes; NO heuristic is error-level
    - DEC-004   # collection-first: report is N sections (one per skill), never a single pass/fail
    - DEC-005   # deterministic ordering; stable rule ids + JSON shape = public contract
  constraints:
    - deterministic-stable-output
    - collection-first-substrate
    - no-heuristic-error
    - test-before-implementation
  related_specs:
    - SPEC-001  # Skill model
    - SPEC-002  # Collection / CollectionItem (walk output the report consumes)

value_link: "completes the STAGE-001 substrate — the sectioned N-skill report + stable-id findings + exit-code logic that lint emits and the PROJ-002 audit reuses"

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
      agent: claude-sonnet-4-8
      interface: claude-code
      tokens_total: 89600
      estimated_usd: 0.59
      duration_minutes: 11
      recorded_at: 2026-07-18
      notes: "metered Sonnet build subagent; tokens_total = subagent_tokens. estimated_usd = tokens x repo rate 6.60 (blended order-of-magnitude, no cache/I-O split). duration wall-clock."
    - cycle: verify
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: 71914
      estimated_usd: 0.47
      duration_minutes: 2
      recorded_at: 2026-07-18
      notes: "metered Opus verify subagent (independent review, APPROVED, 0 punch-list). tokens_total = subagent_tokens. estimated_usd = tokens x 6.60 (order-of-magnitude)."
    - cycle: ship
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-07-18
      notes: "main-loop, not separately metered (ship cycle)"
  totals:
    tokens_total: 161514
    estimated_usd: 1.06
    session_count: 4
shipped_at: 2026-07-18
---

# SPEC-003: finding severity and sectioned report model

## Context

Third and **final** spec of STAGE-001 — the piece that turns discovered, parsed
skills into a report. SPEC-001 gave us `Skill`; SPEC-002 gave us `walk -> Collection`.
This spec adds the **finding + severity + sectioned report model**: the container
types every finding lands in, the assembly that groups findings per skill into a
deterministic sectioned report, the severity → exit-code logic that makes `lint` a
CI gate, and the one **structural** finding the substrate owns (`file.unreadable`,
for a `CollectionItem::Unreadable`). When this ships, STAGE-001 is complete and
STAGE-002's rule engine just supplies rules and emitters on top.

- Parent stage: `STAGE-001-core-substrate` (spec 3 of 3 — closes the stage).
- Reuses: `Skill` (SPEC-001), `Collection`/`CollectionItem` (SPEC-002).
- Design docs: [`docs/data-model.md`](../../../docs/data-model.md) (Finding/Report),
  [`docs/api-contract.md`](../../../docs/api-contract.md) (exit codes),
  [`docs/architecture.md`](../../../docs/architecture.md) (report stage).

## Goal

Implement the report model: `Severity`, `Finding` (stable `rule` id), `Section`,
`Report` with a `Summary`; a `Report::from_collection(collection, rule_fn)`
assembly that maps each `CollectionItem::Unreadable` to a `file.unreadable` error
finding and runs a supplied per-skill rule function over each `Skill`, grouping
findings into **path-sorted** sections with deterministic within-section ordering;
and `Report::exit_code(strict)` implementing the CI contract.

## Inputs

- **Files to read (reuse):** `src/skill.rs` (SPEC-001 `Skill`), `src/walk.rs`
  (SPEC-002 `Collection`, `CollectionItem`). The report **consumes** a `Collection`.
- **Related code paths:** creates `src/report.rs`; wires into `src/lib.rs`.

## Outputs

- **Files created:** `src/report.rs` — the types + assembly + exit-code logic + tests.
- **Files modified:** `src/lib.rs` — expose the `report` module + re-exports.
- **New exports (indicative — final names are the build's call; the shape is fixed):**

  ```rust
  #[derive(Clone, Copy, PartialEq, Eq)]  // plus a stable ordering: Error > Warning > Info
  pub enum Severity { Error, Warning, Info }
  impl Severity { pub fn label(self) -> &'static str; }   // "error" | "warning" | "info"

  pub struct Finding {
      pub rule: &'static str,        // stable id, e.g. "file.unreadable" (public contract, DEC-005)
      pub severity: Severity,
      pub message: String,
      pub path: PathBuf,             // which skill/file
      pub field: Option<String>,     // frontmatter key, when applicable
      pub line: Option<usize>,       // best-effort source line, when cheaply known
  }

  pub struct Section { pub path: PathBuf, pub findings: Vec<Finding> }

  pub struct Summary { pub skills: usize, pub errors: usize, pub warnings: usize, pub infos: usize }

  pub struct Report { pub sections: Vec<Section>, pub summary: Summary }

  impl Report {
      /// Assemble from a walked collection. `rule_fn` is the STAGE-002 rule engine
      /// seam; in STAGE-001 callers pass a no-op (|_| Vec::new()). Unreadable items
      /// become a `file.unreadable` error finding here (structural, owned by the
      /// substrate — not an open-spec rule).
      pub fn from_collection(collection: &Collection, rule_fn: impl Fn(&Skill) -> Vec<Finding>) -> Report;

      /// CI contract (DEC-003/005): 1 if any Error; 1 if `strict` and any Warning;
      /// else 0. Info never affects it. (Usage errors -> exit 2 are CLI-level, not here.)
      pub fn exit_code(&self, strict: bool) -> i32;
  }
  ```
- **Database changes:** none.

## Acceptance Criteria

- [x] **Types exist** as above; `Finding.rule` is a `&'static str` stable id (the
      public contract, DEC-005). `Severity` has a total order Error > Warning > Info.
- [x] **One section per collection item, path-sorted:** `from_collection` produces
      exactly one `Section` per item in the collection, `sections` sorted by `path`
      ascending, deterministically (DEC-004/005) — independent of input order.
- [x] **Unreadable → `file.unreadable` error:** a `CollectionItem::Unreadable`
      becomes a section containing exactly one `Finding` with `rule ==
      "file.unreadable"`, `severity == Error`, and the item's path. (Structural,
      owned here — DEC-003 error is fine: it is a crisp mechanical fact, not a heuristic.)
- [x] **Skill items run `rule_fn`:** for a `CollectionItem::Skill`, the section's
      findings are exactly what `rule_fn(&skill)` returned (in a deterministic order).
- [x] **No-op rule_fn:** with `rule_fn = |_| vec![]` and an all-readable collection,
      every section has empty findings and `summary` counts are all zero — a
      spec-perfect collection yields no findings through this layer (DEC-003).
- [x] **Deterministic within-section order:** findings inside a section are sorted
      by a stable key (severity descending, then `rule` id, then `field`/message) so
      identical input yields byte-identical output (DEC-005). Do not rely on push order.
- [x] **Summary counts:** `summary.skills` = number of `Skill` items (not
      Unreadable), and `errors`/`warnings`/`infos` = totals across all findings.
- [x] **exit_code:** any Error → 1; `strict` && any Warning → 1; Warning without
      strict → 0; Info only → 0; no findings → 0. (Table-tested.)
- [x] **No heuristic, no rules here:** `report.rs` implements **no** open-spec rule
      (no `name.*`, `frontmatter.missing`, etc.) and no heuristic; those arrive via
      `rule_fn` (STAGE-002). The only finding this module emits is `file.unreadable`.

## Failing Tests

Written now (design), before build. Location: `#[cfg(test)] mod tests` in
`src/report.rs`. Build small `Collection`s in-memory (construct `CollectionItem`s
directly — no filesystem needed) and a trivial `rule_fn` closure per test.

- **`src/report.rs` (mod tests)**
  - `"exit_code table"` — Error→1; Warning(!strict)→0; Warning(strict)→1; Info→0;
    empty→0.
  - `"unreadable item → file.unreadable error"` — collection with one `Unreadable`
    → one section, one finding `rule=="file.unreadable"`, `severity==Error`, path matches.
  - `"skill item runs rule_fn"` — `rule_fn` returns one `Warning` finding for a skill
    → that skill's section contains exactly it; `summary.warnings==1`.
  - `"no-op rule_fn on readable collection → zero findings"` — 2 skill items,
    `rule_fn=|_|vec![]` → 2 sections, all empty, summary all zero, `skills==2`.
  - `"sections sorted by path regardless of input order"` — items given out of order
    → `sections` ascending by path.
  - `"findings within a section are deterministically ordered"` — feed a section's
    findings as [Info, Error, Warning] (via rule_fn) → stored order is Error, Warning,
    Info (severity-desc, then rule id).
  - `"summary counts skills and severities"` — mixed collection (1 unreadable + 2
    skills with assorted findings) → correct `skills`, `errors`, `warnings`, `infos`.
  - `"file.unreadable is the exact stable id"` — assert the literal string
    `"file.unreadable"` (guards the public contract).

## Implementation Context

*Read this section (and the files it points to) before starting the build cycle.*

### Decisions that apply

- `DEC-003` — **severity discipline → exit codes.** Error gates CI; Warning gates
  only under `--strict`; Info never. **No heuristic is error-level** — the only
  finding emitted here is `file.unreadable`, which is a crisp mechanical fact (the
  file could not be read), so Error is correct; do not add any judgment call here.
- `DEC-004` — **collection-first.** The report is N sections (one per skill/item),
  never a single pass/fail. This is the shape PROJ-002's audit extends.
- `DEC-005` — **deterministic + stable contract.** Sections path-sorted, findings
  within a section deterministically ordered; `Finding.rule` ids and the report
  shape are a public contract — `file.unreadable` must not be renamed casually.

### Constraints that apply

- `deterministic-stable-output` — explicit sorts; no `HashMap` iteration in
  anything observable; same input → identical `Report`.
- `collection-first-substrate` — this is the report layer the whole substrate feeds;
  keep it free of rule logic so STAGE-002 layers on via `rule_fn`.
- `no-heuristic-error` — no heuristic/analytical finding anywhere in this module.
- `test-before-implementation` — make the Failing Tests pass; no untested behavior.

### Prior related work

- `SPEC-001` (shipped, PR #1) — `Skill` (passed to `rule_fn`).
- `SPEC-002` (shipped, PR #2) — `Collection` / `CollectionItem` (`Skill` |
  `Unreadable { path, error }`). `from_collection` consumes this. The
  `Unreadable.error` string is good `message` material for the `file.unreadable`
  finding.

### The rule_fn seam (why it exists)

STAGE-002's rule engine will be `Fn(&Skill) -> Vec<Finding>` (closing over
`--target` etc.). By taking `rule_fn` as a parameter now, `from_collection` is fully
testable in STAGE-001 with a trivial closure, and STAGE-002 adds rules without
touching this file. Do **not** import or assume any rule here.

### Out of scope (for this spec specifically)

- Any open-spec rule (`name.*`, `description.*`, `frontmatter.missing`, `body.*`,
  `metadata.*`, `allowed-tools.*`, `frontmatter.unknown`) — all STAGE-002. A
  parsed `Skill` with `FrontmatterStatus::Missing/Unclosed/Invalid` is passed to
  `rule_fn` untouched; this module does **not** turn those statuses into findings.
- Emitters (human / `--json` / `--sarif`) — STAGE-002/003. This spec is the model
  the emitters render; do not serialize here (no `serde` derives required unless
  trivial and free — but rendering/format is out of scope).
- The CLI, arg parsing, `--strict` flag wiring, and the usage-error exit code 2
  (STAGE-002 CLI spec). `exit_code(strict)` is the pure function; wiring is later.
- Reporting a permission-denied **subtree** as a finding (signal
  `walk-unreadable-dirs`) — leave for a later spec; only `CollectionItem::Unreadable`
  (a file) becomes `file.unreadable` here.

## Notes for the Implementer

- **No new dependencies.** Pure std + the existing crate. `Finding.rule` is a
  `&'static str` (not `String`) — ids are compile-time constants.
- **Severity ordering:** derive/implement `Ord` so `Error > Warning > Info`
  (Error is "largest"/most severe). The within-section sort is severity-descending;
  a simple `sort_by_key(|f| (Reverse(f.severity), f.rule, f.field.clone()))` works.
- **`from_collection` determinism:** sort `sections` by `path` after building them.
  Don't assume the `Collection` is pre-sorted even though SPEC-002's walk sorts it —
  this function must be correct for any input order (a test enforces it).
- **`file.unreadable` message:** reuse the `Unreadable.error` string, e.g.
  `format!("could not read file: {error}")`. `field`/`line` are `None` for it.
- **Prefer `#[derive(Debug, Clone)]`** on the types for testability. A `serde`
  derive is *not* required here (emitters are out of scope) — don't add `serde`
  wiring in this spec.
- **Keep `report.rs` rule-free:** it must compile and test with zero knowledge of
  any rule; the only id string it contains is `"file.unreadable"`.
- Reuse the `Collection`/`CollectionItem`/`Skill`/`Severity`-adjacent types from
  SPEC-001/002; do not duplicate `Severity` if one already exists (it does not yet —
  create it here as the canonical one).

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-003-report`
- **PR (if applicable):** none yet (build cycle only)
- **All acceptance criteria met?** yes
- **New decisions emitted:**
  - none
- **Deviations from spec:**
  - `exit_code` implemented as `i32::from(errors > 0 || (strict && warnings > 0))`
    rather than a chained `if`/`else if`/`else` — `clippy`'s
    `if_same_then_else` flags the spec's natural if-chain because both the
    `Error` and `strict && Warning` branches return the literal `1`. Same
    truth table, same test coverage (the exit-code table test is unchanged);
    purely a clippy-clean restructuring.
- **Follow-up work identified:**
  - none beyond STAGE-002 (rule engine via `rule_fn`, CLI wiring, emitters),
    already called out as out-of-scope in this spec.

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — Nothing; the Outputs shape, Failing Tests list, and Notes (sort key,
   message format) were specific enough to implement directly without
   guessing.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No new constraint needed. The one friction was cosmetic: the suggested
   `if`/`else if`/`else` for `exit_code` trips clippy's `if_same_then_else`
   lint (both true-branches return `1`); worth a one-line note in future specs
   that "1 if A or B" reads better as a boolean than an if-chain.

3. **If you did this task again, what would you do differently?**
   — Nothing structural; would write `exit_code` as the boolean expression
   from the start instead of the literal if-chain shown in the spec's Notes.

---

## Reflection (Ship)

*Appended during the **ship** cycle. Outcome-focused reflection, distinct
from the process-focused build reflection above.*

1. **What would I do differently next time?**
   — The `rule_fn` seam is the key design bet — taking the rule engine as a
   parameter kept `report.rs` fully testable in STAGE-001 and makes STAGE-002
   purely additive. Worth doing again. Minor: my `exit_code` sketch in the spec was
   an if-chain that trips clippy `if_same_then_else`; the build correctly rewrote it
   to `i32::from(...)`. Next time, sketch spec pseudo-code that's already lint-clean.

2. **Does any template, constraint, or decision need updating?**
   — No. DEC-003/004/005 covered the design exactly; no new decision or signal.
   The metered-subagent pipeline continues to capture real cost cleanly.

3. **Is there a follow-up spec I should write now before I forget?**
   — STAGE-002's first spec (rule engine) now has a concrete target: implement
   `rule_fn: Fn(&Skill) -> Vec<Finding>` producing the open-spec catalog, and wire
   `Report::from_collection` + emitters + the `lint` CLI. The `frontmatter.missing`
   rule must consciously handle the empty-but-`Present` frontmatter case (signal
   `spec-pin-edge-cases`). Already reflected in the STAGE-002 backlog.

4. **Where was the worst defect caught?** — `none` (clean Sonnet build; independent
   Opus verify APPROVED first pass, zero punch-list).
   `design` | `build` | `verify` | `ship` | `escaped` | `none`
