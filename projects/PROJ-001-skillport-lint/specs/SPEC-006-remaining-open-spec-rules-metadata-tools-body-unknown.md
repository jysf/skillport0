---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-006
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
  implementer: claude-sonnet-4-6  # build runs as a Sonnet subagent (cost); updated with the real model
  created_at: 2026-07-18

references:
  decisions:
    - DEC-002   # only open-spec-backed rules are firm; these are all open-spec
    - DEC-003   # severity discipline: crisp = error; recommended = warning; soft = info
    - DEC-005   # stable rule ids = public contract
  constraints:
    - only-verified-constraints-are-firm
    - no-heuristic-error
    - deterministic-stable-output
    - test-before-implementation
  related_specs:
    - SPEC-004  # extends lint_skill (the rule engine) with the rest of the catalog
    - SPEC-005  # the CLI/JSON that renders the new findings (unchanged by this spec)

value_link: "completes the open-spec rule catalog inside lint_skill — after this, `skillport lint` enforces the full open layer (only body.size + --target remain for STAGE-003)"

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
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-006: remaining open-spec rules metadata tools body unknown

## Context

The catalog-completion spec. SPEC-004 implemented the identity/description rules
in `lint_skill`; SPEC-005 shipped the CLI that renders them. This spec adds the
**remaining open-spec rules** — `metadata.*`, `allowed-tools.*`, `body.empty` /
`body.lines`, and `frontmatter.unknown` — plus two carried-over items:
`compatibility.type` (deferred from SPEC-004) and tightening `name.charset` to
ASCII (signal `name-charset-ascii`). After this, `skillport lint` enforces the
**entire open-spec layer**; only `body.size` (needs the real tokenizer) and the
`--target` widening remain, both for STAGE-003.

- Parent stage: `STAGE-002`. Purely additive to `rules::lint_skill` — the CLI
  (`main.rs`/`emit.rs`) is unchanged; new findings flow through automatically.
- Reuses: `Skill`/`Frontmatter`/`YamlValue` (SPEC-001), `Finding`/`Severity`
  (SPEC-003), the existing `lint_skill` structure (SPEC-004).
- Reference: `initial_stuff/lint.rs` implements most of these (metadata/allowed-tools/
  body/unknown-field) with the right severities — port onto the current types.

## Goal

Extend `rules::lint_skill` with `metadata.*`, `allowed-tools.*`, `body.empty`,
`body.lines`, and `frontmatter.unknown` at the catalog's exact severities; add
`compatibility.type`; and tighten `name.charset` to ASCII `[a-z0-9-]`.

## Inputs

- **Files to read (reuse/extend):** `src/rules.rs` (extend `lint_skill`),
  `src/skill.rs` (`Frontmatter`, `YamlValue` accessors: `as_str`/`is_string`/
  `as_mapping`/`is_sequence`, `body`, `keys()`), `src/report.rs` (`Finding`/`Severity`).
- **Rule catalog + severities:** `stages/STAGE-002-*.md`.
- **Reference:** `initial_stuff/lint.rs` (`check_metadata`, `check_allowed_tools`,
  `check_body`, `check_unknown_fields`, `SPEC_KEYS`).

## Outputs

- **Files modified:** `src/rules.rs` — new rule checks wired into `lint_skill` +
  their tests. (No new files, no CLI/emitter changes, no new deps.)
- **No new exports** beyond what's internal to `rules`.
- **Database changes:** none.

## Rules to implement (exact ids & severities)

| Rule id | Sev | Check |
|---|---|---|
| `metadata.type` | warning | if `metadata` present, it is a mapping (key→value) |
| `metadata.values` | info | each `metadata` value is a string (spec is string→string; e.g. unquoted `1.0` → info) |
| `allowed-tools.format` | warning | if `allowed-tools` is a **list**, flag (open spec = space-separated string). (The info-where-a-platform-accepts-a-list downgrade is STAGE-003 via `--target`.) |
| `allowed-tools.type` | warning | if `allowed-tools` is neither a string nor a list (e.g. a number/mapping) |
| `body.empty` | warning | the Markdown body (trimmed) is non-empty |
| `body.lines` | warning | body ≤ 500 lines (recommended) |
| `frontmatter.unknown` | info | each top-level frontmatter key is in the open field set (below); else advisory |
| `compatibility.type` | warning | if `compatibility` present, it is a string (carried from SPEC-004) |

