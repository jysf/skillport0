---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-002                     # stable, zero-padded, continuous across the repo
  status: proposed                  # proposed | active | shipped | cancelled | on_hold
  priority: high                    # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-001                      # parent project
repo:
  id: skillport

created_at: 2026-07-17
shipped_at: null

value_contribution:
  advances: "The 'genuinely useful lint' half of the project thesis — the crisp, citable validator teams put in CI."
  delivers:
    - "the `lint` command over a file, folder, or tree"
    - "the open-spec rule catalog with correct severities"
    - "human + --json output and CI exit codes (+ --strict)"
  explicitly_does_not:
    - "verify or encode any per-platform constraint (STAGE-003)"
    - "emit SARIF or ship a GitHub Action (STAGE-003)"
    - "use a real tokenizer for body.size (STAGE-003)"
---

# STAGE-002: Open-spec rule engine + `lint` command

## What This Stage Is

The first user-facing capability: a `lint` command that runs the open-spec rule
catalog over a single skill, a skill folder, or a whole tree, and reports
findings at three severities with correct CI exit codes. It plugs the rule engine
into STAGE-001's model, walker, and sectioned report. This is table stakes (it
overlaps the official `skills-ref validate`) — implement it correctly and move
the differentiation into STAGE-003 and PROJ-002.

## Why Now

The substrate (STAGE-001) is inert without rules and a command. This stage makes
skillport *do something* and is the credible entry point for adoption. It must
land before the per-platform/DX polish (STAGE-003), which widens and dresses up a
working validator rather than creating one.

## Success Criteria

- `lint <path>` works for a single skill, a folder, and a tree in one pass.
- Every rule in the catalog below is implemented with the **exact seeded
  severity** and a **stable rule id**.
- Human-readable output and `--json` (stable schema, DEC-005).
- Exit codes: non-zero on any **error**; non-zero on any **warning** under
  `--strict`; zero otherwise.
- A malformed skill in a bulk run is reported as a per-file finding and the run
  continues (never aborts — DEC-005).
- Results are path-sorted and deterministic.
- No heuristic/soft rule is error-level (DEC-003).

## Scope

### In scope
- The rule engine + the catalog below (open-spec layer only).
- `lint` subcommand (clap): single skill / folder / tree.
- Human + `--json` emitters over STAGE-001's sectioned report.
- Exit-code logic + `--strict`.

### Explicitly out of scope
- `--target` recognized-field widening beyond the open `SPEC_KEYS` set — the
  `frontmatter.unknown` rule ships here against the **open** field set; the
  platform-specific widening is STAGE-003 (DEC-002).
- SARIF output, GitHub Action (STAGE-003).
- Real tokenizer — `body.size` may ship here as a **placeholder/deferred** check
  or be deferred wholesale to STAGE-003 where the tokenizer lands (design call).

## Open-spec rule catalog (implement exactly; source: agentskills.io)

Severity: **error** = spec violation (gates CI); **warning** =
recommended/likely-wrong; **info** = advisory. Per DEC-002 only these
open-spec-backed rules are firm; per DEC-003 nothing heuristic is error-level.

| Rule id | Sev | Check |
|---|---|---|
| `frontmatter.missing` | error | frontmatter block present |
| `name.required` / `name.type` | error | present; is a string |
| `name.length` | error | 1–64 chars |
| `name.charset` | error | lowercase letters, digits, hyphens only |
| `name.hyphen-edges` | error | no leading/trailing hyphen |
| `name.hyphen-consecutive` | error | no `--` |
| `name.dir-match` | warning | equals parent directory name |
| `description.required` / `description.type` | error | present; is a string |
| `description.length` | error | 1–1024 chars, non-empty |
| `description.detail` | info | too terse to convey *when* to use (soft; tune) |
| `compatibility.length` | error | ≤500 chars if present |
| `metadata.type` | warning | is a key→value map |
| `metadata.values` | info | values are strings (spec is string→string) |
| `allowed-tools.format` | warning* | space-separated string, not a list (*info where a platform is confirmed to accept a list — that downgrade is STAGE-003 / DEC-002) |
| `body.empty` | warning | body non-empty |
| `body.lines` | warning | ≤500 lines recommended |
| `body.size` | warning | ~<5000 tokens recommended (real tokenizer lands STAGE-003; info-level per the answered Frame question) |
| `frontmatter.unknown` | info | key recognized against the open spec field set (widened per `--target` in STAGE-003) |

> Two severity seams that STAGE-003 / DEC-002 govern: `allowed-tools.format` is
> `warning` for the open target and downgrades to `info` only where a platform is
> *confirmed* to accept a list; `frontmatter.unknown` runs against the open
> `SPEC_KEYS` here and widens per verified `--target`.

## Spec Backlog

> Proposed decomposition — the Design cycle turns these into specs via
> `just new-spec "<title>" STAGE-002`. Not yet scaffolded.

- [ ] (not yet written) — Rule engine skeleton: rule registration, iterate rules
  over a `Skill`, collect findings with stable ids.
- [ ] (not yet written) — `name.*`, `description.*`, `compatibility.length` rules.
- [ ] (not yet written) — `metadata.*`, `allowed-tools.format`, `frontmatter.*`
  (against open field set), `body.*` rules.
- [ ] (not yet written) — `lint` command (clap) wiring walker → engine → report,
  single/folder/tree.
- [ ] (not yet written) — Human + `--json` emitters, exit codes, `--strict`.
- [ ] (not yet written) — `key.duplicate` rule *(follow-up from SPEC-001)* — the
  parser lets a duplicate frontmatter key take last-write-wins; flag duplicates
  (warning) so they aren't silently dropped. Decide severity against the spec.

**Count:** 0 shipped / 0 active / 6 pending

## Design Notes

- The prototype's `lint.rs` already implements essentially this catalog with the
  right severities — it is the strongest reuse candidate. Port it onto STAGE-001's
  collection-first model + sectioned report (the prototype lints one skill at a
  time; adapt to N-with-sections). Reuse `lint-fixtures/good|bad`.
- Keep the open `SPEC_KEYS` set (`name`, `description`, `license`,
  `compatibility`, `metadata`, `allowed-tools`) here; STAGE-003 adds the verified
  `--target claude` widening.
- Firm constraints: `no-heuristic-error`, `deterministic-stable-output`,
  `only-verified-constraints-are-firm`; DEC-002, DEC-003, DEC-005.

## Dependencies

### Depends on
- STAGE-001 (model, walker, report, stable-id + severity types).

### Enables
- STAGE-003 (widens `frontmatter.unknown`/`allowed-tools.format`, adds emitters/DX).
- PROJ-002 (audit reuses the same finding model).

## Stage-Level Reflection

*Filled in when status moves to shipped.*

- **Did we deliver the outcome in "What This Stage Is"?** <not yet>
- **How many specs did it actually take?** <not yet>
- **What changed between starting and shipping?** <not yet>
- **Lessons that should update AGENTS.md, templates, or constraints?** <not yet>
- **Signals dispositioned at this close?** <not yet>
- **Should any spec-level reflections be promoted to stage-level lessons?** <not yet>
