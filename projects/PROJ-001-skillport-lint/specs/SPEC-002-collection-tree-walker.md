---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-002
  type: story                      # epic | story | task | bug | chore
  cycle: design                    # frame | design | build | verify | ship
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
  implementer: claude-sonnet-5  # build runs as a Sonnet subagent (cost); updated with the real model
  created_at: 2026-07-18

references:
  decisions:
    - DEC-004   # collection-first: walk returns a COLLECTION, never a bare skill
    - DEC-005   # deterministic (path-sorted); one bad/unreadable file never aborts the walk
  constraints:
    - deterministic-stable-output
    - collection-first-substrate
    - test-before-implementation
  related_specs:
    - SPEC-001  # reuses parse(path, raw) -> Skill and the Skill model

value_link: "infrastructure enabling STAGE-001's collection (a path -> N skills) that both lint and the PROJ-002 audit consume"

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
      notes: "implementer subagent; orchestrator fills real tokens_total at ship"
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-002: collection tree-walker

## Context

Second spec of STAGE-001. SPEC-001 shipped `parse(path, raw) -> Skill` (per-file).
This spec adds the **collection** layer: turning a path — a single `SKILL.md`, a
skill folder, or a whole tree — into an ordered set of parsed skills. This is the
thing that makes skillport collection-first (DEC-004): `lint` runs the rule engine
over the collection, and the PROJ-002 `audit` reuses the exact same walk. Like the
parser, the walk must be **tolerant** (an unreadable or non-UTF-8 file is a
per-item entry, never an aborted run — DEC-005) and **deterministic** (items
sorted by path).

- Parent stage: `STAGE-001-core-substrate` (spec 2 of 3).
- Reuses: SPEC-001's `parse` + `Skill` (PR #1, shipped).
- Design docs: [`docs/architecture.md`](../../../docs/architecture.md) (discover
  stage), [`docs/data-model.md`](../../../docs/data-model.md) (`Collection`).

## Goal

Implement `walk(root) -> Collection`: discover every skill under a path (a single
file is a 1-item collection; a directory is walked recursively, skipping `.git` /
`node_modules` / `target`), parse each via SPEC-001's `parse`, and return the items
**sorted by path** — with unreadable files captured as per-item entries so one bad
file never aborts the walk.

## Inputs

- **Files to read (reuse):** `src/parse.rs`, `src/skill.rs` (SPEC-001) — `walk`
  calls `parse` and returns `Skill`s; do not re-implement parsing.
- **Related code paths:** creates `src/walk.rs`; wires the module into `src/lib.rs`.
- **Fixtures:** `lint-fixtures/good/` exists at repo root (one sample); tests build
  their own temp trees (see Notes).

## Outputs

- **Files created:** `src/walk.rs` — `walk` + `Collection` + `CollectionItem` +
  tests.
