---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-004
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
    - DEC-002   # only open-spec-backed rules are firm; these are all open-spec
    - DEC-003   # severity discipline: crisp violations = error; soft = warning/info
    - DEC-005   # stable rule ids = public contract
  constraints:
    - only-verified-constraints-are-firm
    - no-heuristic-error
    - deterministic-stable-output
    - test-before-implementation
  related_specs:
    - SPEC-001  # Skill + FrontmatterStatus (rules read the frontmatter map + status)
    - SPEC-003  # Finding/Severity + the rule_fn seam this implements

value_link: "the crisp-error core of STAGE-002's lint — the identity/description rules that make lint a meaningful CI gate, implemented as the rule_fn Report::from_collection consumes"

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
      notes: "metered subagent; orchestrator fills real tokens_total/duration/estimated_usd at ship from the Agent result"
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-004: open-spec rule engine and identity rules

## Context

First spec of STAGE-002 and the start of skillport actually *validating*. STAGE-001
built the substrate (`Skill`, `Collection`, `Report::from_collection(collection,
rule_fn)`). This spec implements the **rule engine** — the `rule_fn: Fn(&Skill) ->
Vec<Finding>` that seam expects — and the **crisp identity/description rule batch**
from the open-spec catalog: frontmatter presence, `name.*`, `description.*`,
`compatibility.length`. These are the highest-value, error-heavy rules (they gate
CI). The remaining rules (`metadata.*`, `allowed-tools.*`, `body.*`,
`frontmatter.unknown`) are the next spec; the CLI + emitters are after that.

- Parent stage: `STAGE-002` (open-spec rule engine + `lint`), spec 1 of it.
- Reuses: `Skill` + `FrontmatterStatus` (SPEC-001); `Finding`/`Severity` + the
  `rule_fn` seam (SPEC-003).
- Rule catalog (authoritative severities): the table in
  `projects/PROJ-001-skillport-lint/stages/STAGE-002-*.md` (from agentskills.io).
- Reference: `initial_stuff/lint.rs` implements most of these with the right
  severities — port the checks onto the `Finding` type + `FrontmatterStatus`.

## Goal

Implement `rules::lint_skill(skill: &Skill) -> Vec<Finding>` (the open-spec
`rule_fn`), covering frontmatter presence and the `name.*` / `description.*` /
`compatibility.length` rules at the catalog's exact severities, wired so
`Report::from_collection(collection, rules::lint_skill)` produces a real report.

## Inputs

- **Files to read (reuse):** `src/skill.rs` (`Skill`, `Frontmatter`,
  `FrontmatterStatus`, `YamlValue`), `src/report.rs` (`Finding`, `Severity`),
  `src/walk.rs` (`Collection` — for the integration test).
