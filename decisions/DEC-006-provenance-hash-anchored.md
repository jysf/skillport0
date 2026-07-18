---
insight:
  id: DEC-006
  type: decision
  confidence: 0.85
  audience:
    - developer
    - agent

agent:
  id: claude-opus-4-8
  session_id: null

project:
  id: PROJ-002
repo:
  id: skillport

created_at: 2026-07-17
supersedes: null
superseded_by: null

affected_scope:
  - "src/**"

tags:
  - security
  - provenance
  - audit
  - proj-002
---

# DEC-006: Provenance is hash-anchored, not honor-system

## Decision

Trustworthy skill provenance is a **content hash + observed source** that the
*tool* records and checks for drift. Self-asserted frontmatter (`metadata.author`,
`version`) is unverifiable — it is **reported but never trusted** as provenance.

The PROJ-002 `audit` maintains a lockfile (e.g. `.skillport.lock`) recording a
content hash + observed source per skill. On later audits it flags:
**drift** ("modified since recorded"), **new/unknown** skills, and
**unrecognized sources**.

## Context

Skill marketplaces are emerging; teams are told to audit third-party skills
before enabling exec/network capability. `metadata.author`/`version` are trivial
to forge and carry no integrity guarantee, so they cannot be the basis of trust.
A tool-recorded hash + source is the only claim skillport can stand behind. This
decision is recorded now (during PROJ-001) because it constrains the substrate
DEC-004 builds, even though it is implemented in PROJ-002.

## Alternatives Considered

- **Option A: Trust `metadata.author` / `version`**
  - Why rejected: self-asserted, unverifiable, forgeable.

- **Option B (chosen): Hash-anchored provenance recorded/checked by the tool**
  - What it is: hash + observed source in a lockfile; drift/unknown detection.
  - Why selected: only mechanism that yields a real integrity signal.

## Consequences

- **Positive:** enables the differentiated signal — "this skill, from source X,
  can run Bash and reach the network, and it has changed since you recorded it."
- **Negative:** requires a lockfile and hashing every skill.
- **Neutral:** lockfile format/location and the "recognized source" set are
  PROJ-002 Frame open questions.

## Validation

Right if audit reliably flags a modified/added skill against a recorded lockfile.
Revisit if a stronger provenance primitive (e.g. signatures) becomes standard.

## References

- Related decisions: DEC-003 (audit signals are advisory), DEC-004 (reuses walker)
- PROJ-002 kickoff: `projects/PROJ-002-skillport-audit/brief.md`