- **Files modified:** `src/lib.rs` — expose the `walk` module.
- **New exports (indicative — final names are the build's call; the shape is fixed):**

  ```rust
  /// An ordered, deterministic set of what a walk discovered. Sorted by path.
  pub struct Collection {
      pub root: PathBuf,
      pub items: Vec<CollectionItem>,   // sorted by item path, ascending, stable
  }

  pub enum CollectionItem {
      /// A discovered SKILL.md that was read and parsed (parse is total, so this
      /// exists even when the skill's frontmatter is Missing/Unclosed/Invalid).
      Skill(Skill),
      /// The file was found but could not be read as UTF-8 text (I/O error,
      /// invalid UTF-8, …). Captured, never fatal.
      Unreadable { path: PathBuf, error: String },
  }

  /// Total: never returns Err, never panics. A missing root yields an empty
  /// Collection (the CLI reports "path not found" as a usage error, later spec).
  pub fn walk(root: &Path) -> Collection;
  ```
- **Database changes:** none.

## Acceptance Criteria

- [ ] **Single file:** `walk` on a path to a `SKILL.md` → a 1-item `Collection`
      whose item is `Skill` with that file parsed. An explicitly-passed file is
      parsed **regardless of its filename** (the user pointed at it on purpose).
- [ ] **Directory (recursive):** `walk` on a directory discovers **every** file
      named exactly `SKILL.md` beneath it, at any depth, each parsed to a `Skill`.
- [ ] **Ignored dirs:** subtrees named `.git`, `node_modules`, or `target` are
      **not descended into** — a `SKILL.md` inside them is not discovered.
- [ ] **Deterministic order:** `items` are sorted by path ascending; the same tree
      yields the same order on every run and OS (DEC-005). (Do not rely on
      `read_dir` order.)
- [ ] **Tolerant / never aborts:** a file named `SKILL.md` that is not valid UTF-8
      (or otherwise unreadable) becomes a `CollectionItem::Unreadable`, and the walk
      **continues** and still returns every other skill (DEC-005).
- [ ] **`dir_name` populated:** each discovered `Skill` carries its parent
      directory name (SPEC-001's `Skill.dir_name`), so the later `name.dir-match`
      rule works.
- [ ] **Malformed frontmatter still walks:** a discovered `SKILL.md` whose
      frontmatter is `Missing`/`Unclosed`/`Invalid` is still returned as a `Skill`
      item (parse is total) — the walk does not treat it as unreadable.
- [ ] **Empty / missing root:** an empty directory → empty `Collection`; a
      non-existent path → empty `Collection` (no panic, no error type).
- [ ] **No symlink loops:** directory symlinks are not followed (or are otherwise
      guarded), so a self-referential symlink cannot cause an infinite walk.
- [ ] **Totality:** `walk` returns `Collection` (no `Result`), never panics on any
      of the above.

## Failing Tests

Written now (design), before build. Location: `#[cfg(test)] mod tests` in
`src/walk.rs`. Tests construct temp directory trees (see Notes) and assert on the
resulting `Collection`.

- **`src/walk.rs` (mod tests)**
  - `"single SKILL.md file → one Skill item"` — path to a lone `SKILL.md` → 1 item,
    `Skill`, name parsed.
  - `"explicit non-SKILL filename is still parsed"` — `walk` on `foo.md` → 1 `Skill`
    item (explicit file is honored regardless of name).
  - `"recursive discovery finds nested SKILL.md"` — tree with `a/SKILL.md`,
    `b/c/SKILL.md` → 2 `Skill` items.
  - `"items are sorted by path"` — a tree whose `read_dir` order is unlikely to be
    sorted → `items` paths are in ascending order.
  - `"ignores .git, node_modules, target"` — `SKILL.md` placed inside each of
    `.git/`, `node_modules/`, `target/` plus one real `skill/SKILL.md` → only the
    real one is discovered (count == 1).
  - `"unreadable (non-UTF-8) file → Unreadable item, walk continues"` — a
    `bad/SKILL.md` with invalid UTF-8 bytes next to a `good/SKILL.md` → 2 items:
    one `Skill` (good) and one `Unreadable` (bad); both present.
  - `"malformed frontmatter is still a Skill item"` — a `SKILL.md` with no closing
    fence → 1 `Skill` item with `FrontmatterStatus::Unclosed` (not `Unreadable`).
  - `"dir_name is the parent directory"` — `my-skill/SKILL.md` → the item's
    `dir_name == Some("my-skill")`.
  - `"empty dir → empty collection"` and `"missing path → empty collection"` — no
    panic, `items` empty.
  - `"only exact SKILL.md is matched"` — files `skill.md`, `SKILL.MD`, `SKILL.md~`
    in a dir are **not** discovered by a directory walk (only exact `SKILL.md`);
    document the case-sensitivity choice in a comment.
- **fixture-backed**
  - `"walks the repo lint-fixtures/good tree"` — `walk` on `lint-fixtures/good`
    finds `data-analysis/SKILL.md` as a `Skill` item.

## Implementation Context

*Read this section (and the files it points to) before starting the build cycle.*

### Decisions that apply

- `DEC-004` — **collection-first.** `walk` returns a `Collection`, never a bare
  `Skill`; a single file is a 1-item collection. This is the reuse seam for
  PROJ-002's audit.
- `DEC-005` — **deterministic + never abort.** Sort items by path; one unreadable
  file is a per-item entry, not a failure. `walk` is total (no `Result`).

### Constraints that apply

- `deterministic-stable-output` — explicit path sort; no reliance on `read_dir`
  ordering or `HashMap` iteration in anything observable.
- `collection-first-substrate` — this IS the collection layer; keep `walk` a pure
  function of the filesystem so the rule engine (STAGE-002) and audit (PROJ-002)
  both consume its output unchanged.
- `test-before-implementation` — make the Failing Tests above pass; don't add
  behavior without a test.

### Prior related work

- `SPEC-001` (shipped, PR #1) — `parse(path, raw) -> Skill`, the `Skill` model with
  `dir_name` + `FrontmatterStatus`. `walk` reads each file's bytes, converts to a
  `String`, and calls `parse`. UTF-8 conversion failure is the `Unreadable` case
  (parse takes `&str`, so non-UTF-8 can't reach it).

### Out of scope (for this spec specifically)

- Any **rules**, severities, rule ids, or turning `Unreadable`/`FrontmatterStatus`
  into findings — that is the report/finding-model spec (next, SPEC-003) and
  STAGE-002. `walk` returns raw discovery only.
- The CLI, arg parsing, and the "path not found → exit code 2" usage error (that's
  a STAGE-002 CLI spec; `walk` just returns empty for a missing root).
- Emitters (human/JSON/SARIF), `--target`, tokenizer.
- Following symlinks intentionally / a `--follow` option; configurable ignore lists
  (hard-code the three ignored dirs for now).

## Notes for the Implementer

- **Prefer std, no new runtime dep:** a small manual recursion over
  `std::fs::read_dir` handles the ignore-list and no-symlink-follow cleanly — do
  **not** add `walkdir` (keep the dependency surface minimal). If you disagree,
  raise it rather than adding silently.
- **Tests need temp trees:** adding `tempfile` as a **dev-dependency** is
  sanctioned (dev-only, trivial — per the deps constraint's DEC-004-rule-4
  exception); add it and note it in the Build Completion (a one-line DEC is fine
  but not required for a dev-only dep). Alternatively use `std::env::temp_dir()` +
  a unique subdir you clean up — your call, but tests must be hermetic and not
  litter the repo.
- **Non-UTF-8 fixture:** write raw invalid bytes (e.g. `0xFF 0xFE`) to a
  `SKILL.md`; `fs::read` + `String::from_utf8` fails → `Unreadable`. Use
  `fs::read` then `String::from_utf8`, not `read_to_string`, so you can distinguish
  I/O errors from UTF-8 errors if you want a clearer message.
- **Determinism:** collect all matching paths, `sort()` them, then parse in that
  order — simplest way to guarantee stable output.
- **Symlinks:** use `Path::is_dir` semantics carefully; `fs::symlink_metadata` (no
  follow) or a visited-set on canonicalized dirs prevents loops. A simple
  "don't recurse into symlinked dirs" is acceptable for this spec.
- Keep `walk.rs` free of any rule/severity/report types — it depends only on
  `parse`/`Skill` and std.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-002-walker`
- **PR (if applicable):** not opened by build (orchestrator drives verify/ship)
- **All acceptance criteria met?** yes
- **New decisions emitted:**
  - none
- **Deviations from spec:**
  - Root-level symlink handling is slightly more explicit than the spec's minimal
    guidance: `walk` uses `symlink_metadata` to detect the root itself, and if the
    *root path itself* is a symlink (not a symlink encountered *during* recursion),
    it resolves that one, explicit, user-provided root so pointing `walk` directly
    at a symlinked file/dir still works — while symlinks encountered while
    recursing are never followed. This is consistent with "no symlink loops" (a
    symlink can only be resolved once, at the explicit root, never recursively)
    and is covered by `no_symlink_loops`.
  - `SKILL_FILENAME` matching is case-sensitive by design (documented on the
    constant per the spec's "document the case-sensitivity choice in a comment"
    instruction); `only_exact_skill_md_is_matched` asserts `skill.md`, `SKILL.MD`,
    and `SKILL.md~` are all excluded from directory discovery.
- **Follow-up work identified:**
  - None beyond the spec's existing Out-of-scope list (rules/findings/CLI are
    later specs).

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — Nothing major; the fixed Outputs shape and the Notes section (std-only
   recursion, `fs::read` + `String::from_utf8`, sort-then-parse) left little
   ambiguity. The one judgment call was how literally to take "explicit file
   is honored regardless of its name" when the *root itself* is a symlink to a
   file — the spec's symlink guidance is written with directory recursion in
   mind, so I extended the same "don't silently loop, but honor what the user
   explicitly pointed at" spirit to that one edge case and documented it above.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No — DEC-004/DEC-005 plus the three constraints covered everything needed;
   `SPEC-001`'s `parse`/`Skill` reuse seam was exactly as documented.

3. **If you did this task again, what would you do differently?**
   — Nothing structural; would keep the same std-only recursion + sort-then-parse
   approach. Might write the case-sensitivity comment slightly earlier in the
   review pass since it's the one place a reviewer is most likely to ask "why."

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
