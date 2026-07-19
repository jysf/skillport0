---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-013
  type: chore                      # epic | story | task | bug | chore
  cycle: ship  # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: S                    # S | M | L  (L means split it)

project:
  id: PROJ-001
  stage: STAGE-004
repo:
  id: skillport

agents:
  architect: claude-opus-4-8      # design cycle (this orchestrator session)
  implementer: claude-sonnet-5    # build runs as a Sonnet subagent (cost); updated with the real model
  created_at: 2026-07-18

references:
  decisions:
    - DEC-009   # distribution strategy — this is its Phase-0 pre-flight
    - DEC-005   # the CLI/JSON/exit-code contract is frozen; release work is packaging, not behavior
  constraints:
    - deterministic-stable-output   # no behavior/schema change
    - license-policy                # cargo-deny already gates deps; dual license must stay policy-clean
    - test-before-implementation
  related_specs:
    - SPEC-009  # the Action (identity target; its --git install is SPEC-016's job, not this spec's)
    - SPEC-012  # README refresh (this updates the README License section it left stale)

value_link: "infrastructure enabling STAGE-004's release: the Phase-0 pre-flight from DEC-009 — dual-license files, crates.io metadata, and identity consistency — so the crate can be packaged and published without a scramble."

# Self-reported AI cost per cycle. Each cycle (design, build, verify,
# ship) appends one entry to sessions[]. Totals are computed at ship.
# Record a REAL tokens_total for metered cycles (build/verify) — the
# orchestrator fills it from the Agent result's subagent_tokens at ship
# (or /cost interactively). Only un-metered main-loop cycles (design/ship)
# may be null-with-note. `just cost-audit` enforces this on shipped specs.
# See AGENTS.md §4 and docs/cost-tracking.md. interface: claude-code |
# claude-ai | api | ollama | other.
cost:
  sessions:
    - cycle: design
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-07-18
      notes: "main-loop, not separately metered (design cycle); includes the design-time probe (crates.io name-availability check = free/404, identity-inconsistency scan, current LICENSE = Apache-2.0)"
    - cycle: build
      agent: claude-sonnet-5
      interface: claude-code
      tokens_total: 64164
      estimated_usd: 0.42
      duration_minutes: 15
      recorded_at: 2026-07-18
      notes: "metered Sonnet build subagent; tokens_total = subagent_tokens. estimated_usd = tokens x repo rate 6.60 (order-of-magnitude). duration wall-clock. Smallest build so far (config/packaging only)."
    - cycle: verify
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: 54156
      estimated_usd: 0.36
      duration_minutes: 3
      recorded_at: 2026-07-18
      notes: "metered Opus verify subagent; ran cargo publish --dry-run on the clean committed tree (exit 0), diffed the Apache text against main, confirmed no src/Cargo.lock change. APPROVED, 0 punch-list."
    - cycle: ship
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-07-18
      notes: "main-loop, not separately metered (ship cycle)"
  totals:
    tokens_total: 118320
    estimated_usd: 0.78
    session_count: 4
shipped_at: 2026-07-18
---

# SPEC-013: release phase-0 prep — dual license + crates.io metadata

## Context

First spec of STAGE-004 (Release & distribution). Per DEC-009's **Phase-0 pre-flight**,
before any release workflow or `cargo publish` can happen the repo must be
release-consistent: the license on disk must match `Cargo.toml`'s declared
`MIT OR Apache-2.0`, the crate must carry the crates.io metadata a good listing needs,
and the identity must be uniformly `github.com/jysf/skillport`. This spec does that
prep and nothing else — it is deliberately the smallest, safest first step, with **no
runtime code change** and **no publish** (publishing is human-only, SPEC-015).

