---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-012
  type: story                      # epic | story | task | bug | chore
  cycle: build  # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: M                    # S | M | L  (L means split it)

project:
  id: PROJ-001
  stage: STAGE-003
repo:
  id: skillport

agents:
  architect: claude-opus-4-8      # design cycle (this orchestrator session)
  implementer: claude-sonnet-5    # build runs as a Sonnet subagent (cost); updated with the real model
  created_at: 2026-07-18

references:
  decisions:
    - DEC-002   # per-platform behavior verified from primary docs — the 5 new CLAUDE_KEYS were re-verified at design
    - DEC-005   # rule ids are a public contract (MAJOR bump to change) — the catalog operationalizes this
    - DEC-003   # severity discipline — the README severity column must reflect the real emitted severities
  constraints:
    - only-verified-constraints-are-firm
    - deterministic-stable-output
    - test-before-implementation
    - no-heuristic-error
  related_specs:
    - SPEC-011  # --target claude + CLAUDE_KEYS (this completes its enumeration)
    - SPEC-006  # the open-spec catalog (metadata/allowed-tools/body/unknown)
    - SPEC-007  # dir.unreadable / file.unreadable structural findings
    - SPEC-005  # the CLI (flags to document)
    - SPEC-008  # --sarif (flag to document)

value_link: "the DX capstone of STAGE-003: a doc-drift-proof rule reference (README table checked against a code catalog), per-rule fixtures + a spec-perfect zero-findings guarantee, and the last 5 verified Claude fields — so a user can trust what skillport enforces and see it demonstrated."

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
      notes: "main-loop, not separately metered (design cycle); includes the design-time doc re-verification of the 5 new CLAUDE_KEYS (WebFetch code.claude.com) and the full severity probe of src/rules.rs + src/report.rs"
    - cycle: build
      agent: claude-sonnet-5
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-07-18
      notes: "metered subagent build; orchestrator fills tokens_total/duration/estimated_usd from the Agent result at ship"
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-012: rule reference readme, per-rule fixtures, complete claude keys

## Context

This is the **last spec of STAGE-003** and the DX capstone of PROJ-001's `lint`.
The rule engine is functionally complete (open-spec catalog 100% + `--target
claude`), but three trust/DX gaps remain:

1. **The README is stale and undocumented as a reference.** Its Status table
   stops at SPEC-006 ("Remaining rules ⏳ next (SPEC-006)"), so it under-sells a
   tool that already enforces the full catalog + `--sarif` + `--target claude` +
   a real tokenizer. And there is **no rule reference** — a user cannot see the
   list of rule ids, their severities, and what each means without reading source.
2. **No drift guard.** Rule ids are a public contract (DEC-005), but nothing stops
   the README and the code from diverging as rules are added. A user who trusts a
   stale table is worse off than one who reads none.
3. **`CLAUDE_KEYS` is incomplete.** SPEC-011's verify flagged that the live
   Frontmatter reference documents **5 more** recognized Claude fields not in
   `CLAUDE_KEYS` (`when_to_use`, `argument-hint`, `agent`, `paths`, `shell`), so a
   real Claude skill using them is still (incorrectly) flagged `frontmatter.unknown`
   under `--target claude`. The user chose to fold this fix into this spec.

The fix for (1)+(2) is a **single source of truth in code** — a rule catalog the
README table is mechanically checked against — plus **per-rule fixtures** that
prove every documented rule fires on a real example, and a **spec-perfect fixture**
that proves a compliant skill yields zero findings.

## Goal

Make skillport's rule surface **documented and drift-proof**: add a code-level rule
catalog (single source of truth), refresh the README with a rule-reference table +
current status + the `--target`/`--sarif`/`--strict` flags, add per-rule good/bad
fixtures + a spec-perfect zero-findings fixture, and complete `CLAUDE_KEYS` with the
5 remaining doc-verified Claude fields — all guarded by tests so the docs can't
silently drift from the code.

## Inputs

- **Files to read (extend):** `src/rules.rs` (`SPEC_KEYS`, `CLAUDE_KEYS`, every
  `push(...)` emit site, the thresholds), `src/report.rs` (the two structural ids
  `file.unreadable`/`dir.unreadable`), `src/lib.rs` (re-exports), `README.md`
  (rewrite Status + add Rule reference + flags), `tests/cli.rs`, the existing
  `lint-fixtures/` tree.
