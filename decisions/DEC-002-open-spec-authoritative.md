---
insight:
  id: DEC-002
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
  - verification
  - per-platform
---

# DEC-002: Only the open spec is authoritative; per-platform constraints are advisory until verified

## Decision

Validation rules come **only** from the open Agent Skills specification
(https://agentskills.io/specification). Every *per-platform* constraint
(Claude / Cursor / Codex / Vercel) must be confirmed from that platform's own
primary docs before it is encoded as anything stronger than **info**. Unverified
per-platform behavior is emitted as advisory (info) with a source comment —
**never** as an error or warning. Do not assert what you have not verified.

## Context

The prototype ships `claude/cursor/codex/vercel` profiles that are **unverified
guesses**. Encoding a guessed constraint as an error would make skillport wrong
and untrustworthy — the opposite of its value. The open spec is the one source
we can cite; the official `skills-ref validate` implements the same open-spec
checks, so the open layer is table stakes and the per-platform layer is where
correctness discipline matters most.

PROJ-001 verifies **Claude Code first** (from docs.claude.com) for `--target`;
other platforms remain advisory field-recognizers until similarly verified.

## Alternatives Considered

- **Option A: Ship the prototype's guessed profiles as real rules**
  - Why rejected: asserts unverified constraints as errors; erodes trust.

- **Option B (chosen): Open spec is authoritative; per-platform verified-or-advisory**
  - What it is: open-spec rules are firm (error/warning); a per-platform
    constraint is firm only once cited from primary docs, else info-with-source.
  - Why selected: keeps every firm assertion defensible.

## Consequences

- **Positive:** skillport never emits a wrong error; findings are citable.
- **Negative:** per-platform coverage grows only as fast as verification.
- **Neutral:** each verified platform constraint should carry a source comment /
  link in code and, where load-bearing, its own DEC.

## Validation

Right if every error/warning maps to a citable spec/doc line. Revisit the
mechanism only if the open spec itself is superseded.

## References

- Open spec: https://agentskills.io/specification
- Existing validator: `skills-ref validate` (github.com/agentskills/agentskills)
- Primary docs to verify per-platform: Claude (docs.claude.com), Cursor
  (cursor.com/docs), Codex (Codex docs / `AGENTS.md`), Vercel (skills.sh)
- Related decisions: DEC-003 (severity discipline)
