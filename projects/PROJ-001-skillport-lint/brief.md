---
project:
  id: PROJ-001
  status: active
  activity: design
  priority: high
  target_ship: null

repo:
  id: skillport

created_at: 2026-07-17
shipped_at: null

value:
  thesis: >
    A genuinely useful `lint` command — built on a collection-first substrate
    shaped for the PROJ-002 audit to reuse — is the credible, table-stakes entry
    point that earns skillport a place in skill-authoring CI, from which the
    differentiated audit can land as an additive layer.
  beneficiaries:
    - skill authors (fast, citable conformance feedback per file)
    - teams/CI maintaining a skill library (bulk lint + exit codes)
    - PROJ-002 (reuses parser, model, walker, sectioned report)
  success_signals:
    - "`lint` runs over a single skill and a whole tree, emitting human + JSON + SARIF"
    - "correct CI exit codes (non-zero on error; on warning under --strict)"
    - "a spec-perfect skill yields zero findings (proven by a test)"
    - "every rule has a stable id and per-rule tests + good/bad fixtures"
    - "the substrate (collection walker, sectioned report, stable ids) is ready for audit"
  risks_to_thesis:
    - "open-spec checks overlap with `skills-ref validate` — differentiation must come from the per-platform layer + bulk/CI ergonomics, not the open layer alone"
    - "encoding an unverified per-platform constraint as an error would destroy trust (mitigated by DEC-002)"
    - "over-building toward audit before lint is useful (mitigated by shipping the crisp validator first)"
---

# PROJ-001: Foundation + lean `lint`

## What This Project Is

The first wave of skillport: a fast, deterministic `lint` command that validates
agent `SKILL.md` files against the open Agent Skills spec, over a single file, a
skill folder, or a whole tree — with three severities, human + `--json` +
`--sarif` output, and CI-friendly exit codes. Critically, `lint` is built on a
**collection-first substrate** (tolerant lossless parser, canonical order-
preserving model, tree-walker returning a set of skills, a sectioned N-skill
report with stable rule ids) that PROJ-002's `audit` reuses without a refactor.
Ship the foundation and the crisp validator — not the polish.

## Why Now

Nothing else can ship until skillport exists as a usable tool, and `lint` is the
credible entry point: it is table stakes (the open spec has an existing
validator, `skills-ref validate`), so it earns adoption while the substrate it
runs on is deliberately shaped so the differentiated `audit` (PROJ-002) lands as
an additive layer rather than a rewrite. Building lint as a throwaway single-file
linter now would force that rewrite later (DEC-004).

## Success Criteria

- `lint` validates a single skill, a skill folder, and a whole tree in one pass.
- Emits human-readable, `--json`, and `--sarif` output; `--json`/`--sarif`
  schemas are stable (a CI contract, DEC-005).
- Correct CI exit codes: non-zero if any **error**; also non-zero on **warning**
  under `--strict`; zero otherwise.
- One malformed skill never aborts a bulk run — reported as a per-file finding.
- The open-spec rule catalog (below) is implemented exactly, with the seeded
  severities; **no heuristic is error-level** (DEC-003).
- `--target claude` widens recognized frontmatter fields using constraints
  **verified from docs.claude.com**; other platforms stay advisory (DEC-002).
- `body.size` uses a real tokenizer (info severity).
- Per-rule unit tests + good/bad fixtures; a test proving a spec-perfect skill
  yields zero findings; a CI snippet (GitHub Action).
- The substrate (collection walker, sectioned report, stable ids) is in place for
  PROJ-002.

## Scope

### In scope
- Tolerant, lossless, order-preserving `SKILL.md` parser + canonical `Skill` model.
- Collection tree-walker (skips `.git`, `node_modules`, `target`).
- Open-spec rule engine (error / warning / info) — the catalog below.
- `lint <path>`: single skill, folder, or tree.
- `--target claude` recognized-field layer (verified); other targets advisory.
- Human + `--json` + `--sarif` output; exit codes; `--strict`.
- Real-tokenizer `body.size` estimate (info).
- A GitHub Action / CI workflow wrapping the binary.
- Per-rule tests, good/bad fixtures, README with rule ids/severities.

### Explicitly out of scope
- `--fix` autofix (deferred — needs a lossless round-trip writer + safe-fix selection).
- Anything audit/collection-level: inventory, overlap, permissions manifest,
  provenance/lockfile — that is **all PROJ-002**.
- Format conversion / migration / `push` (out per DEC-001, permanently).
- Verifying Cursor/Codex/Vercel constraints (advisory-only this wave; DEC-002).

## Stage Plan

- [ ] STAGE-001 (active) — Core substrate: tolerant lossless parser, canonical
  order-preserving `Skill` model, collection tree-walker, sectioned N-skill
  report + finding model with stable ids (built for PROJ-002 reuse).
- [ ] STAGE-002 (proposed) — Open-spec rule engine + `lint` command: implement the
  rule catalog; single-skill + tree modes; human + `--json`; exit codes +
  `--strict`; per-file parse errors don't abort a bulk run.
- [ ] STAGE-003 (proposed) — Per-platform layer + DX: `--target claude` (verified
  from primary docs); real-tokenizer `body.size`; `--sarif`; GitHub Action;
  README with rule ids/severities; per-rule tests + fixtures + the
  zero-findings-on-a-perfect-skill test.

**Count:** 0 shipped / 1 active / 2 pending

## Dependencies

### Depends on
- The open Agent Skills spec (agentskills.io) as the authoritative rule source (DEC-002).
- Optional starting point: the prototype under `initial_stuff/` — its `lint.rs`
  and `lint-fixtures/` are spec-backed and reusable; its `convert`/`push`/
  `profiles` machinery is out of scope (DEC-001) and its `claude/cursor/codex/
  vercel` profiles are **unverified guesses** (DEC-002); its `=`-pinned deps were
  a Rust-1.75 artifact — use current versions (DEC-005).

### Enables
- PROJ-002 (`audit`) — reuses the parser, model, walker, and sectioned report.

## Project-Level Reflection

*Filled in when status moves to shipped.*

- **Did we deliver the outcome in "What This Project Is"?** <not yet>
- **How many stages did it actually take?** <not yet>
- **What changed between starting and shipping?** <not yet>
- **Lessons that should update AGENTS.md, templates, or constraints?** <not yet>
- **What did we defer to the next project?** <not yet>
