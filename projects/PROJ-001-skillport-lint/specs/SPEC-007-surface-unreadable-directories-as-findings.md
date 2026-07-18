---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-007
  type: story                      # epic | story | task | bug | chore
  cycle: design                    # frame | design | build | verify | ship
  blocked: false
  priority: medium
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
    - DEC-003   # severity: dir.unreadable is a coverage/operational fact -> warning
    - DEC-004   # collection-first: the walk records what it could/couldn't see
    - DEC-005   # deterministic; never abort; stable rule id
  constraints:
    - deterministic-stable-output
    - collection-first-substrate
    - no-heuristic-error
    - test-before-implementation
  related_specs:
    - SPEC-002  # walk / Collection / CollectionItem (this extends them)
    - SPEC-003  # Report::from_collection (adds the dir.unreadable finding)

value_link: "closes STAGE-002's coverage gap — a directory the walk can't read is surfaced (dir.unreadable) instead of silently dropped, so `lint` never claims clean coverage it didn't have"

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
      notes: "metered subagent; orchestrator fills real numbers from Agent result at ship"
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-007: surface unreadable directories as findings

## Context

A STAGE-002 loose end (signal `walk-unreadable-dirs`): today, when the walker
hits a directory it can't read (`std::fs::read_dir` errors — e.g. permission
denied), it **silently skips that subtree**. So `skillport lint` can report a
clean tree while having quietly failed to check part of it — a real trust gap for
a linter. This spec surfaces that case as a `dir.unreadable` finding so coverage
is never silently incomplete. The walk still never aborts (DEC-005) — other skills
in the tree are unaffected.

This spec also **closes `key.duplicate`** (the other STAGE-002 backlog item) as
**resolved-redundant**: investigation showed the parser does *not* take
last-write-wins on duplicate frontmatter keys — `serde_yaml_ng` rejects them, so a
duplicate key already surfaces as `frontmatter.invalid` (error) with a precise
message (`duplicate entry with key "name"`). A separate `key.duplicate` rule would
add redundant public surface (DEC-005) for no gain, so it is **not** added; the
backlog item is closed here.

- Parent stage: `STAGE-002` (the last loose end — after this, the stage's backlog
  is done except the STAGE-003-owned `body.size`/`--target`).
- Extends: `walk` / `Collection` / `CollectionItem` (SPEC-002) and
  `Report::from_collection` (SPEC-003).

## Goal

When the walk cannot read a directory, record it as a new
`CollectionItem::UnreadableDir { path, error }` (instead of silently dropping the
subtree), and have `Report::from_collection` turn it into a `dir.unreadable`
**warning** finding — so `lint` reports incomplete coverage rather than hiding it.

## Inputs

- **Files to read (extend):** `src/walk.rs` (the `collect` recursion + the
  `let Ok(entries) = read_dir(dir) else { return; }` swallow — this is the site to
  change; the `CollectionItem` enum + its `path()`), `src/report.rs`
  (`from_collection`'s `match` over `CollectionItem`, the `FILE_UNREADABLE` pattern
  to mirror).