- **Rule catalog + severities:** `stages/STAGE-002-open-spec-rule-engine-and-lint-command.md`.
- **Reference (port, don't copy blindly):** `initial_stuff/lint.rs`.

## Outputs

- **Files created:** `src/rules.rs` (or `src/rules/mod.rs`) — `lint_skill` + the
  rule implementations + tests.
- **Files modified:** `src/lib.rs` — expose `rules` + `lint_skill`.
- **New exports:**
  ```rust
  /// The open-spec rule_fn. Runs every implemented rule over a parsed skill and
  /// returns findings (unordered — the report layer sorts). Open spec only (no
  /// --target yet). This IS the function passed to Report::from_collection.
  pub fn lint_skill(skill: &Skill) -> Vec<Finding>;
  ```
- **Database changes:** none.

## Rules to implement (this spec only — exact ids & severities)

| Rule id | Sev | Check |
|---|---|---|
| `frontmatter.missing` | error | `FrontmatterStatus::Missing` (no frontmatter block) |
| `frontmatter.unclosed` | error | `FrontmatterStatus::Unclosed` (opening fence, no close) |
| `frontmatter.invalid` | error | `FrontmatterStatus::Invalid` (fenced block, bad YAML / non-mapping) |
| `name.required` | error | `name` key present (only when frontmatter is `Present`) |
| `name.type` | error | `name` is a string |
| `name.length` | error | 1–64 chars |
| `name.charset` | error | lowercase letters, digits, hyphens only |
| `name.hyphen-edges` | error | no leading/trailing hyphen |
| `name.hyphen-consecutive` | error | no `--` |
| `name.dir-match` | warning | equals `skill.dir_name` (skip if `dir_name` is None) |
| `description.required` | error | `description` present |
| `description.type` | error | `description` is a string |
| `description.length` | error | 1–1024 chars, non-empty |
| `description.detail` | info | present but too terse to convey *when* to use (soft — **info only**, DEC-003) |
| `compatibility.length` | error | ≤500 chars if present |

**`frontmatter.*` extension (design decision):** the catalog lists only
`frontmatter.missing`. The SPEC-001 parser distinguishes three failure modes
(`Missing`/`Unclosed`/`Invalid`), so this spec surfaces each as its own **stable
error id** (`frontmatter.missing` / `frontmatter.unclosed` / `frontmatter.invalid`)
for precise messages. All three are crisp mechanical facts → error (DEC-003). An
intentional *extension* of the catalog, not a departure. Flag for verify.

**Empty-but-`Present` frontmatter decision (locks signal `spec-pin-edge-cases`):**
when `FrontmatterStatus::Present` but the map lacks `name`/`description`,
`frontmatter.missing` does **NOT** fire (a block *is* present) — `name.required`
and `description.required` fire instead. Rationale: clearer, more actionable
messages than pretending there's no frontmatter. Locked with an explicit test.

## Acceptance Criteria

- [ ] `lint_skill(&Skill) -> Vec<Finding>` exists and is usable as the `rule_fn` in
      `Report::from_collection` (an integration test does exactly that).
- [ ] Every rule above is implemented with the **exact id and severity**; each
      `Finding.rule` is the stable id string.
- [ ] **Frontmatter presence:** `Missing`→`frontmatter.missing`,
      `Unclosed`→`frontmatter.unclosed`, `Invalid`→`frontmatter.invalid` (each
      error). When frontmatter is not `Present`, the `name.*`/`description.*`/
      `compatibility` rules are **skipped** — no spurious findings.
- [ ] **Empty-`Present` → `name.required` + `description.required`, NOT
      `frontmatter.missing`** (locked decision; explicit test).
- [ ] `name.*` fire correctly: missing, non-string, length 0 and 65,
      uppercase/space charset, leading/trailing hyphen, `--`, and `name.dir-match`
      (warning) when `name != dir_name` (not when equal; skipped when `dir_name` None).
- [ ] `description.*`: missing, non-string, empty, >1024 → errors; short-but-present
      → `description.detail` (info); good description → none.
- [ ] `compatibility.length`: >500 → error; ≤500 → none; absent → none.
- [ ] **A valid skill yields zero findings from this batch.** Proven directly and
      via the repo `lint-fixtures/good` fixture through `from_collection` (zero **errors**).
- [ ] **No heuristic is error-level** (DEC-003): only `description.detail` (info)
      and `name.dir-match` (warning) are non-error; all else crisp = error.
- [ ] Deterministic: `lint_skill` returns findings the report layer sorts; no
      `HashMap`-iteration-dependent observable output.

## Failing Tests

Written now (design). Location: `#[cfg(test)] mod tests` in `src/rules.rs`. Build
`Skill`s in-memory (a small helper making a `Skill` from given frontmatter map +
`FrontmatterStatus` + `dir_name` keeps tests terse). Assert on the set of
`(rule, severity)` pairs produced.

- **`src/rules.rs` (mod tests)**
  - `"frontmatter Missing → frontmatter.missing error; no name/desc rules"`.
  - `"frontmatter Unclosed → frontmatter.unclosed"` / `"Invalid → frontmatter.invalid"`.
  - `"empty Present frontmatter → name.required + description.required, NOT frontmatter.missing"`.
  - `"name.required when absent"`, `"name.type when non-string"`.
  - `"name.length 0 → error"`, `"65 → error"`, `"64 ok"`.
  - `"name.charset uppercase → error"`, `"space → error"`, `"lowercase-digits-hyphen ok"`.
  - `"name.hyphen-edges leading/trailing → error"`, `"name.hyphen-consecutive -- → error"`.
  - `"name.dir-match mismatch → warning"`, `"match → none"`, `"dir_name None → skipped"`.
  - `"description.required/type/empty/too-long → errors"`.
  - `"description.detail short → info"`, `"good description → no detail"`.
  - `"compatibility.length >500 → error"`, `"≤500 → none"`, `"absent → none"`.
  - `"valid skill → zero findings"`.
  - `"no error-level heuristic: dir-match is warning, detail is info"`.
- **integration / fixture-backed**
  - `"from_collection(lint_skill) over lint-fixtures/good → zero errors"` —
    `walk("lint-fixtures/good")` → `Report::from_collection(&c, rules::lint_skill)`
    → `summary.errors == 0`.

## Implementation Context

### Decisions that apply

- `DEC-002` — every rule here is open-spec-backed → firm severities justified. No
  per-platform behavior in this spec.
- `DEC-003` — crisp violations = **error**; `name.dir-match` = **warning** (spec
  "should"); `description.detail` = **info** (soft). No heuristic at error level.
- `DEC-005` — rule id strings are a stable public contract, the three
  `frontmatter.*` ids included.

### Constraints that apply

- `only-verified-constraints-are-firm` — open-spec rules → firm.
- `no-heuristic-error` — `description.detail`/`name.dir-match` are the only
  non-error rules here; keep them info/warning.
- `deterministic-stable-output` — return findings; the report layer orders them.
- `test-before-implementation` — the Failing Tests are the contract.

### Prior related work

- `SPEC-001` — `Skill { frontmatter: IndexMap<String, YamlValue>, dir_name,
  frontmatter_status, .. }`. Read `name`/`description`/`compatibility` from
  `frontmatter`; use the YAML value's string accessors for the type checks.
- `SPEC-003` — `Finding { rule, severity, message, path, field, line }`,
  `Severity`, `Report::from_collection`. `lint_skill` returns `Vec<Finding>`; set
  `path` from the skill's path, `field` to the frontmatter key where apt.
- `initial_stuff/lint.rs` — reference for the name charset/hyphen logic and
  description thresholds. Port; adapt to `Finding` + `FrontmatterStatus` (the
  prototype keyed off "frontmatter map empty").

### The `description.detail` threshold

Soft/tunable. The prototype used `< 40` chars → info. Keep one simple threshold
(document it in a comment); it is **info**, so a false positive is harmless. Never
make it error/warning.

### Out of scope (for this spec specifically)

- The rest of the catalog: `metadata.type`/`metadata.values`,
  `allowed-tools.format`, `body.empty`/`body.lines`/`body.size`,
  `frontmatter.unknown` — **next spec (SPEC-005)**.
- The `lint` CLI, arg parsing, human/`--json`/`--sarif` emitters, `--strict` — later
  STAGE-002/003 specs. This spec is the rule function + its tests.
- `--target` widening, tokenizer, `key.duplicate` — later specs.
- Don't change `report.rs`/`walk.rs`/`skill.rs` beyond re-export needs (additive).

## Notes for the Implementer

- **No new dependency.** Pure std + existing crate types.
- **Structure:** a small helper per rule (or per group) + a top-level `lint_skill`
  that concatenates group findings. Match the readable style of `initial_stuff/lint.rs`.
- **Skip discipline:** if `frontmatter_status != Present`, emit exactly the one
  `frontmatter.*` finding and return — don't run field rules against an empty map
  (prevents `name.required` firing *on top of* `frontmatter.missing`).
- **charset:** lowercase letters, digits, hyphen only — reject uppercase even
  though `char::is_alphanumeric` accepts it (mirror the prototype).
- **`field`/`line`:** set `field` to the key name where sensible; `line` may be
  `None` (best-effort; precise line computation is out of scope here).
- Keep `message`s clear and actionable; emitters (later) render them.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-004-rules`
- **PR (if applicable):** none (build cycle only; no PR/merge per instructions)
- **All acceptance criteria met?** yes
- **New decisions emitted:**
  - none
- **Deviations from spec:**
  - None in behavior. One deliberate interpretation: `compatibility.type` is
    explicitly out of scope for this spec's table (only `compatibility.length`
    is listed), so a non-string `compatibility` value is silently skipped
    rather than flagged — left as a comment in `check_compatibility` for
    SPEC-005 to pick up alongside the rest of the catalog.
- **Follow-up work identified:**
  - SPEC-005 covers `metadata.*`, `allowed-tools.*`, `body.*`,
    `frontmatter.unknown`, and could also add `compatibility.type` if the
    catalog wants it.

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — Nothing major; the two locked design decisions (frontmatter-status
   short-circuit, empty-Present → field-required) were stated precisely enough
   to implement directly and write a test against.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. `no-heuristic-error`, `deterministic-stable-output`, and DEC-002/003
   covered every judgment call (severities, threshold placement).

3. **If you did this task again, what would you do differently?**
   — Nothing; porting `initial_stuff/lint.rs` rule-by-rule onto `Finding` +
   `FrontmatterStatus` was straightforward and the spec's Failing Tests list
   mapped 1:1 onto `#[test]` functions.

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
