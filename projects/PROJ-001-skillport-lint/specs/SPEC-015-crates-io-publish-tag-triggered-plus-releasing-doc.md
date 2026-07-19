---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-015
  type: story                      # epic | story | task | bug | chore
  cycle: build  # frame | design | build | verify | ship
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
    - DEC-009   # distribution strategy — this is Attack-plan step 3 (crates.io publish)
    - DEC-005   # frozen contract — publishing is packaging, not behavior
  constraints:
    - deterministic-stable-output
    - license-policy
    - test-before-implementation
  related_specs:
    - SPEC-014  # release.yml (this adds a tag-gated publish job to it)
    - SPEC-013  # Phase-0 metadata/licenses that make the crate publishable

value_link: "infrastructure enabling STAGE-004's release: `cargo install skillport` — a tag-gated crates.io publish job (fired by the same v* tag as the binary release) plus a RELEASING doc, so the human can publish safely and repeatably. The publish itself (token + tag) is human-only per DEC-009."

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
      notes: "main-loop, not separately metered (design cycle); re-confirmed the crates.io name `skillport` is still free (404), reviewed the SPEC-014 release.yml job layout"
    - cycle: build
      agent: claude-sonnet-5
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-07-18
      notes: "metered subagent build; orchestrator fills tokens_total/duration/estimated_usd from the Agent result at ship"
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-015: crates.io publish (tag-triggered) + RELEASING doc

## Context

STAGE-004 Attack-plan step 3 (DEC-009): make `cargo install skillport` possible. SPEC-013
made the crate packageable (`cargo publish --dry-run` is green in CI) and SPEC-014 added
`release.yml` that builds binaries on a `v*` tag. This spec adds the **crates.io publish**
to that same tag-triggered pipeline — a `publish` job in `release.yml` that runs
`cargo publish` — plus a **`RELEASING.md`** documenting the human-only setup (crates.io
token secret, ownership) and the release procedure.

The publish itself is **human-only** (DEC-009): it needs a crates.io API token (an
irreversible credential the human holds) and is fired by a `v*` tag push (also human).
This spec **prepares** the automation and the runbook; it does **not** publish anything.
The job is inert until the human (a) adds the `CARGO_REGISTRY_TOKEN` secret and (b)
pushes a tag (SPEC-017). The crate name `skillport` was re-confirmed **free** at design
(crates.io API 404, 2026-07-18) — the RELEASING doc instructs a final re-check + a manual
first publish to establish ownership before the automation is relied on.

## Goal

Add a tag-gated `publish` job to `.github/workflows/release.yml` that runs
`cargo publish --locked` with a `CARGO_REGISTRY_TOKEN` secret (only on a `v*` tag, only
if the version matches, only after the binaries build), and add `RELEASING.md` — the
human runbook for the token secret, the first manual publish, and the tag-driven release
flow. No runtime code change; nothing is actually published.

## Inputs

- **Files to modify:** `.github/workflows/release.yml` (add the `publish` job).
- **Files to create:** `RELEASING.md` (repo root).
- **Files to read:** `Cargo.toml` (version/name), `decisions/DEC-009` (step 3 + the
  human-only guardrail), `.github/workflows/release.yml` (the `version`/`build`/`release`
  jobs to slot alongside).
- **No `src/` change. No new Cargo dependency.**

## Outputs

