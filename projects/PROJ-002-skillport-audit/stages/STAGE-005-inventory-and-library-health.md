---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-005                     # stable, zero-padded, continuous across the repo
  status: active                    # proposed | active | shipped | cancelled | on_hold
  priority: high                    # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-002                      # parent project
repo:
  id: skillport

created_at: 2026-07-19
shipped_at: null

# What part of the project's value thesis this stage advances.
# If you can't articulate value_contribution, the stage may be
# infrastructure-only — acceptable but flag it.
value_contribution:
  advances: "The 'library health' third of the project thesis — the first per-collection read that makes a skill library legible (inventory + overlap + health), and the `audit` command substrate the security (STAGE-006) and provenance (STAGE-007) stages extend."
  delivers:
    - "`audit <path>` walks a skill library and reports an inventory (name / path / size / token count) + a summary"
    - "lexical description overlap/collision detection (near-duplicate descriptions that confuse agent routing)"
    - "library-health heuristics: oversized / likely-dead skills + description-vs-body coherence"
    - "a sectioned human report + a stable `--json` audit schema, reusing PROJ-001's walker / report / tokenizer"
  explicitly_does_not:
    - "the permissions manifest / `--security` focus mode (STAGE-006)"
    - "provenance / the hash-anchored lockfile (STAGE-007)"
    - "AGENTS.md / instruction-file health (`agents-md-audit` — parked)"
    - "any CI-gating semantics — `audit` is an advisory report, not a gate (DEC-003)"
---

# STAGE-005: inventory and library health

## What This Stage Is

The stage that stands up the **`audit` command** and its first, table-stakes read:
point it at a folder or tree of skills and get back a legible picture of the library —
an **inventory** (what skills exist, where, how big), **overlap/collision** among
descriptions (near-duplicates that make an agent pick the wrong skill), and a few
**health** signals (oversized skills, likely-dead stubs, description-vs-body
incoherence). Unlike `lint` (per-file, CI-gating, spec-conformance), `audit` is
per-collection and produces a report a human reads periodically or before enabling a
library — so heuristic checks that would be too noisy as CI gates are appropriate here
(DEC-003). This stage also establishes the `audit` report substrate (inventory rows +
advisory sections + a stable `--json` schema) that STAGE-006 (`--security`) and
STAGE-007 (provenance) extend without a rewrite.

## Why Now

It's the first stage of PROJ-002 and the natural MVP of `audit`: inventory + health is
the credible, low-risk entry that makes the command real and useful on its own, and it
builds the report/output substrate the differentiated security + provenance signals plug
into. It stands entirely on PROJ-001's shipped substrate (walker, `Skill` model,
sectioned report, real tokenizer — DEC-004 built these collection-first precisely so this
lands as an additive layer, not a rewrite). (Framed 2026-07-19 while PROJ-001 is
code-complete but pre-release/rename — see the project brief.)

## Success Criteria

- `audit <path>` walks a single skill, a folder, or a tree and prints an **inventory**
  (per skill: name, relative path, size, token count) + a summary line (N skills, totals).
- Surfaces **description overlap/collision** via deterministic lexical/normalized
  similarity — exact-normalized dupes and high-overlap near-dupes — never using an
  ML/embedding dependency.
- Surfaces **health** flags: oversized skills (reusing the real tokenizer), likely-dead
  stubs, and description-vs-body coherence — all advisory.
- Emits a **sectioned human report** and a **stable, versioned `--json`** audit schema
  (deterministic, path-sorted; DEC-005); `audit` exits 0 as a report (usage errors → 2),
  not a CI gate.
- Reuses PROJ-001's `walk` / report / tokenizer with **no rewrite** of the substrate and
  **no new runtime dependency** (overlap is lexical).

## Scope

### In scope
- The `audit` subcommand (clap) alongside `lint`; the audit report shape (inventory +
  advisory sections) + human and `--json` emitters.
- Inventory read (name / path / size / token count) + summary.
- Lexical description overlap/collision detection.
- Library-health heuristics: oversized, likely-dead, description-vs-body coherence.

### Explicitly out of scope
- **Permissions manifest / `--security`** (STAGE-006) and **provenance / lockfile**
  (STAGE-007).