- **Primary docs (re-verified at design — re-check only if editing field facts):**
  https://code.claude.com/docs/en/skills — Frontmatter reference table.
- **Related code paths:** `src/emit.rs` (unchanged — the table documents its
  severities), `src/main.rs` (unchanged — the flags it already exposes are documented).

## Outputs

- **Files modified:**
  - `src/rules.rs`:
    - Add the 5 fields to `CLAUDE_KEYS`, each on its own line with a
      `// source: code.claude.com/docs/en/skills` comment (matching the existing
      8). Order: append `"when_to_use"`, `"argument-hint"`, `"agent"`, `"paths"`,
      `"shell"` (keep it a flat list; the exact order is not contractual).
    - Add a **rule catalog**: `pub const RULES: &[RuleDoc]` (or
      `pub fn all_rule_ids() -> &'static [&'static str]`) — the single, ordered,
      duplicate-free list of **every** rule id the engine can emit (the 24 engine
      ids below). Prefer a `RuleDoc { id, severity, summary }` struct so the
      catalog also carries the default severity and a one-line summary (this lets
      the README-drift test check ids *and* severities against one source). Export
      it from `lib.rs`. `allowed-tools.format`'s catalog severity is its **default**
      (`Warning`); the `--target claude` downgrade to `Info` is documented as a
      note, not a second catalog entry.
  - `src/report.rs`: expose the two structural ids (`file.unreadable` = Error,
    `dir.unreadable` = Warning) so the catalog/README can include them (either fold
    them into the same catalog with a `structural: true` marker, or a small second
    list `STRUCTURAL_RULES`). Whichever — the README table and the drift test must
    cover all **26** ids.
  - `src/lib.rs`: re-export the catalog (`RULES` / `all_rule_ids` / `RuleDoc`).
  - `README.md`:
    - Rewrite the **Status** table: SPEC-001…011 all shipped; `lint` enforces the
      full open-spec catalog + `--target claude` + `--sarif` + real-tokenizer
      `body.size`. Remove the "⏳ next (SPEC-006)" / "arrive in SPEC-006" stale prose.
    - Add a **## Rule reference** section: a table of every rule id | severity |
      what it flags | notes, grouped or ordered stably. Include the `--target
      claude` note on `allowed-tools.format` (list → info) and `frontmatter.unknown`
      (recognizes Claude's fields), the `body.*` thresholds (lines > 500,
      ~tokens > 5000, real tokenizer), and the two structural ids.
    - Document the flags in the usage section: `--target claude`, `--sarif`
      (mutually exclusive with `--json`), `--strict`. Regenerate the example
      output block from the **real current binary** (do not hand-edit — run it).
  - `lint-fixtures/` — add fixtures (see below).
  - `tests/cli.rs` (and/or `src/rules.rs` unit tests): the drift + coverage +
    zero-findings tests.
- **New fixtures created:**
  - `lint-fixtures/good/<a-spec-perfect-skill>/SKILL.md` — a fully-compliant skill
    that yields **0 errors / 0 warnings / 0 infos** both with and without
    `--target claude`. (Use `allowed-tools` as a *space-separated string*, a
    directory name matching `name`, a description ≥ 40 chars stating what+when, a
    non-empty body under the thresholds, only recognized fields.) The existing
    `lint-fixtures/good/data-analysis` may already qualify — verify and reuse it if
    so, otherwise add one.
  - Coverage fixtures so that, across all `lint-fixtures/`, **every** engine rule id
    in the catalog is emitted by at least one fixture. Reuse `bad/My-Skill` and
    `warn-only/mismatched-name`; add focused fixtures for any id not yet covered
    (e.g. a `frontmatter.unclosed` skill, a `frontmatter.missing` skill, a
    `body.empty` skill, a metadata-map-with-non-string-value skill, a
    `compatibility.type` skill, a long-body skill for `body.lines`/`body.size`,
    etc.). Mutually-exclusive frontmatter states (missing/unclosed/invalid vs
    present) legitimately need separate fixtures. **Structural** ids
    (`file.unreadable`, `dir.unreadable`) are out of the fixture-coverage assertion
    (they need a non-UTF-8 file / unreadable dir, already covered by `report.rs`
    unit tests) — document that exclusion explicitly in the coverage test.