- **`.github/workflows/release.yml` — add a `publish` job:**
  - `if: startsWith(github.ref, 'refs/tags/v')` (tag-only; **skipped on
    `workflow_dispatch`** — a dry dispatch must never publish), `needs: [version, build]`
    (only publish if the whole binary matrix succeeded), `runs-on: ubuntu-latest`.
  - Steps: checkout; `dtolnay/rust-toolchain@stable`; a **version-match guard** —
    fail if the tag version (`needs.version.outputs.version`) ≠ the `Cargo.toml`
    version (prevents publishing a mismatched crate); `cargo publish --locked` with
    `env: CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}`.
  - No `permissions` beyond default (publish uses the crates.io token, not
    `GITHUB_TOKEN`). Pin actions to major like the rest of the file.
  - The token is referenced **only** as `${{ secrets.CARGO_REGISTRY_TOKEN }}` — never a
    literal; if the secret is unset the job fails at publish (acceptable — it means the
    human hasn't finished setup).
- **`RELEASING.md`** (repo root) — the human runbook:
  - **One-time setup:** create a crates.io account; generate a scoped API token;
    add it as the `CARGO_REGISTRY_TOKEN` GitHub Actions secret (Settings → Secrets and
    variables → Actions); re-confirm `skillport` is free (`cargo search skillport` / the
    crates.io page) and do the **first `cargo publish` manually** (`cargo publish
    --locked`) to establish crate ownership. After that, tag pushes auto-publish.
  - **Per-release flow:** bump the version + CHANGELOG (SPEC-017 / `just next-version`),
    commit, push `vX.Y.Z` → `release.yml` builds the binaries (SPEC-014) **and** this
    `publish` job publishes to crates.io. Optionally smoke-test with a
    `workflow_dispatch` run first (builds artifacts, no Release, no publish).
  - **Guardrails:** the tag version must equal `Cargo.toml`'s version (the job enforces
    this); a version already on crates.io cannot be re-published (bump to retry); macOS
    binaries are unsigned until an Apple key exists (Homebrew deferred, DEC-009).
  - Link `decisions/DEC-009` and note which steps are human-only.
- **No change to `src/`, `Cargo.toml`, `ci.yml`, `action.yml`** (the Action swap is
  SPEC-016; the version bump/CHANGELOG/tag are SPEC-017).

## Acceptance Criteria

- [ ] `release.yml` gains a `publish` job that is **tag-only**
      (`if: startsWith(github.ref, 'refs/tags/v')`), `needs: [version, build]`, and runs
      `cargo publish --locked` with the token from `${{ secrets.CARGO_REGISTRY_TOKEN }}`
      (never a literal). `actionlint` passes on the whole file.
- [ ] The `publish` job includes a version-match guard step that **fails** when the tag
      version ≠ the `Cargo.toml` version, and otherwise proceeds.
- [ ] On `workflow_dispatch` the `publish` job is **skipped** (no publish on a dry run),
      same as the `release` job.
- [ ] `RELEASING.md` exists at repo root and documents: the `CARGO_REGISTRY_TOKEN` secret
      setup, the manual first-publish to establish ownership, the tag-driven per-release
      flow, and the version-match / already-published guardrails. It marks the human-only
      steps.
- [ ] No third-party publish action; only `checkout` + `dtolnay/rust-toolchain` + cargo.
      No `src/`/`Cargo.toml`/`ci.yml`/`action.yml` change; no new dependency; nothing is
      actually published (the crate remains absent from crates.io — API still 404).
- [ ] Existing gates green (`cargo test`/`clippy`/`fmt`/`cargo publish --dry-run`).

## Failing Tests

Static + local checks (no in-repo way to unit-test a workflow; this satisfies
`test-before-implementation` by pre-specifying the assertions):

- **actionlint clean** — `actionlint .github/workflows/release.yml` exits 0.
- **publish job shape** — parse asserts: a `publish` job exists with
  `if: startsWith(github.ref, 'refs/tags/v')`, `needs` including `build`, a
  `cargo publish` invocation, and `CARGO_REGISTRY_TOKEN` referenced via `secrets.` (grep
  finds no literal token). The version-guard step is present.
- **dispatch does not publish** — reasoning + grep: the `publish` job's `if` is the same
  tag guard as `release`, so a `workflow_dispatch` (branch ref) skips it. State this
  explicitly.
- **dry-run still green** — `cargo publish --dry-run` exits 0 (the crate is still
  packageable; this is the closest executable proxy for the real publish).
- **not published** — `curl -s -o /dev/null -w '%{http_code}' -H 'User-Agent: x'
  https://crates.io/api/v1/crates/skillport` returns `404` (nothing was published by
  this spec).
- **RELEASING.md present** — the file exists and contains the token-secret setup, the
  first-publish step, and the tag-driven flow (grep for the key phrases).
- **contract untouched** — `git diff main -- src/ Cargo.toml Cargo.lock
  .github/workflows/ci.yml action.yml` is empty.

## Implementation Context

*Read this section before starting the build cycle.*

### Decisions that apply

