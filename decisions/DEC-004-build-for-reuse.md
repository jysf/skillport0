---
insight:
  id: DEC-004
  type: decision
  confidence: 0.9
  audience:
    - developer
    - agent

agent:
  id: claude-opus-4-8
  session_id: null

project:
  id: PROJ-001
repo:
  id: skillport

created_at: 2026-07-17
supersedes: null
superseded_by: null

affected_scope:
  - "src/**"

tags:
  - architecture
  - substrate
  - reuse
---

# DEC-004: Build PROJ-001 as a collection-first substrate for PROJ-002 to reuse

## Decision

The PROJ-001 substrate is **collection-first from day one**, so PROJ-002's
`audit` is an additive layer, not a refactor:

1. The tree-walker returns a **set of skills** (a collection), never a single
   skill — even when linting one file.
2. The parser produces a **canonical, order-preserving, lossless** `Skill`
   model (frontmatter order preserved; nothing dropped on round-trip).
3. The report layer takes **N skills with sections**, not a single pass/fail.
4. Every rule and finding has a **stable id** (e.g. `name.charset`).

## Context

PROJ-002 (`audit`) analyzes a *collection* and reuses PROJ-001's parser, model,
walker, and report. If PROJ-001 were built as a single-file linter, PROJ-002
would begin with a rewrite. Shaping the substrate for N-skills up front makes
the audit additive. This is why skillport is not "just a linter with a folder
loop" — the folder/collection is the primary unit.

## Alternatives Considered

- **Option A: Single-skill linter, add bulk later**
  - Why rejected: forces a refactor of the model/report at PROJ-002 start.

- **Option B (chosen): Collection-first substrate from day one**
  - Why selected: `audit` layers on top; stable ids let both commands and CI
    consumers reference findings unambiguously.

## Consequences

- **Positive:** PROJ-002 is additive; ids are a stable public contract (DEC-005).
- **Negative:** slightly more structure than a one-file linter needs on day one.
- **Neutral:** one malformed skill must never abort a bulk run — report it as a
  per-file finding and continue (see DEC-005).

## Validation

Right if PROJ-002 ships `audit` without changing the parser/model/walker/report
signatures. Revisit if the collection model proves wrong for audit's needs.

## References

- Related decisions: DEC-003 (shared finding model), DEC-005 (stable output),
  DEC-006 (provenance builds on the walker)
- PROJ-002 kickoff: `projects/PROJ-002-skillport-audit/brief.md`
