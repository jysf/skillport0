---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-001                     # stable, zero-padded, continuous across the repo
  status: shipped                   # proposed | active | shipped | cancelled | on_hold
  priority: high                    # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-001                      # parent project
repo:
  id: skillport

created_at: 2026-07-17
shipped_at: 2026-07-18

# What part of the project's value thesis this stage advances.
# If you can't articulate value_contribution, the stage may be
# infrastructure-only — acceptable but flag it.
value_contribution:
  advances: "The 'built for PROJ-002 reuse' half of the project thesis — the collection-first substrate every later stage and the audit build on."
  delivers:
    - "a parsed, canonical model of any SKILL.md (even malformed ones)"
    - "a walk of a path into a collection of skills"
    - "a sectioned N-skill report + finding model with stable ids"
  explicitly_does_not:
    - "implement any rules (STAGE-002)"
    - "expose a CLI command (STAGE-002)"
    - "do per-platform / --target work (STAGE-003)"
---

# STAGE-001: Core substrate

## What This Stage Is

The shared foundation the rest of skillport stands on: a **tolerant, lossless,
order-preserving** `SKILL.md` parser; a canonical `Skill` model; a **tree-walker
that returns a collection** of skills; and a **finding + report model that
already takes N skills with sections and stable ids**. When this ships, the rule
engine (STAGE-002) and the PROJ-002 audit both plug into the same substrate
instead of re-deciding it. This is deliberately designed here as the reuse base
(DEC-004) — it is *not* a single-file linter with a folder loop.

## Why Now

Everything in PROJ-001 and PROJ-002 depends on it. Built single-file-first, it
would force a rewrite when the audit arrives (DEC-004). The parser's tolerance
and losslessness are load-bearing: bulk runs must survive malformed skills
(DEC-005), and future normalization/round-trip work needs nothing dropped on
parse.

## Success Criteria

- Any `SKILL.md` parses into a canonical `Skill` (frontmatter + body), preserving
  frontmatter key order and losing nothing.
- Tolerant of real-world messiness: BOM, leading blank lines, missing frontmatter,
  unclosed frontmatter, CRLF — each handled gracefully (surfaced as a finding,
  never a panic).
- Walking a path yields a **collection**: a single file → 1 skill; a folder / tree
  → all `SKILL.md` under it, skipping `.git`, `node_modules`, `target`.
- The report model represents **N skills, each with a section of findings**; a
  finding carries `{ severity, rule (stable id), message, location }`.
- Output ordering is deterministic (sorted by path) — the substrate guarantees it
  even before any emitter exists (DEC-005).

## Scope

### In scope
- `parse`: split YAML frontmatter from Markdown body; tolerant of the edge cases above.
- `Skill` model: order-preserving frontmatter map + raw body + source path.
- `walk`: path → `Vec<Skill-or-parse-error>` collection, with directory skips.
- `Finding` + `Severity` (error/warning/info) + a sectioned report type.
- Stable rule-id convention (the ids are a public contract per DEC-005).

### Explicitly out of scope
- Any rule logic or the `lint` command (STAGE-002).
- Human/JSON/SARIF emitters (STAGE-002/003 — the report *model* lives here; the
  *rendering* comes later).
- `--target` / per-platform recognized fields (STAGE-003).
- Tokenizer (STAGE-003, used by `body.size`).

## Spec Backlog

> Proposed decomposition — the Design cycle turns these into specs via
> `just new-spec "<title>" STAGE-001`. Not yet scaffolded.