- **No new dependency. No `--json`/SARIF/exit-code/rule-id changes** (existing ids
  are unchanged; only *added*: none — the catalog just enumerates existing ids).

## Acceptance Criteria

- [ ] `CLAUDE_KEYS` contains the 5 added fields (`when_to_use`, `argument-hint`,
      `agent`, `paths`, `shell`), each with a `// source:` comment. Under
      `--target claude`, a skill using any of them does **not** fire
      `frontmatter.unknown`; without the target it **does**. A genuinely-unknown key
      still fires it under the target.
- [ ] A public rule catalog exists (`RULES` / `all_rule_ids`) enumerating all
      **26** rule ids (24 engine + 2 structural), duplicate-free, re-exported from
      `lib.rs`. A test locks its exact contents (adding/removing/renaming a rule id
      without updating the catalog fails the test — DEC-005 tripwire).
- [ ] **No orphan rule:** every finding any fixture produces has a `rule` present in
      the catalog (guards against an id the catalog forgot).
- [ ] **Full coverage:** every *engine* rule id in the catalog is emitted by at
      least one committed fixture (structural ids explicitly excused, with a comment).
- [ ] **Spec-perfect → zero findings:** a designated compliant fixture yields
      0 errors / 0 warnings / 0 infos, and exit code 0, both with and without
      `--target claude`.
- [ ] **README drift guard:** a test parses the README **Rule reference** table and
      asserts its set of rule ids equals the catalog's set (no documented-but-absent
      id, no emitted-but-undocumented id), and that each documented severity matches
      the catalog's default severity for that id.
- [ ] README **Status** table shows SPEC-001…011 shipped and no longer says
      SPEC-006/body/metadata are "next"; the usage section documents `--target
      claude`, `--sarif`, and `--strict`; the example output block matches real
      current binary output.
- [ ] `cargo test` / `clippy -D warnings` / `fmt --check` green; deterministic; no
      new dependency; the full pre-existing suite still passes unchanged.

## Failing Tests

Written during **design**, BEFORE build. Paths are indicative — the build may place
unit tests in `src/rules.rs`'s `#[cfg(test)]` module and integration/README tests in
`tests/`. What must be asserted:

- **`src/rules.rs` (unit) — `claude_keys_complete`**
  - Build a skill with frontmatter keys `when_to_use`, `argument-hint`, `agent`,
    `paths`, `shell` (all recognized Claude fields) plus a genuinely-unknown
    `not_a_field`. `lint_skill_with_target(skill, Some(Target::Claude))` →
    **no** `frontmatter.unknown` for the 5 Claude fields, **one**
    `frontmatter.unknown` for `not_a_field`.
  - `lint_skill_with_target(skill, None)` (or `lint_skill`) → `frontmatter.unknown`
    fires for **all 6** (the 5 Claude fields + the unknown), since without the
    target they aren't recognized.
- **`src/rules.rs` (unit) — `catalog_is_locked`**
  - `all_rule_ids()` (or `RULES` mapped to ids) equals an explicit expected slice of
    the 26 ids (asserts exact membership + no duplicates). This is the DEC-005
    tripwire — changing a rule id must change this test.
- **`tests/*.rs` — `no_orphan_rule_ids`**
  - Run the binary (or `lint_skill*`) over the whole `lint-fixtures/` tree; collect
    every emitted `rule`; assert each is in the catalog.
- **`tests/*.rs` — `every_engine_rule_has_a_fixture`**
  - Collect every emitted `rule` across `lint-fixtures/` (both with and without
    `--target claude` so `allowed-tools.format` info is seen); assert the set
    ⊇ (catalog engine ids minus the 2 structural ids). Fail listing any uncovered id.
- **`tests/*.rs` — `spec_perfect_skill_is_clean`**
  - `skillport lint lint-fixtures/good/<perfect>` and `... --target claude` both
    print `0 error(s), 0 warning(s), 0 info(s)` and exit `0`.