- `DEC-009` — Attack-plan step 3. The `cargo publish` **execution** is human-only (token
  + tag). This spec adds the automation + runbook; it must **not** publish, add a token,
  or push a tag. Do not touch the Action (step 4 = SPEC-016) or bump the version /
  CHANGELOG (step 5 = SPEC-017).
- `DEC-005` — contract frozen; publishing is packaging. No `src/` change.

### Constraints that apply

- `license-policy` — the crate is dual MIT/Apache (SPEC-013), policy-clean; no new dep.
- `deterministic-stable-output` — no behavior/output change.
- `test-before-implementation` — the static + dry-run checks above are the pre-written
  verification.

### Prior related work

- `SPEC-014` (shipped, PR #14) — `release.yml` with `version`/`build`/`release` jobs;
  this adds a sibling `publish` job with the same tag guard and reuses
  `needs.version.outputs.version`.
- `SPEC-013` (shipped, PR #13) — crates.io metadata + the `cargo publish --dry-run` CI
  guard; the crate is already packageable.

### Out of scope (for this spec specifically)

- **No real `cargo publish`, no crates.io token, no tag push** (all human-only).
- **No `action.yml` change** (SPEC-016), **no version bump / CHANGELOG** (SPEC-017),
  **no `ci.yml`/`src/`/`Cargo.toml` change**.
- No auto-publish of docs, no crates.io badge wiring beyond what SPEC-017's README does.

## Notes for the Implementer

- Put the `publish` job in `release.yml` (one release pipeline), not a new workflow file.
  Guard it exactly like the `release` job so a `workflow_dispatch` never publishes.
- Version-match guard example: derive `cargo_ver` from `Cargo.toml` (awk, as the
  `version` job does) and compare to `needs.version.outputs.version`; `exit 1` with a
  clear message on mismatch. This stops a `v0.2.0` tag from publishing a `0.1.0` crate.
- `cargo publish --locked` (respect `Cargo.lock`). Do **not** add `--allow-dirty`.
- Keep `RELEASING.md` concise and skimmable — a numbered runbook, human-only steps
  clearly marked. It's the doc the human follows to actually ship v0.1.0 (SPEC-017).
- Do not weaken or remove the CI `cargo publish --dry-run` guard from SPEC-013.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-015-crates-publish`
- **PR (if applicable):** none yet (subagent build; not opened per instructions)
- **All acceptance criteria met?** yes
- **New decisions emitted:**
  - none
- **Deviations from spec:**
  - none
- **Follow-up work identified:**
  - none beyond SPEC-016 (Action swap) and SPEC-017 (version bump/CHANGELOG/tag,
    which is when RELEASING.md's per-release flow gets exercised for real)

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — Nothing; the spec's Notes for the Implementer (version-guard example, `--locked`
   not `--allow-dirty`, job placement) mapped directly onto the diff with no
   ambiguity.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — `cargo publish --dry-run` fails on *any* uncommitted working-tree changes
   (repo-wide git-dirty check, not scoped to packaged files), so the dry-run gate
   can only go green after this spec's own changes are committed. Worth noting
   explicitly next time a spec's Failing Tests include `cargo publish --dry-run`.

3. **If you did this task again, what would you do differently?**
   — Nothing material; would do it the same way.

---

## Reflection (Ship)

*Appended during the **ship** cycle. Outcome-focused reflection, distinct
from the process-focused build reflection above.*

1. **What would I do differently next time?**
   — <answer>

2. **Does any template, constraint, or decision need updating?**
   — <answer — if yes but not done this session, record it in
   `/guidance/signals.yaml`: `type: lesson` (with its N-count) for a recurring
   coding pattern, `type: process-debt` for tooling/process friction. A close
   then forces the decision. See `docs/signals.md`.>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>

4. **Where was the worst defect caught?** — one word from a fixed vocabulary so
   the defect-escape distribution is greppable across specs:
   `design` | `build` | `verify` | `ship` | `escaped` (reached prod/runtime) |
   `none` (clean first try).
   — <one word>
   *(Runtime/operational defects — the escape-prone class — only exist once the
   artifact meets its real host. `escaped` here is a signal to strengthen the
   §12 behavioral pre-flight for that surface.)*