**Open field set (`SPEC_KEYS`) for `frontmatter.unknown`:** `name`, `description`,
`license`, `compatibility`, `metadata`, `allowed-tools`. Any other top-level key →
`frontmatter.unknown` (info) — "not a recognized field; compliant agents ignore
unknown keys". (`--target` widening of this set is STAGE-003.)

**`name.charset` tightening (resolves signal `name-charset-ascii`):** change the
existing rule from `is_alphanumeric() && !is_uppercase()` (which accepts non-ASCII
letters/digits like `café`, Arabic-Indic digits) to strict **ASCII**: allow only
`a`–`z`, `0`–`9`, and `-`. Rationale: the open-spec `name` is a kebab-case
identifier that must map to a directory name and be portable — ASCII is the
conservative, correct reading; tightening only *rejects more*, never produces a
false error on valid ASCII kebab-case. Keep the id `name.charset` and severity
**error**. Update the existing charset test(s) and add a non-ASCII → error case.

**Severity rationale (DEC-003):** `metadata.type` / `allowed-tools.*` /
`compatibility.type` are recommended-practice/likely-wrong → **warning**;
`body.empty` / `body.lines` are recommended → **warning**; `metadata.values` and
`frontmatter.unknown` are advisory → **info**. Nothing here is error-level except
the (already-error) `name.charset`. No heuristic at error level.

**`allowed-tools.*` / `metadata.*` extension note:** the catalog lists
`allowed-tools.format`, `metadata.type`, `metadata.values`. `allowed-tools.type`
(neither string nor list) is a small precise extension mirroring the prototype —
flag for verify, same spirit as SPEC-004's `frontmatter.*` split.

## Acceptance Criteria

- [ ] Every rule above is added to `lint_skill` with the **exact id and severity**;
      ids are stable `&'static str`.
- [ ] Rules only run when frontmatter is `Present` (same skip discipline as
      SPEC-004 — a non-`Present` skill still yields only its one `frontmatter.*`
      finding).
- [ ] `metadata`: non-mapping → `metadata.type` (warning); a mapping with a
      non-string value → `metadata.values` (info) per offending value; absent → none.
- [ ] `allowed-tools`: a list → `allowed-tools.format` (warning); a string → none;
      neither → `allowed-tools.type` (warning); absent → none.
- [ ] `body`: empty/whitespace-only → `body.empty` (warning); >500 lines →
      `body.lines` (warning); a normal body → neither.
- [ ] `frontmatter.unknown`: a top-level key outside the open field set → info (one
      per unknown key); a skill using only known fields → none.
- [ ] `compatibility`: non-string → `compatibility.type` (warning); string ≤500 →
      none; (>500 still → `compatibility.length` error from SPEC-004, unchanged).
- [ ] `name.charset` now rejects non-ASCII: a `name` with `café` or a non-ASCII
      digit → `name.charset` (error); a valid ASCII kebab name → none.
- [ ] **The good fixture still yields zero findings** (`lint-fixtures/good` through
      `Report::from_collection` → `summary.errors == 0`, and no new
      warnings/infos on it — verify its frontmatter uses only known fields, a
      string-valued metadata map, no `allowed-tools`, a non-empty body).
- [ ] No heuristic is error-level; determinism preserved (findings returned
      unordered, report layer sorts).
- [ ] `skillport lint lint-fixtures/bad` now additionally reports
      `allowed-tools.format` (warning), `metadata.values` (info), and
      `frontmatter.unknown` (info, for `random_field`) — an end-to-end check that
      the new rules flow through the unchanged CLI.

## Failing Tests

Written now (design). Location: `#[cfg(test)] mod tests` in `src/rules.rs`
(extend the existing module). Build `Skill`s in-memory with the existing test
helper; assert on the `(rule, severity)` set.