- **`tests/*.rs` — `readme_rule_table_matches_catalog`**
  - Parse `README.md`'s Rule reference table (rows with a backtick-wrapped rule id
    in the first column + a severity column); assert the id set == catalog id set
    and each row's severity == the catalog default severity. (Parse defensively:
    match ``` `rule.id` ``` tokens within the table region delimited by the
    `## Rule reference` heading and the next `##`.)

## Implementation Context

*Read this section (and the files it points to) before starting the build cycle.*

### The authoritative rule catalog (design-time probe of `src/rules.rs` + `src/report.rs`)

These are the **26** rule ids with their **real** emitted severities and triggers,
read directly from the code at design (do not re-derive — transcribe, then let the
tests confirm). Engine rules (24) live in `src/rules.rs`; structural (2) in
`src/report.rs`.

| rule id | severity | fires when |
|---|---|---|
| `frontmatter.missing` | error | no YAML frontmatter block |
| `frontmatter.unclosed` | error | opening `---` but no closing `---` |
| `frontmatter.invalid` | error | frontmatter is not a valid YAML mapping |
| `frontmatter.unknown` | info | a key isn't recognized (open set; `--target claude` also allows Claude's fields) |
| `name.required` | error | `name` is missing |
| `name.type` | error | `name` is not a string |
| `name.length` | error | `name` not 1–64 characters |
| `name.charset` | error | `name` has chars outside `[a-z0-9-]` (strict ASCII) |
| `name.hyphen-edges` | error | `name` starts or ends with `-` |
| `name.hyphen-consecutive` | error | `name` contains `--` |
| `name.dir-match` | warning | `name` ≠ the skill's directory name |
| `description.required` | error | `description` is missing |
| `description.type` | error | `description` is not a string |
| `description.length` | error | `description` empty or > 1024 chars |
| `description.detail` | info | `description` < 40 chars (state what + when) |
| `compatibility.length` | error | `compatibility` > 500 chars |
| `compatibility.type` | warning | `compatibility` is not a string |
| `metadata.type` | warning | `metadata` is not a key-value map |
| `metadata.values` | info | a `metadata` value is not a string |
| `allowed-tools.format` | warning → **info** under `--target claude` | `allowed-tools` given as a YAML list |
| `allowed-tools.type` | warning | `allowed-tools` is neither a string nor a list |
| `body.empty` | warning | the `SKILL.md` body is blank |
| `body.lines` | warning | body > 500 lines |
| `body.size` | info | body > ~5000 tokens (real cl100k_base tokenizer) |
| `file.unreadable` | error | a `SKILL.md` couldn't be read (e.g. non-UTF-8) — *structural* |
| `dir.unreadable` | warning | a directory in the tree couldn't be read — *structural* |

Recognized frontmatter key sets (for reference; `frontmatter.unknown` logic):
- Open spec (`SPEC_KEYS`, always recognized): `name`, `description`, `license`,
  `compatibility`, `metadata`, `allowed-tools`.
- Claude extension (`CLAUDE_KEYS`, recognized only under `--target claude`) — the
  **13** after this spec: `disable-model-invocation`, `user-invocable`,
  `disallowed-tools`, `model`, `effort`, `context`, `hooks`, `arguments`,
  **`when_to_use`**, **`argument-hint`**, **`agent`**, **`paths`**, **`shell`**.

### The 5 new CLAUDE_KEYS — verified at design (DEC-002)

Source: https://code.claude.com/docs/en/skills, Frontmatter reference table
(re-fetched and verified 2026-07-18). Each is a documented, recognized Claude Code
frontmatter field, and none is in the open `SPEC_KEYS`:

- `when_to_use` — "Additional context for when Claude should invoke the skill…
  Appended to `description` in the skill listing…"
- `argument-hint` — "Hint shown during autocomplete to indicate expected arguments."
- `agent` — "Which subagent type to use when `context: fork` is set."
- `paths` — "Glob patterns that limit when this skill is activated. Accepts a
  comma-separated string or a YAML list."
- `shell` — "Shell to use for `!` command blocks… Accepts `bash` (default) or
  `powershell`."

