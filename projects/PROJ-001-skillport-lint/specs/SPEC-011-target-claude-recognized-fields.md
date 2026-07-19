---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-011
  type: story                      # epic | story | task | bug | chore
  cycle: design                    # frame | design | build | verify | ship
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
  implementer: claude-sonnet-4-6  # build runs as a Sonnet subagent (cost); updated with the real model
  created_at: 2026-07-18

references:
  decisions:
    - DEC-002   # THE decision — per-platform behavior verified from primary docs; this is the first verified --target
    - DEC-003   # allowed-tools.format downgrades to info (Claude accepts a list)
    - DEC-005   # stable; the --json target field
  constraints:
    - only-verified-constraints-are-firm
    - no-heuristic-error
    - deterministic-stable-output
  related_specs:
    - SPEC-003  # rule_fn seam (CLI closes over target) + the --json target slot
    - SPEC-004  # check_unknown_fields / SPEC_KEYS
    - SPEC-006  # check_allowed_tools (the format downgrade seam)
    - SPEC-005  # the CLI (adds --target)

value_link: "the differentiated per-platform layer — the FIRST --target verified from primary docs (DEC-002): Claude Code's recognized SKILL.md fields, so a real Claude extension isn't flagged unknown and allowed-tools-as-a-list is accepted"

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
      notes: "main-loop, not separately metered (design cycle); includes primary-doc research (WebFetch code.claude.com) per DEC-002"
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-011: target claude recognized fields

## Context

The differentiated core (per DEC-001/002): per-platform awareness. This adds
`--target claude` — the **first** platform layer, and the first place skillport
encodes per-platform behavior, so it is done under strict DEC-002 discipline:
**every Claude-specific fact is verified from Claude's primary docs and carries a
source comment**; nothing is guessed. The design cycle already did the
verification (see below).

Two seams from earlier specs make this additive: `check_unknown_fields`
(`frontmatter.unknown` vs `SPEC_KEYS`) and `check_allowed_tools` (the
`allowed-tools.format` list warning). Both already have comments noting the
`--target` widening lands in STAGE-003. The rule engine is consumed via the
`rule_fn` seam (SPEC-003), so the CLI closes over the target — no reshaping.

## Verified from Claude's primary docs (DEC-002 — source of every fact below)

**Source: https://code.claude.com/docs/en/skills** ("Extend Claude with skills",
Frontmatter reference). (`docs.claude.com/en/docs/claude-code/skills` 301-redirects
here.) Verified 2026-07-18.

- Claude Code's recognized `SKILL.md` frontmatter fields are: `name`,
  `description`, `disable-model-invocation`, `user-invocable`, `allowed-tools`,
  `disallowed-tools`, `model`, `effort`, `context`, `hooks`, and `arguments`.
- **`allowed-tools`** — *quote:* "Accepts a space- or comma-separated string, or a
  **YAML list**." → so under `--target claude`, `allowed-tools.format` (which warns
  a list isn't a space-separated string for the open target) **downgrades to info**.
- `name` is optional for Claude (defaults to the directory name), and `description`
  has a 1,536-char *listing truncation*. **We do NOT change** `name.required`
  (error) or `description.length` (1024) for `--target claude` — the open spec stays
  authoritative (DEC-002); `--target` only *widens recognized fields* and applies
  the *confirmed* list downgrade. (Note the 1,536 truncation in a comment only.)

## Goal