- **Related code paths:** `tests/` if you add a CLI-level check; `src/lib.rs`
  re-exports if the new variant needs exporting (it's part of `CollectionItem`).

## Outputs

- **Files modified:**
  - `src/walk.rs` — add `CollectionItem::UnreadableDir { path, error }`; when
    `read_dir(dir)` errors during the walk, record an `UnreadableDir` for that dir
    instead of returning silently; include it in the path-sorted items; update
    `CollectionItem::path()` for the new variant.
  - `src/report.rs` — handle `CollectionItem::UnreadableDir` in `from_collection`:
    emit one `dir.unreadable` **warning** finding (`const DIR_UNREADABLE =
    "dir.unreadable"`); do **not** increment `summary.skills` (it's not a skill),
    same as the `Unreadable`/`file.unreadable` arm.
- **No new exports** beyond the enum variant (re-exported via `CollectionItem`).
- **No new dependency. No CLI/emitter change** — the new finding renders through
  the existing `emit` (human + `--json`) unchanged.
- **Database changes:** none.

## Design decisions

- **Severity = warning** (not error). `file.unreadable` is an **error** because a
  `SKILL.md` was found but couldn't be validated. An unreadable **directory** is a
  *coverage/operational* fact — it may contain no skills at all — so a `warning`
  is proportionate: it's visible in output and fails CI under `--strict`, but a
  filesystem-permission quirk doesn't hard-fail a plain `lint`. It is a crisp
  mechanical fact (not a heuristic), so warning is consistent with DEC-003.
- **Shape:** reuse the existing collection→section model. `UnreadableDir` becomes
  its own `Section` (path = the dir) with the single `dir.unreadable` finding,
  path-sorted among the skill sections (deterministic, DEC-005). This keeps `--json`
  uniform (a section per item) and needs no emitter change.
- **Root that can't be read:** if `walk`'s `root` is itself an unreadable directory,
  it yields one `UnreadableDir` for the root (the CLI's path-exists check passed;
  the read failure is a real finding, exit stays 0 unless `--strict`).

## Acceptance Criteria

- [x] `CollectionItem::UnreadableDir { path, error }` exists; `CollectionItem::path()`
      returns its `path`; the variant participates in the path sort.
- [x] A directory whose `read_dir` fails (permission denied) is recorded as
      `UnreadableDir`, **not** silently skipped; sibling skills in the same walk are
      still discovered (walk never aborts — DEC-005).
- [x] `Report::from_collection` maps `UnreadableDir` → exactly one finding with
      `rule == "dir.unreadable"`, `severity == Warning`, the dir's path; a message
      that makes clear the subtree wasn't checked; `summary.skills` unchanged by it.
- [x] `dir.unreadable` counts toward `summary.warnings`; it flips the exit code to
      1 only under `--strict` (via the existing `Report::exit_code`).
- [x] Deterministic: `UnreadableDir` sections are path-sorted with the rest; same
      tree → byte-identical output.
- [x] End-to-end: `skillport lint <tree-with-an-unreadable-subdir>` shows the
      `dir.unreadable` warning in human and `--json` output (no emitter change).
- [x] No new dependency; `main.rs`/`emit.rs` unchanged (except an optional test).
- [x] The good fixture still yields zero findings; existing tests still pass.
- [x] `key.duplicate` is **not** added (documented here as resolved-redundant); the
      STAGE-002 backlog item is closed.

## Failing Tests

Written now (design). Filesystem permission tests are **Unix-only** (like SPEC-002's
symlink test) — gate them `#[cfg(unix)]` and restore permissions in the test so the
temp dir can be cleaned up.

- **`src/walk.rs` (mod tests, `#[cfg(unix)]`)**
  - `"unreadable subdir → UnreadableDir item, siblings still found"` — build a temp
    tree with `good/SKILL.md` and a `locked/` dir `chmod 000`; `walk` yields the
    `good` skill **and** an `UnreadableDir` for `locked/`; then `chmod 0755 locked`
    to allow cleanup. Assert the skill is present (walk didn't abort).
  - `"items including UnreadableDir are path-sorted"`.
- **`src/report.rs` (mod tests)** — construct a `Collection` with an `UnreadableDir`
  item directly (no filesystem needed):
  - `"UnreadableDir → one dir.unreadable warning, skills unchanged"` — asserts
    `rule=="dir.unreadable"`, `severity==Warning`, `summary.warnings==1`,
    `summary.skills` counts only real skills.
  - `"dir.unreadable is the exact stable id"`.
  - `"exit_code: dir.unreadable warning → 0 non-strict, 1 strict"`.
- **`tests/cli.rs` (`#[cfg(unix)]`, optional but preferred)** — a temp tree with a
  `chmod 000` subdir: `skillport lint <tree>` stdout contains `dir.unreadable`;
  exit 0 without `--strict`, 1 with `--strict`; restore perms after.

## Implementation Context

### Decisions that apply

- `DEC-003` — `dir.unreadable` is a crisp coverage fact → **warning** (rationale
  above); not error, not a heuristic.
- `DEC-004` — the collection records what the walk saw *and* couldn't see; the
  audit (PROJ-002) reuses this, so surfacing coverage gaps belongs in the substrate.
- `DEC-005` — deterministic (path-sorted), never abort (one unreadable dir doesn't
  stop the walk), and `dir.unreadable` joins the stable rule-id contract.

### Constraints that apply

- `deterministic-stable-output`, `collection-first-substrate`, `no-heuristic-error`,
  `test-before-implementation`.

### Prior related work

- `SPEC-002` — `walk`/`collect`. The exact change site: in `collect`,
  `let Ok(entries) = std::fs::read_dir(dir) else { return; };` currently swallows
  the error. Replace with recording an `UnreadableDir` for `dir` (thread a way to
  push items/dir-errors out of the recursion — e.g. `collect` also takes a
  `&mut Vec<(PathBuf, String)>`, or push a `CollectionItem` directly).
- `SPEC-003` — `from_collection`'s `match`; mirror the `Unreadable`/`FILE_UNREADABLE`
  arm for `UnreadableDir`/`DIR_UNREADABLE`.

### Out of scope (for this spec specifically)

- `body.size` / tokenizer, `--target`, `--sarif` — STAGE-003.
- Any new lint *rule* over skill content — this is a substrate/coverage finding.
- A dedicated `key.duplicate` rule — resolved-redundant (see Context); do **not** add it.
- Changing `file.unreadable` behavior or severity — unchanged.

## Notes for the Implementer

- **Threading dir errors out of `collect`:** the current `collect` gathers
  `Vec<PathBuf>` then maps to `read_item`. Simplest: give `collect` a second
  out-param `dir_errors: &mut Vec<(PathBuf, String)>`; in `walk`, turn those into
  `CollectionItem::UnreadableDir` and extend `items` before the final sort.
- **Ignored dirs:** a `.git`/`node_modules`/`target` dir that is *also* unreadable
  should still be skipped silently (they're intentionally ignored — don't emit
  `dir.unreadable` for them). Only emit for directories the walk actually tried to
  descend.
- **Message:** something like `format!("could not read directory (skills inside
  were not checked): {error}")`.
- **Test hygiene:** always `chmod` the locked dir back to a readable mode at the end
  of the test (even on assert failure, prefer a guard) so `tempfile` cleanup works.
- Confirm `cargo run --example lint_demo` and `skillport lint` still behave
  identically on the normal fixtures (the new variant only appears on read errors).

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-007-unreadable-dir`
- **PR (if applicable):** not opened (build cycle only, per orchestrator instructions)
- **All acceptance criteria met?** yes
- **New decisions emitted:**
  - none
- **Deviations from spec:**
  - None in behavior/design. Implementation detail: `collect`'s out-param is
    `&mut Vec<(PathBuf, String)>` (dir errors only), converted to
    `CollectionItem::UnreadableDir` in `walk` right before the final sort —
    matches the spec's suggested approach exactly (Notes section).
- **Follow-up work identified:**
  - none — this closes the STAGE-002 backlog (both `walk-unreadable-dirs` and
    the resolved-redundant `key.duplicate` item).

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — Nothing; the spec's Notes section named the exact swallow site and a
   concrete threading strategy, so the change was mechanical.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No new constraint needed; DEC-003/004/005 fully covered the severity,
   substrate, and determinism questions this spec raised.

3. **If you did this task again, what would you do differently?**
   — Nothing meaningful; would follow the same out-param threading approach.

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
