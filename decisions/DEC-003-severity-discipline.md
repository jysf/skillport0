---
insight:
  id: DEC-003
  type: decision
  confidence: 0.95
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
  - rules
  - severity
  - ci
---

# DEC-003: Severity discipline — crisp violations are errors; heuristics are never errors

## Decision

Three severities: **error**, **warning**, **info**.

- **error** = a crisp, mechanical spec violation. Errors gate CI (non-zero exit).
- **warning** = recommended-practice / likely-wrong (spec "should"). Fails CI
  only under `--strict`.
- **info** = advisory (e.g. an unrecognized-but-ignored frontmatter key, a soft
  detail suggestion).

**No heuristic or analytical finding is ever error-level.** Fuzzy signals
(description overlap, coherence, "looks dead", size guidance beyond a hard spec
limit) are advisory and belong in the PROJ-002 `audit` report, never as an
error-level CI gate.

## Context

skillport is trusted to gate CI. A false error blocks a valid build and destroys
trust; a heuristic promoted to error guarantees false positives. Keeping errors
mechanical and citable (per DEC-002) makes the CI gate safe, and reserves the
noisy-but-useful signals for the human-read audit.

## Alternatives Considered

- **Option A: Let strong heuristics gate CI**
  - Why rejected: heuristics are probabilistic; as gates they produce false
    failures and get disabled wholesale.

- **Option B (chosen): Errors mechanical only; heuristics advisory**
  - Why selected: safe CI gate + a place (audit) where fuzzy signals still land.

## Consequences

- **Positive:** a green `lint` is meaningful; a red one is a real spec break.
- **Negative:** some genuinely useful signals never fail a build (by design —
  they surface in `audit`).
- **Neutral:** `--strict` lets teams opt into treating warnings as failures.

## Validation

Right if teams leave `lint` in required CI without exemptions. Revisit if a
class of crisp violation is found that the three-severity model can't express.

## References

- Related decisions: DEC-002 (only verified constraints are firm), DEC-004
  (audit reuses the same finding model)
- Rule catalog + severities: `projects/PROJ-001-skillport-lint/stages/` and README