Add `skillport lint <PATH> --target claude`: recognize Claude Code's documented
frontmatter fields (so `frontmatter.unknown` doesn't fire on them) and downgrade
`allowed-tools.format` (list) to **info** (Claude accepts a list) — every change
cited from primary docs — and label the `--json` output `target: "claude"`.

## Inputs

- **Files to read (extend):** `src/rules.rs` (`SPEC_KEYS`, `check_unknown_fields`,
  `check_allowed_tools`, `lint_skill`), `src/main.rs` (add `--target`; the `Lint`
  command; the `emit::json(&report, None)` call → pass the target), `src/emit.rs`
  (`json` already takes `target: Option<&str>`), `src/report.rs` (unchanged).
- **Primary docs (already verified — re-check if editing behavior):**
  https://code.claude.com/docs/en/skills

## Outputs

- **Files modified:**
  - `src/rules.rs` — a `pub enum Target { Claude }`; a `CLAUDE_KEYS` const (the
    Claude-only fields, each on a line with a `// source: code.claude.com/docs/en/skills`
    comment); thread the target into the rule engine. Keep `pub fn lint_skill(skill)
    -> Vec<Finding>` (open-spec, = target None) **unchanged in behavior**, and add
    `pub fn lint_skill_with_target(skill, target: Option<Target>) -> Vec<Finding>`
    (or give `check_unknown_fields`/`check_allowed_tools` a `target` param and route
    both `lint_skill` variants through it). `frontmatter.unknown` recognizes
    `SPEC_KEYS ∪ CLAUDE_KEYS` when `target == Some(Claude)`; `allowed-tools.format`
    (list) is `Info` (not `Warning`) when `target == Some(Claude)`, with a message
    noting Claude accepts a list.
  - `src/main.rs` — add `--target <TARGET>` to `Lint` as a clap `ValueEnum` with the
    single variant `claude` (unknown values → clap usage error, exit 2); map it to
    `rules::Target`; call `Report::from_collection(&c, |s| lint_skill_with_target(s,
    target))`; pass `target` label to `emit::json` (so `"target":"claude"`).
  - `src/lib.rs` — re-export `Target` + `lint_skill_with_target`.
- **No new dependency.** SARIF is unchanged (it's a standard format; no `target`
  field). **Database changes:** none.

## Behavior under `--target claude` (exact)

| Rule | Open (no `--target`) | `--target claude` |
|---|---|---|
| `frontmatter.unknown` on `context`/`model`/`disable-model-invocation`/`user-invocable`/`disallowed-tools`/`effort`/`hooks`/`arguments` | **info** (unknown) | **not fired** (recognized) |
| `frontmatter.unknown` on a truly-unknown key (e.g. `random_field`) | info | **still info** (Claude doesn't recognize it either) |
| `allowed-tools.format` when `allowed-tools` is a **list** | **warning** | **info** (Claude accepts a list — cited) |
| `allowed-tools.type` (neither string nor list) | warning | warning (unchanged) |
| `name.required`, `description.length`, all other open-spec rules | firm | **unchanged** (open spec stays authoritative, DEC-002) |
| `--json` `target` field | `null` | `"claude"` |

## Acceptance Criteria

- [ ] `skillport lint <PATH> --target claude` is accepted; `--target <bogus>` is a
      clap usage error (exit 2, stderr, empty stdout). Only `claude` is a valid value.
- [ ] `pub enum Target { Claude }` + `CLAUDE_KEYS` exist; every Claude field carries
      a `// source: code.claude.com/docs/en/skills` comment (DEC-002).
- [ ] With `--target claude`, `frontmatter.unknown` does **not** fire on any of
      Claude's documented fields, but **still fires** on a genuinely unknown key.
- [ ] With `--target claude`, an `allowed-tools` **list** yields `allowed-tools.format`
      at **info** (not warning); the message notes Claude accepts a list. Without the
      target it stays **warning**.
- [ ] `allowed-tools.type` and every open-spec rule (name/description/etc.) are
      **unchanged** by `--target` (DEC-002 — no relaxing open-spec requirements).
- [ ] The default `lint_skill(skill)` (no target) behaves exactly as before — all
      existing tests pass unchanged.
- [ ] `--json` shows `"target":"claude"` under `--target claude`, `null` otherwise.
      SARIF unchanged.
- [ ] Deterministic; no new dependency; `cargo test`/`clippy`/`fmt` green; the good
      fixture stays 0/0/0 with and without `--target claude`.

## Failing Tests

Written now (design).

- **`src/rules.rs` (mod tests)** — construct in-memory `Skill`s:
  - `"target claude: a Claude field (context) does NOT trigger frontmatter.unknown"`
    vs `"no target: context DOES trigger frontmatter.unknown (info)"`.
  - `"target claude: a truly-unknown key STILL triggers frontmatter.unknown"`.
  - `"target claude: allowed-tools list → allowed-tools.format INFO"` vs
    `"no target: allowed-tools list → allowed-tools.format WARNING"`.
  - `"target claude: allowed-tools.type (a number) still WARNING"`.
  - `"target claude does NOT relax name.required / description.length"` (still errors).
  - `"lint_skill (no target) is unchanged"` (a Claude field → info; a list → warning).
- **`tests/cli.rs`** (integration):
  - `"lint --target claude on a Claude-fields fixture → 0 errors, no frontmatter.unknown for those fields"`.
  - `"lint --target bogus → exit 2"`.
  - `"lint --target claude --json → \"target\":\"claude\""`; `"without → target null"`.
- **fixture:** add `lint-fixtures/good-claude/<name>/SKILL.md` — a valid skill using
  a couple of Claude fields (e.g. `allowed-tools:` as a YAML list, `context: fork`)
  that is **clean under `--target claude`** (0/0/0) but would emit
  `frontmatter.unknown`(info) + `allowed-tools.format`(warning) **without** it.

## Implementation Context

### Decisions that apply

- `DEC-002` — **the** governing decision. Only facts verified from
  code.claude.com are encoded; each carries a source comment. Unverified platforms
  (Cursor/Codex/Vercel) are **not** added — `--target` accepts only `claude`.
- `DEC-003` — `allowed-tools.format` at info under `--target claude` is the
  "confirmed a platform accepts a list" downgrade, not a heuristic.
- `DEC-005` — the `--json` `target` field is part of the stable schema (already a
  nullable slot); deterministic output.

### Constraints that apply

- `only-verified-constraints-are-firm` — do not encode any Claude behavior not on
  the cited docs page; if a fact is unclear, leave the open-spec behavior.
- `no-heuristic-error`, `deterministic-stable-output`.

### Prior related work

- `SPEC-003` — `Report::from_collection(collection, rule_fn)`; the CLI passes
  `|s| lint_skill_with_target(s, target)`. `emit::json(&report, target)` already
  has the `target` param (SPEC-005 passes `None`).
- `SPEC-004` — `check_unknown_fields` + `SPEC_KEYS`. `SPEC-006` — `check_allowed_tools`.

### Out of scope (for this spec specifically)

- Cursor / Codex / Vercel targets — **not verified**, so not added (DEC-002). A
  later spec can verify each from its primary docs and add a `Target` variant.
- Encoding Claude's `name`-optional / 1,536-char-truncation behavior as rules — we
  keep the open-spec rules; only recognized-fields + the list downgrade change.
- Any new rule, the README table, SARIF `target` field, `audit` — out of scope.

## Notes for the Implementer

- **Thread the target minimally:** the cleanest shape is to give
  `check_unknown_fields` and `check_allowed_tools` a `target: Option<Target>`
  parameter, and have both `lint_skill(skill)` (calls with `None`) and
  `lint_skill_with_target(skill, target)` route through the same body. Keep
  `lint_skill`'s public signature so existing tests/example compile unchanged.
- **`CLAUDE_KEYS`:** list only the Claude-*specific* fields (those not already in
  `SPEC_KEYS`): `disable-model-invocation`, `user-invocable`, `disallowed-tools`,
  `model`, `effort`, `context`, `hooks`, `arguments`. One `// source:` comment on the
  const (or per line) citing code.claude.com/docs/en/skills.
- **allowed-tools.format message under claude:** e.g. `"'allowed-tools' is a list;
  the open spec expects a space-separated string, but Claude Code accepts a list
  (source: code.claude.com/docs/en/skills)"` at info.
- **clap `--target`:** `#[arg(long, value_enum)]` with `enum TargetArg { Claude }`;
  map to `rules::Target`. Unknown value → clap's own usage error (exit 2).
- **`--json target`:** thread `Some("claude")` into `emit::json`. Confirm `--json`
  without `--target` still emits `"target":null` (regression).
- Keep human + SARIF output otherwise unchanged; only the findings + the `--json`
  target label differ.

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