- **`src/rules.rs` (mod tests)**
  - `"metadata non-mapping → metadata.type warning"`; `"metadata string value ok"`;
    `"metadata non-string value → metadata.values info"`; `"metadata absent → none"`.
  - `"allowed-tools list → allowed-tools.format warning"`;
    `"allowed-tools string → none"`;
    `"allowed-tools number → allowed-tools.type warning"`; `"absent → none"`.
  - `"body empty → body.empty warning"`; `"body >500 lines → body.lines warning"`;
    `"normal body → neither"`.
  - `"unknown top-level key → frontmatter.unknown info"`;
    `"only known fields → no unknown finding"`.
  - `"compatibility non-string → compatibility.type warning"`;
    `"compatibility string ≤500 → none"`.
  - `"name.charset rejects non-ASCII (café) → error"`;
    `"name.charset rejects non-ASCII digit → error"`;
    `"ascii kebab name → no charset finding"`.
  - `"valid skill still yields zero findings"` (re-assert with the new rules active).
- **integration / fixture-backed**
  - `"lint-fixtures/good → zero findings (errors, warnings, infos all 0)"` via
    `walk` + `from_collection(lint_skill)`.
  - Optionally extend `tests/cli.rs`: `lint lint-fixtures/bad` stdout now contains
    `allowed-tools.format` and `frontmatter.unknown` (proves flow through the CLI).

## Implementation Context

### Decisions that apply

- `DEC-002` — all rules here are open-spec-backed → firm severities justified;
  no per-platform behavior. `name.charset` ASCII is the conservative reading of
  the open spec's kebab-case identifier (tightening only rejects more).
- `DEC-003` — warnings for recommended/likely-wrong; info for advisory; the only
  error touched is the already-error `name.charset`. No heuristic at error level.
- `DEC-005` — new rule ids (`metadata.type`, `metadata.values`,
  `allowed-tools.format`, `allowed-tools.type`, `body.empty`, `body.lines`,
  `frontmatter.unknown`, `compatibility.type`) join the stable public contract.

### Constraints that apply

- `only-verified-constraints-are-firm` — open-spec rules → firm.
- `no-heuristic-error` — keep the new rules at warning/info; do not make body-size
  or metadata judgments error-level.
- `deterministic-stable-output` — `lint_skill` returns findings unordered; the
  report layer sorts. Iterate the frontmatter via its order-preserving structure,
  not a `HashMap`.
- `test-before-implementation`.

### Prior related work

- `SPEC-004` (shipped) — `lint_skill` structure + the identity/description rules,
  and the `frontmatter_status != Present → return` skip discipline. Extend the
  same function; keep the skip behavior.
- `initial_stuff/lint.rs` — `check_metadata` (mapping + string values),
  `check_allowed_tools` (list vs string), `check_body` (empty + line count),
  `check_unknown_fields` + `SPEC_KEYS`. Port these; adapt to `Finding`.

### `body.size` is explicitly deferred

Do **not** implement `body.size` (the ~5000-token check) here — it needs the real
tokenizer, which lands in STAGE-003 at info severity (per the answered Frame
question). `body.empty` and `body.lines` are line/emptiness checks and belong here.

### Out of scope (for this spec specifically)

- `body.size` / the tokenizer — STAGE-003.
- `--target` widening of the recognized-field set (and the `allowed-tools.format`
  → info downgrade where a platform accepts a list) — STAGE-003.
- `--sarif` — STAGE-003. `key.duplicate` — a separate later spec.
- Any CLI/emitter change — none needed; the new findings render through SPEC-005's
  code unchanged. Do not touch `main.rs`/`emit.rs` except (optionally) a test in
  `tests/cli.rs`.

## Notes for the Implementer

- **No new dependency.** Extend `rules.rs` only.
- **Keep the skip discipline:** the new field rules run only when
  `frontmatter_status == Present`, same as SPEC-004.
- **`metadata.values`:** emit one info per non-string value; put the offending key
  in the `field` (e.g. `metadata.version`) if convenient.
- **`frontmatter.unknown`:** compare each top-level key against `SPEC_KEYS`; the
  skill's frontmatter is order-preserving so iterate it directly for determinism.
- **`name.charset`:** the minimal change is the predicate — replace
  `c.is_alphanumeric() && !c.is_uppercase()` with `c.is_ascii_lowercase() ||
  c.is_ascii_digit() || c == '-'`. Update the existing charset test's expectations
  and add the non-ASCII cases. Everything else about `name.*` is unchanged.
- Re-run `cargo run --example lint_demo -- lint-fixtures/bad` and
  `skillport lint lint-fixtures/bad` by eye — the bad fixture should now surface
  the `allowed-tools`/`metadata`/unknown-field findings too.

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