- [x] SPEC-001 (shipped 2026-07-18, PR #1) — Tolerant, lossless, order-preserving
  `SKILL.md` parser + canonical `Skill` model (BOM / leading blanks / missing /
  unclosed / invalid frontmatter / CRLF; total `parse`, never aborts). Model
  folded in. Emitted DEC-007 (`serde_yaml_ng` + `indexmap`).
- [x] SPEC-002 (shipped 2026-07-18, PR #2) — Collection tree-walker: `walk(root) ->
  Collection` (skips `.git`/`node_modules`/`target`; single file & tree both yield
  a collection; unreadable/non-UTF-8 file → `Unreadable` item, never aborts;
  path-sorted). Reuses SPEC-001's `parse`. `tempfile` dev-dep only.
- [x] SPEC-003 (shipped 2026-07-18, PR #3) — Finding + `Severity` + sectioned
  N-skill `Report` model: stable rule ids, path-sorted sections,
  `Report::from_collection(collection, rule_fn)` (Unreadable → `file.unreadable`
  error; `rule_fn` seam for STAGE-002), `exit_code(strict)`. No rules/heuristics/emitters.

**Count:** 3 shipped / 0 active / 0 pending — **stage backlog complete.**

## Design Notes

- Pick a **current, maintained** YAML crate (the prototype's `serde_yaml` is now
  deprecated; its `=`-pins were a Rust-1.75 artifact — drop them). Adding it is a
  runtime dep → author a DEC in the same pass (`no-new-top-level-deps-without-decision`).
- Order preservation implies an order-preserving map (e.g. an index-map style
  structure) rather than a plain `HashMap`.
- The prototype's `parse.rs` / `skill.rs` are a reasonable reference for the split,
  but the collection-first walker and the sectioned/stable-id report are the parts
  the prototype does *not* fully have — design them here.
- Firm constraints this stage must honor: `collection-first-substrate`,
  `deterministic-stable-output` (see `guidance/constraints.yaml`); DEC-004, DEC-005.

## Dependencies

### Depends on
- None (foundational stage).

### Enables
- STAGE-002 (rule engine plugs into the model + report).
- STAGE-003 (emitters, tokenizer, `--target`).
- PROJ-002 (`audit` reuses walker + model + report).

## Stage-Level Reflection

*Shipped 2026-07-18.*

- **Did we deliver the outcome in "What This Stage Is"?** Yes. The collection-first
  substrate is complete and reusable: tolerant lossless `parse` (SPEC-001), a
  path-sorted `Collection` walker that never aborts (SPEC-002), and the sectioned
  N-skill `Report` + stable-id findings + `exit_code` with the `rule_fn` seam
  (SPEC-003). PROJ-002's audit and STAGE-002's rule engine both plug into this
  without a refactor — the DEC-004 bet held.
- **How many specs did it actually take?** 3 (as planned — the "Skill model" item
  was folded into SPEC-001). 65 tests total; every spec APPROVED first pass.
- **What changed between starting and shipping?** Cost metering: SPEC-001's
  build/verify ran as hand-driven sessions (unmetered → grandfathered); from
  SPEC-002 on, build (Sonnet) and verify (Opus) run as **metered subagents**, so
  real cost is captured and cost-audit passes without grandfathering.
- **Lessons that should update AGENTS.md, templates, or constraints?** None promoted
  to codified yet (all below the N=3 bar). The metered-subagent pipeline is worth
  making the default workflow note — captured in memory; revisit at PROJ-001 close.
- **Signals dispositioned at this close?** All STAGE-001-owned `watch` lessons walked
  (no silent carry):
  - `spec-pin-edge-cases` (N=1) — **kept watch** (below N=3 bar); concrete carry into
    STAGE-002: lock empty-block→`Present`-empty with a test where `frontmatter.missing`
    is designed. `last_touched` bumped.
  - `walk-unreadable-dirs` (N=1) — **kept watch**; carry into STAGE-002's report/rules:
    decide whether a permission-denied subtree becomes a finding. `last_touched` bumped.
  - (`cost-metering-manual-sessions` is a process-debt, dispositioned at PROJECT close, not here.)
- **Should any spec-level reflections be promoted to stage-level lessons?** No — the
  `rule_fn` seam and metered-subagent pipeline are recorded; nothing else recurred
  enough to promote.
