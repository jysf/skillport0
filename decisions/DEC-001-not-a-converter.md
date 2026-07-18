---
insight:
  id: DEC-001
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
  - "projects/**"

tags:
  - scope
  - strategy
  - product
---

# DEC-001: skillport is a validator/auditor, not a converter

## Decision

skillport's scope is **validate / normalize / audit** agent skills. Skill↔rule
semantic migration (`.cursor/rules/*.mdc`, `AGENTS.md`, `CLAUDE.md`) and
cross-platform format conversion are **explicitly out of scope**.

## Context

Spec-compliant skills are already portable across Claude Code, Codex, Cursor,
and Vercel by design, and distribution/migration is already covered by Vercel's
`npx skills` and Cursor's native `/migrate-to-skills`. A converter is therefore
a near-no-op in a crowded lane. The unoccupied lane — and skillport's bet — is
**validation + normalization + library/security audit** with per-platform
awareness and bulk/CI ergonomics.

The optional prototype is currently converter-first (`inspect` / `convert` /
`push` / `profiles`); that machinery is out of scope by this decision. Its
`lint.rs` and fixtures are the reusable parts.

## Alternatives Considered

- **Option A: Converter-first tool**
  - What it is: parse a skill and re-emit it into another platform's layout.
  - Why rejected: already solved by first-party tooling; no differentiation.

- **Option B (chosen): Validate / normalize / audit only**
  - What it is: `lint` (per-file, CI) + `audit` (per-collection, security &
    provenance). No format conversion.
  - Why selected: this is the unoccupied, defensible lane.

## Consequences

- **Positive:** effort concentrates on the differentiated audit and CI
  ergonomics, not on a commodity converter.
- **Negative:** users wanting conversion must use other tools.
- **Neutral:** if conversion is ever revisited, it is a **separate wave** with
  explicit lossiness reporting — not smuggled into lint/audit.

## Validation

Right if skillport is adopted for CI validation and library audit, not asked to
convert. Revisit only if first-party conversion tooling disappears.

## References

- Related decisions: DEC-002 (open spec authority), DEC-004 (reuse substrate)
- External: https://agentskills.io/specification, Vercel `npx skills`,
  Cursor `/migrate-to-skills`