- **AGENTS.md / instruction-file health** (parked `agents-md-audit` signal).
- **CI-gating** semantics, autofix, conversion/migration (DEC-001).
- **ML / embeddings** for overlap (lexical only, DEC-005 determinism).
- Changing any `lint` behavior or its `--json`/SARIF/exit-code/rule-id contract (DEC-005).

## Spec Backlog

Proposed decomposition (turned into specs via `just new-spec` as the stage progresses).
Repo-wide numbering continues from SPEC-017.

- [~] **SPEC-018 — `audit` command + inventory** *(build)* — the `audit <path>` subcommand
  (clap, alongside `lint`); walk → per-skill **inventory** (name / path / **token count**
  as the headline metric; bytes+lines in `--json`) + summary (skills, tokens_total,
  unreadable-counted); human report + a **separate** versioned `--json` audit schema
  (`AUDIT_SCHEMA=1`, distinct from lint's); report semantics (exit 0; usage → 2). Exposes
  the tokenizer (`token_count`). Establishes the growable `AuditReport` substrate. No `lint`
  change, no new dep (DEC-003/004/005).
- [ ] **SPEC-019 — description overlap / collision** *(planned)* — deterministic
  lexical/normalized similarity across skill descriptions; flag exact-normalized dupes and
  high-overlap near-dupes as an audit section. No ML.
- [ ] **SPEC-020 — library-health heuristics** *(planned)* — oversized skills (via the
  tokenizer), likely-dead stubs, and description-vs-body coherence, as advisory health
  flags. (May split if it runs large.)

**Count:** 0 shipped / 0 active / 3 pending (SPEC-018 designed next)

## Design Notes

- **`audit` is a report, not a gate (DEC-003).** All its findings are advisory; exit code
  is 0 for a normal run (usage errors → 2, matching `lint`). Do **not** give audit
  error-level CI-failing semantics.
- **Reuse, don't rewrite (DEC-004).** `walk` (SPEC-002), the `Skill` model (SPEC-001), the
  real tokenizer (SPEC-010), and the emit conventions (SPEC-005/008) are the substrate.
  The audit *report shape* is new (inventory rows + aggregate/pairwise sections, not purely
  per-file `Finding`s) — SPEC-018 designs it; it may reuse `Finding`/`Severity` for the
  advisory flag-type outputs and add an inventory/summary structure for the tabular part.
- **Determinism + stable schema (DEC-005).** Path-sorted output; a **separate** versioned
  `--json` audit schema (don't overload the `lint` schema). Rename note: the `--json`
  `tool` field and any lockfile name will be swept by the pending project rename.
- **No new runtime dependency** — overlap is lexical (normalize + compare); reuse the
  existing tokenizer for size.

## Dependencies

### Depends on
- PROJ-001's shipped substrate — `walk` (STAGE-001), the `Skill` model + report layer,
  the real tokenizer (STAGE-003). All shipped; audit is an additive layer (DEC-004).
- The `lint`/CLI surface (SPEC-005) that `audit` slots alongside.

### Enables
- **STAGE-006** (`--security` permissions manifest) and **STAGE-007** (provenance/lockfile)
  — both extend the audit report substrate this stage builds.
- The differentiated skillport value: library trust for the skill marketplace.

## Stage-Level Reflection

*Filled in when status moves to shipped. Run Prompt 1c (Stage Ship) in
FIRST_SESSION_PROMPTS.md to draft this.*

- **Did we deliver the outcome in "What This Stage Is"?** <yes/no + notes>
- **How many specs did it actually take?** <number vs. plan>
- **What changed between starting and shipping?** <one sentence>
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - <one-line updates>
- **Signals dispositioned at this close?** (Prompt 1d step 7) Every
  `type: lesson` signal in `/guidance/signals.yaml` owned by this stage close
  was walked — codified (at its bar), left `watch`, or dropped. No silent carry.
  - <note what codified / what's still watch + its N>
- **Should any spec-level reflections be promoted to stage-level lessons?**
  - <one-line items — record below-bar ones as `watch` signals; don't codify yet>