The full documented Claude frontmatter set is 16 fields; minus the 3 already in the
open spec (`name`, `description`, `allowed-tools`) that is 13 Claude-extension fields
— exactly the 13 `CLAUDE_KEYS` after this spec. This closes the enumeration gap
SPEC-011 verify flagged.

### Decisions that apply

- `DEC-002` — per-platform facts must be verified from primary docs; the 5 new keys
  were re-verified at design, each carrying a `// source:` comment. **Do not** relax
  any open-spec rule for `--target claude` (this spec doesn't touch rule severities,
  only documents them and completes the recognized-field set).
- `DEC-005` — rule ids, `--json` schema, and exit codes are a public contract; a
  breaking change is a MAJOR bump. The catalog + `catalog_is_locked` test make the
  rule-id set an explicit, guarded surface. This spec only *enumerates* existing ids
  — it must not rename or remove any.
- `DEC-003` — severity discipline (no heuristic at error). The README severity
  column must reflect the **real** emitted severities (info for heuristics like
  `body.size`/`description.detail`/`metadata.values`), enforced by the drift test.

### Constraints that apply

- `only-verified-constraints-are-firm` — the 5 keys are doc-verified; nothing new is
  asserted as error/warning.
- `deterministic-stable-output` — the catalog is ordered and stable; README output
  examples are regenerated from the real binary; no nondeterminism introduced.
- `test-before-implementation` — the tests above are the spec; build makes them pass.
- `no-heuristic-error` — unchanged; the reference simply documents the existing
  severities.

### Prior related work

- `SPEC-011` (shipped, PR #11) — `--target claude` + `CLAUDE_KEYS` (8 fields). This
  spec completes that enumeration (→ 13) and documents the whole rule surface.
- `SPEC-006` (shipped, PR #6) — the open-spec catalog (`metadata.*`,
  `allowed-tools.*`, `body.*`, `frontmatter.unknown`).
- `SPEC-007` (shipped, PR #7) — the structural `dir.unreadable` / `file.unreadable`.
- `SPEC-005` / `SPEC-008` — the CLI flags (`--json`/`--strict`, `--sarif`) to document.

### Out of scope (for this spec specifically)

- **No new rules, no severity changes, no rule-id renames.** This is
  documentation + fixtures + a catalog over the *existing* surface (plus the 5
  already-decided Claude keys).
- **No auto-generation of the README from code.** The README stays hand-authored;
  the drift *test* is what keeps it honest. (A generator could be a later spec.)
- **No Cursor/Codex/Vercel targets** (DEC-002 — future specs; `--target` still
  accepts only `claude`).
- **No SARIF/`--json` schema changes.** The `--json` `target` field and SARIF output
  are unchanged.
- The two structural ids are documented + in the catalog but **excused from the
  fixture-coverage assertion** (they require non-UTF-8 / unreadable-dir conditions
  already covered by `report.rs` unit tests).

## Notes for the Implementer

- **Regenerate, don't hand-write, the README example output.** Run
  `./target/debug/skillport lint lint-fixtures/bad` (and a `--target claude` /
  `--json` example) and paste the real output. The current README example happens to
  be accurate, but the point of this spec is that docs are checked against reality.
- **Make the drift test robust, not brittle.** Parse only the `## Rule reference`
  table region; match rule ids as backtick-wrapped tokens in the first column and
  severity words (`error`/`warning`/`info`) in the severity column. Don't assert on
  prose wording. If parsing the README proves too fragile, an acceptable alternative
  is to *generate* the table body from the catalog into a fenced block the test
  compares byte-for-byte — but prefer the parse-and-compare-sets approach first.
- **`allowed-tools.format` has two severities** depending on `--target`. In the
  catalog store its default (`warning`); document the `--target claude` → `info`
  downgrade in the notes column. The drift test compares the *default* severity.
- **Reuse existing fixtures** (`bad/My-Skill` covers ~8 ids; `warn-only`,
  `good/data-analysis`, `good-claude`). Add the minimum new fixtures to reach full
  engine-rule coverage; keep each fixture focused and commented.
- Keep `RuleDoc` and the catalog `pub` and re-exported so a future `audit` (PROJ-002)
  and any docs generator can consume the same source of truth.

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