Design-time probe (done this cycle):
- The crate name **`skillport` is free** on crates.io (API returned 404 "does not
  exist", 2026-07-18). Re-confirm immediately before first publish (SPEC-015).
- The on-disk `LICENSE` is the **Apache-2.0** text; there is no `LICENSE-MIT`, so the
  files do **not** yet match `Cargo.toml`'s dual `MIT OR Apache-2.0`.
- Identity is *almost* consistent already: `Cargo.toml repository`, `action.yml`, and
  `.repo-context.yaml` all say `jysf/skillport`. The only `skillport0` references are
  (a) an archived SPEC-001 PR link (a real historical URL — leave it) and (b) DEC-009's
  own "renamed from `skillport0`" note (correct — leave it). The stale item is the
  **README `## License` section**, which still says "Apache-2.0 … final license is a
  call to confirm before first release".
- Git author identity for the `authors` field: `jysf <jyashinsky@gmail.com>`.

## Goal

Make skillport packageable for crates.io: replace the single Apache `LICENSE` with
dual `LICENSE-MIT` + `LICENSE-APACHE` matching `Cargo.toml`, fill the crates.io Cargo
metadata (`readme`, `keywords`, `categories`, `homepage`, `authors`), update the stale
README License section, and prove it with a clean `cargo publish --dry-run` — all with
no runtime code or contract change.

## Inputs

- **Files to read/modify:** `Cargo.toml` (add metadata), `LICENSE` (→ `LICENSE-APACHE`),
  `README.md` (License section), and add `LICENSE-MIT`.
- **Reference:** `decisions/DEC-009-distribution-strategy.md` (Phase-0 pre-flight +
  crates.io metadata list), crates.io category slugs
  (https://crates.io/category_slugs), the standard MIT license text.
- **No source (`src/`) changes.** No new dependency.

## Outputs

- **Files created:**
  - `LICENSE-MIT` — the standard MIT License text, copyright line
    `Copyright (c) 2026 jysf` (year 2026, holder `jysf`).
- **Files renamed:**
  - `LICENSE` → `LICENSE-APACHE` (the existing Apache-2.0 text, unchanged content).
    Use `git mv` so history is preserved.
- **Files modified:**
  - `Cargo.toml` `[package]` — add:
    - `authors = ["jysf <jyashinsky@gmail.com>"]`
    - `readme = "README.md"`
    - `homepage = "https://github.com/jysf/skillport"`
    - `keywords = ["skills", "linter", "agent", "validation", "cli"]` (≤ 5, each ≤ 20
      chars, ASCII — adjust only if crates.io rejects one)
    - `categories = ["command-line-utilities", "development-tools"]` (both are valid
      crates.io slugs; verify against the slug list — do not invent slugs)
    - keep `license = "MIT OR Apache-2.0"` (do NOT add `license-file`; it is mutually
      exclusive with `license`). Keep `repository` as-is.
  - `README.md` `## License` — replace the "Apache-2.0 … call to confirm" paragraph
    with the dual-license statement: skillport is licensed under **either** MIT
    (`LICENSE-MIT`) **or** Apache-2.0 (`LICENSE-APACHE`) at the user's option — the
    Rust-idiomatic dual license — linking both files.
- **Optional (recommended, keep it small):** a CI guard — extend `.github/workflows/ci.yml`
  with a step (or fold into an existing job) that runs `cargo publish --dry-run` so
  packaging can't silently regress. If it complicates the S scope, document it as a
  SPEC-014/015 follow-up instead.
- **No `src/`/behavior/schema/exit-code change** (DEC-005).

## Acceptance Criteria

- [x] `LICENSE-MIT` and `LICENSE-APACHE` both exist at repo root; no bare `LICENSE`
      file remains. `LICENSE-APACHE` is the original Apache-2.0 text (unchanged);
      `LICENSE-MIT` is the standard MIT text with `Copyright (c) 2026 jysf`.
- [x] `Cargo.toml` declares `license = "MIT OR Apache-2.0"` **and** `authors`,
      `readme`, `homepage`, `keywords` (≤ 5, each ≤ 20 chars), `categories` (only valid
      crates.io slugs). No `license-file` key.
- [x] `cargo publish --dry-run` (equivalently `cargo package`) **succeeds** and the
      packaged file list includes `LICENSE-MIT`, `LICENSE-APACHE`, and `README.md`.
- [x] `cargo metadata --no-deps --format-version 1` shows the new fields
      (authors/keywords/categories/homepage) populated for the `skillport` package.
- [x] The README `## License` section states the dual MIT-OR-Apache license and links
      both `LICENSE-MIT` and `LICENSE-APACHE`; no "call to confirm" / "inherited from
      the template" language remains.
- [x] `cargo test` / `clippy --all-targets -- -D warnings` / `fmt --check` still green;
      no runtime code changed; no new dependency; no `--json`/SARIF/exit-code/rule-id
      change (DEC-005).

## Failing Tests

This is a **packaging/config** spec, so the "failing tests" are concrete command
checks run against the tree (the `test-before-implementation` discipline is satisfied
by pre-specifying these exact assertions; the build makes them pass). No Rust unit
tests are added.

- **Packaging dry-run** — `cargo publish --dry-run` exits `0`. Before build it fails
  (missing `readme` file reference / absent metadata surfaces as an error or the
  packaged crate is incomplete). After build it passes.
- **License files present** — after build: `LICENSE-MIT` and `LICENSE-APACHE` exist and
  a bare `LICENSE` does not; `LICENSE-APACHE` first line contains `Apache License`;
  `LICENSE-MIT` contains `Copyright (c) 2026 jysf` and the word `MIT`.
- **Metadata present** — `cargo metadata --no-deps --format-version 1` for the
  `skillport` package shows `authors`, `keywords`, `categories`, `homepage`, `readme`
  non-empty and `license == "MIT OR Apache-2.0"`, with `categories` ⊆ the valid slug set.
- **Packaged contents** — `cargo package --list` includes `LICENSE-MIT`,
  `LICENSE-APACHE`, `README.md`, and `Cargo.toml`.
- **README updated** — the `## License` section links both `LICENSE-MIT` and
  `LICENSE-APACHE` and does NOT contain the strings "call to confirm" or "inherited
  from the template".

## Implementation Context

*Read this section before starting the build cycle.*

### Decisions that apply

- `DEC-009` — this spec is its Phase-0 pre-flight (dual license, crates.io metadata,
  identity). Do **not** go further into the attack plan (no release workflow, no
  publish job, no Action change — those are SPEC-014/015/016). **Do not run
  `cargo publish` for real** — dry-run only; publishing is a human-only step (SPEC-015).
- `DEC-005` — the `--json` schema, SARIF, exit codes, and rule ids are a frozen public
  contract. This spec touches none of them; if a change to `src/` seems needed, stop —
  it's out of scope.

### Constraints that apply

- `license-policy` — cargo-deny already gates dependency licenses in CI; the dual
  MIT/Apache choice for skillport itself must stay policy-clean (it is — both are
  permissive and OSI-approved). Don't introduce a copyleft file.
- `deterministic-stable-output` — no behavior/output change.
- `test-before-implementation` — the command checks above are the pre-written
  verification; build makes them pass.

### Prior related work

- `DEC-009` (2026-07-18) — set the strategy; this executes Phase-0.
- `SPEC-012` (shipped, PR #12) — last refreshed the README; this updates only its
  License section, which SPEC-012 did not (it was out of that spec's scope).
- `SPEC-009` (shipped, PR #9) — the Action; its `cargo install --git` line is
  intentionally **left alone** here (SPEC-016 swaps it for a release-binary download).

### Out of scope (for this spec specifically)

- **No `cargo publish` for real** (human-only, SPEC-015).
- **No release workflow** (`release.yml`), **no Action change**, **no version bump**,
  **no tag**, **no CHANGELOG** (later STAGE-004 specs).
- **No `src/` change**, no dependency change, no contract change.
- Not touching the historical `skillport0` references (the archived SPEC-001 PR link
  and DEC-009's rename note) — those are correct as-is.

## Notes for the Implementer

- Use `git mv LICENSE LICENSE-APACHE` to preserve history; do not edit the Apache text.
- Get the standard MIT text right (the canonical short form). Copyright line exactly:
  `Copyright (c) 2026 jysf`.
- crates.io **categories must be valid slugs** — `command-line-utilities` and
  `development-tools` are both valid; if you add/substitute any, check
  https://crates.io/category_slugs (an invalid slug fails `cargo publish`). Keywords
  are freeform but capped (≤ 5, ≤ 20 chars each, ASCII).
- Run `cargo publish --dry-run` as the primary proof. It both packages and type-checks
  the crate in isolation, catching a missing `readme` file or a bad category. (If the
  working tree is mid-edit when you run it, `--allow-dirty` is fine for the check; the
  final clean run should not need it.)
- Keep the diff tight: `Cargo.toml`, `LICENSE-MIT` (new), `LICENSE`→`LICENSE-APACHE`
  (rename), `README.md` (License section), and optionally the CI guard step.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-013-release-prep`
- **PR (if applicable):** none (build cycle only; orchestrator opens PR later)
- **All acceptance criteria met?** yes
- **New decisions emitted:**
  - none
- **Deviations from spec:**
  - None. Added the optional CI guard (recommended in spec) rather than
    skipping it — it was a two-line addition to the existing `rust` job in
    `.github/workflows/ci.yml` (a new `cargo publish --dry-run` step after
    `cargo test`), well within "keep it small".
- **Follow-up work identified:**
  - None new; SPEC-014/015/016 already cover the release workflow, real
    publish, and Action `cargo install --git` swap per DEC-009.

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — Nothing; the spec's file list, exact copyright line, and category slugs
   were precise enough to implement directly without guessing.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — None found; DEC-009 and DEC-005 covered the relevant boundaries
   (packaging-only, no contract change) completely.

3. **If you did this task again, what would you do differently?**
   — Nothing material; would run `cargo publish --dry-run` once early
   (with `--allow-dirty`) to catch packaging issues before finishing the
   README/CI edits, exactly as done here.

---

## Reflection (Ship)

*Appended during the **ship** cycle. Outcome-focused reflection, distinct
from the process-focused build reflection above.*

1. **What would I do differently next time?**
   — Nothing material. The design-time probe (checking crates.io for the name, scanning
   for identity drift, confirming the on-disk license) made this the smallest, fastest
   cycle so far — the "failing test" was literally `cargo publish --dry-run`, which is
   the exact command a release runs, so build was pure transcription. The pattern for a
   packaging spec: make the real release command the acceptance test.

2. **Does any template, constraint, or decision need updating?**
   — No. DEC-009 already sequenced this; DEC-005 kept it honest (0 lines under `src/`).
   The `cargo publish --dry-run` CI guard the build added is a nice standing tripwire so
   packaging can't silently regress before SPEC-015 actually publishes — no signal
   needed, it's just good hygiene.

3. **Is there a follow-up spec I should write now before I forget?**
   — Yes, the STAGE-004 backlog continues: SPEC-014 (release workflow — cross-compile
   matrix on `v*`, strip/archive/sha256/attach), then SPEC-015 (crates.io publish —
   re-confirm the name is free, then the human-only `cargo publish`), SPEC-016 (Action
   downloads the release binary), SPEC-017 (cut v0.1.0 — CHANGELOG + install matrix +
   human-only tag push). SPEC-014 is next.

4. **Where was the worst defect caught?** — one word from a fixed vocabulary so
   the defect-escape distribution is greppable across specs:
   `design` | `build` | `verify` | `ship` | `escaped` (reached prod/runtime) |
   `none` (clean first try).
   — none
   *(Runtime/operational defects — the escape-prone class — only exist once the
   artifact meets its real host. `escaped` here is a signal to strengthen the
   §12 behavioral pre-flight for that surface.)*
