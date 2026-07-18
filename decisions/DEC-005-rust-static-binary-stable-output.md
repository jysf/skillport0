---
insight:
  id: DEC-005
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
  - "Cargo.toml"

tags:
  - architecture
  - output
  - ci
  - determinism
---

# DEC-005: Rust static binary with deterministic, stable output

## Decision

skillport is a single Rust static binary with **deterministic output**:

- Results are **sorted by path** (stable ordering across runs and machines).
- The `--json` schema is **stable and versioned** (a public CI contract, along
  with rule ids and exit codes — see `.repo-context.yaml` `version.scheme:
  semver`). Breaking any of these requires a MAJOR bump.
- **One malformed skill never aborts a bulk run** — it is reported as a per-file
  finding and the walk continues.
- Exit codes: non-zero if any error (or any warning under `--strict`), zero
  otherwise — so `lint` drops straight into CI.

## Context

skillport runs in CI, where flaky ordering, an unstable JSON shape, or an abort
on the first bad file would make it unusable. A compiled static binary gives
fast, dependency-free distribution; determinism + a stable schema make it a
dependable gate and let downstream tooling parse findings reliably.

## Alternatives Considered

- **Option A: Scripted tool (Node/Python), best-effort output**
  - Why rejected: runtime dependency to install in CI; easier to let output
    drift; slower on large trees.

- **Option B (chosen): Rust static binary, deterministic + stable schema**
  - Why selected: single artifact, fast, and a schema consumers can pin.

## Consequences

- **Positive:** trivial CI install, reproducible output, parseable JSON.
- **Negative:** schema stability is now a maintenance obligation (semver).
- **Neutral:** current stable toolchain + current dependency versions (the
  prototype's `=`-pins were a Rust-1.75 artifact and are dropped).

## Validation

Right if CI consumers pin the JSON schema and never see ordering flakiness or a
bulk-run abort. Revisit the schema only via a documented MAJOR version.

## References

- Related decisions: DEC-003 (severities → exit codes), DEC-004 (stable ids)
- Versioning: `docs/versioning.md`, `.repo-context.yaml` `version`
