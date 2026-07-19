---
project:
  id: PROJ-002
  status: active                    # framed 2026-07-19 (pivoted to before PROJ-001's release — see note)
  activity: framing
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

> **STATUS: ACTIVE (framing) — started 2026-07-19.** Framed while PROJ-001 is
> **code-complete but not yet released** (v0.1.0 tag on hold pending a project
> **rename** — the `skillport` name collides with existing apps). This was a
> deliberate call to keep momentum: the release is blocked on a human naming
> decision, so the `audit` wave's design advances in parallel. The eventual rename
> spec sweeps PROJ-002 code along with PROJ-001, so no work is lost. Stages/specs
> continue repo-wide: **first stage = STAGE-005, first spec = SPEC-018.**
>
> **Framing decisions (2026-07-19, from the user):**
> - **One `audit` command** with sectioned output + focus flags (e.g. `--security`),
>   not split commands (resolves `p2-audit-command-shape`).
> - **First stage = Inventory + library health** (STAGE-005); permissions manifest
>   and provenance follow (STAGE-006/007).
> - **Overlap detection = lexical/normalized only, no ML/embeddings** — deterministic
>   (DEC-005), no heavy dep (resolves `p2-overlap-detection-method`).
> - **AGENTS.md / instruction-file health (`agents-md-audit` signal) = parked** —
>   PROJ-002 stays skill-focused for now; revisit at a later frame.
> - Deferred to the provenance stage: lockfile format/location (`p2-lockfile-format-location`,
>   lean: `.<name>.lock` TOML at the audited root, committed) and the recognized-source
>   set (`p2-recognized-source-set`, lean: git remote + local path as the MVP).

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

## Open Questions (resolved / deferred at Frame 2026-07-19 — see guidance/questions.yaml `p2-*`)

1. **`p2-audit-command-shape` — ANSWERED:** one `audit` command, sectioned output +
   focus flags (`--security`). Not split.
2. `p2-lockfile-format-location` — **deferred to the provenance stage (STAGE-007).**
   Lean: `.<name>.lock` (TOML) at the audited root, committed.
3. **`p2-overlap-detection-method` — ANSWERED:** lexical/normalized only, no ML/embeddings
   (deterministic, no heavy dep).
4. `p2-recognized-source-set` — **deferred to the provenance stage (STAGE-007).** Lean:
   git remote (origin) + local path as the MVP recognized sources; registry later.

## Stage Plan

Framed 2026-07-19. Repo-wide continuous numbering continues from PROJ-001 (which ended
at STAGE-004 / SPEC-017).

- [~] **STAGE-005 — Inventory + library health** *(active; first stage)* — the `audit`
  command skeleton + per-collection inventory (name / path / size / token count),
  lexical description **overlap/collision** detection, and oversized / likely-dead-skill
  + description-vs-body coherence heuristics; sectioned human + `--json` report reusing
  PROJ-001's report/emit/tokenizer. `audit` is a **report, not a CI gate** (advisory
  severities; DEC-003).
- [ ] **STAGE-006 — Permissions manifest (`--security`)** *(planned)* — per skill, surface
  *what it can do*: declared `allowed-tools`, presence/type of `scripts/`, network hints;
  flag execute-/network-capable skills. A `--security` focus mode.
- [ ] **STAGE-007 — Provenance & integrity (SBOM lockfile)** *(planned)* — a
  content-hash + observed-source lockfile (DEC-006); flag drift / new-unknown /
  unrecognized-source. Fuses with STAGE-006 into the capability-plus-drift signal.

**Count:** 0 shipped / 1 active (STAGE-005) / 2 planned

## Dependencies

### Depends on
- PROJ-001 shipped (parser, `Skill` model, collection walker, sectioned report —
  DEC-004).

### Enables
- The differentiated skillport value: library trust for the skill marketplace.

## Project-Level Reflection

*Filled in when status moves to shipped.*
