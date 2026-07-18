---
project:
  id: PROJ-002
  status: proposed                  # NOT STARTED — framed after PROJ-001 ships
  activity: null
  priority: high
  target_ship: null

repo:
  id: skillport

created_at: 2026-07-17
shipped_at: null

value:
  thesis: >
    An `audit` command over a collection of skills emits a signal nothing else
    produces today — "this skill, from source X, can run Bash and reach the
    network, and it has changed since you recorded it" — the trust gap in the
    emerging skill marketplaces. This is skillport's differentiated core.
  beneficiaries:
    - teams enabling third-party skills (need a health + security + provenance read)
    - skill-library maintainers (inventory, overlap/collision, dead-skill detection)
    - security reviewers (permissions manifest + drift detection)
  success_signals:
    - "`audit` walks a library and reports inventory + health"
    - "a permissions manifest surfaces what each skill can do (allowed-tools, scripts/, network hints)"
    - "a hash-anchored lockfile detects drift, new/unknown skills, unrecognized sources"
    - "the fused capability-plus-drift risk signal is surfaced"
  risks_to_thesis:
    - "overlap detection is noisy if purely lexical (start lexical, no ML — open question)"
    - "provenance is only as good as the recognized-source set (open question)"
    - "scope creep into converter/migration territory (barred by DEC-001)"
---

# PROJ-002: The audit (library health + security + provenance)

> **STATUS: PROPOSED — NOT STARTED.** This brief captures Part 3 of the seed so
> the wave is defined and its decisions (DEC-006) are recorded, but no stages or
> specs exist yet and no code is written. Frame this wave (GETTING_STARTED Step 2)
> **only after PROJ-001 ships.** Its `STAGE-*` / `SPEC-*` numbers continue
> repo-wide from wherever PROJ-001 ends (e.g. STAGE-004+).

## What This Project Is

An `audit` command that analyzes a *collection* of skills — the differentiated
core of skillport. Think "SBOM + health report for a skill library." It reuses
PROJ-001's parser, model, tree-walker, and sectioned report (that reuse is why
PROJ-001 is built collection-first — DEC-004). Where `lint` is per-file and gates
CI, `audit` is per-collection and produces a report a human reads periodically or
before enabling third-party skills — so the fuzzy checks that would be noisy as
CI gates are appropriate here (DEC-003).

## Why Now

Deferred until PROJ-001 ships: audit stands on PROJ-001's substrate, and lint is
the credible table-stakes entry point that earns skillport adoption first. Once
the substrate exists, audit is an additive layer, not a rewrite.

## Success Criteria

- `audit <path>` walks a library and reports **inventory + health**.
- Surfaces a **permissions manifest** per skill (what it can do).
- Maintains a **hash-anchored lockfile** with drift / new-unknown / unrecognized-
  source detection (DEC-006 — self-asserted `metadata` reported, never trusted).
- Surfaces the fused **capability-plus-drift** risk signal.

## Scope

### In scope (suggested stages — decompose at Frame)
1. **Inventory + library health.** Inventory (name, size, location); description
   **overlap/collision** detection (near-duplicate descriptions confuse agent
   routing — high value); oversized / likely-dead skills; description-vs-body
   coherence. Sectioned report reusing PROJ-001's report layer.
2. **Permissions manifest.** Per skill, surface *what it can do*: declared
   `allowed-tools`, presence/type of `scripts/`, network hints — flag anything
   execute- or network-capable. A `--security` focus mode.
3. **Provenance & integrity (SBOM).** A lockfile (e.g. `.skillport.lock`)
   recording a **content hash + observed source** per skill; on later audits flag
   drift, new/unknown skills, unrecognized sources (DEC-006).

> Where the value concentrates: stages 2–3 fuse into the one signal nothing else
> emits today (capability + provenance drift).

### Explicitly out of scope
- Anything that belongs to `lint` / the open-spec layer (PROJ-001).
- Format conversion / migration (DEC-001, permanently).
- Trusting self-asserted `metadata.author` / `version` as provenance (DEC-006).

## Open Questions (surface at Frame — see guidance/questions.yaml `p2-*`)

1. One `audit` command with sections + a `--security` flag, or split `audit`
   (health) from a separate `provenance`/`sbom` command? *(Lean: one command,
   sections, focus flags.)*
2. Lockfile format + location — `.skillport.lock` (TOML/JSON?) at the audited
   root; committed to the repo or not?
3. Overlap detection method — exact/normalized string match to start, or embed
   for semantic similarity later? *(Lean: start lexical, no ML dep.)*
4. What counts as a recognized "source" for provenance (git remote, registry,
   local)? Define the minimal viable set.

## Stage Plan

*Not yet defined — framed after PROJ-001 ships. The three suggested stages above
become STAGE-004+ (repo-wide continuous numbering).*

- [ ] (not yet defined) — Inventory + library health
- [ ] (not yet defined) — Permissions manifest (`--security`)
- [ ] (not yet defined) — Provenance & integrity (SBOM lockfile)

**Count:** 0 shipped / 0 active / 0 defined

## Dependencies

### Depends on
- PROJ-001 shipped (parser, `Skill` model, collection walker, sectioned report —
  DEC-004).

### Enables
- The differentiated skillport value: library trust for the skill marketplace.

## Project-Level Reflection

*Filled in when status moves to shipped.*
