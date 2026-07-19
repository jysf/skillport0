---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-009
  type: story                      # epic | story | task | bug | chore
  cycle: design                    # frame | design | build | verify | ship
  blocked: false
  priority: medium
  complexity: M                    # S | M | L  (L means split it)

project:
  id: PROJ-001
  stage: STAGE-003
repo:
  id: skillport

agents:
  architect: claude-opus-4-8      # design cycle (this orchestrator session)
  implementer: claude-sonnet-5    # build runs as a Sonnet subagent (cost); updated with the real model
  created_at: 2026-07-18

references:
  decisions:
    - DEC-005   # lint's exit codes + SARIF are the contract CI depends on
  constraints:
    - deterministic-stable-output
    - license-policy
  related_specs:
    - SPEC-005  # the lint CLI + exit codes the CI relies on
    - SPEC-008  # --sarif output the Action uploads to code-scanning

value_link: "CI-ergonomics DX — a reusable GitHub Action that runs `skillport lint --sarif` + uploads to code-scanning, plus this repo's own Rust CI gates, so the tool is trivially adoptable and self-verified"

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
      notes: "main-loop, not separately metered (design cycle)"
    - cycle: build
      agent: claude-sonnet-5
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-07-18
      notes: "orchestrator fills real tokens_total/duration/estimated_usd from the Agent result at ship"
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-009: github action and rust ci

## Context

STAGE-003 DX: make skillport trivially adoptable in CI and self-verified. Two
gaps today: (1) there's no **reusable GitHub Action** for other repos to lint
their skills with, and (2) skillport's own **Rust gates don't run in CI** — the
template's `.github/workflows/ci.yml` only runs the `cost-audit` job and
explicitly invites the app's build/test/lint jobs. This spec ships both: a
composite Action that runs `skillport lint --sarif` and uploads to code-scanning,
and this repo's Rust CI (fmt / clippy / test) + a dogfood smoke check.

- Parent stage: `STAGE-003`, the "GitHub Action / CI workflow" backlog item.
- Reuses: the shipped CLI — `skillport lint <path> [--strict] --sarif` and its exit
  codes (SPEC-005) + SARIF output (SPEC-008).

> **Testability note:** GitHub Actions run on GitHub's runners, not locally, so
> this spec's "tests" are (a) **YAML validity** + **schema conformance** checks the
> build can run locally, and (b) confirming the **commands the workflows invoke
> actually work** by running them against the local binary/fixtures. The
> subagent must NOT claim a workflow "passed CI" — it can't trigger CI. Assert
> what's locally checkable and reason about the rest by inspection.

## Goal

Ship (1) a reusable composite **GitHub Action** (`action.yml`) that runs
`skillport lint --sarif` on a path and uploads the SARIF to code-scanning, and
(2) this repo's **CI**: Rust gates (fmt-check, clippy `-D warnings`, test) plus a
dogfood job that lints the good fixtures — with a documented usage snippet.

## Inputs

- **Files to read:** `.github/workflows/ci.yml` (extend, don't clobber the
  `cost-data` job), `README.md` (add a CI usage section), `app.just` (the gate
  commands to mirror), `docs/api-contract.md` (exit codes the CI relies on),
  `docs/license-policy.md` (for the optional cargo-deny job).
- **External:** GitHub Actions marketplace actions — `actions/checkout@v4`,
  a Rust toolchain action (e.g. `dtolnay/rust-toolchain@stable`),
  `github/codeql-action/upload-sarif@v3` (SARIF upload).

## Outputs

- **Files created:**
  - `action.yml` (repo root) — a **composite** Action. Inputs: `path` (default
    `.`), `strict` (default `false`), `upload-sarif` (default `true`). Steps:
    ensure a Rust toolchain; install skillport (`cargo install --path .` when run
    inside this repo, or document `cargo install --git` for external consumers —
    see Notes); run `skillport lint <path> [--strict] --sarif > skillport.sarif`
    (capture the exit code so the job fails on findings/`--strict`); if
    `upload-sarif`, `github/codeql-action/upload-sarif@v3` with the sarif file;
    surface the lint exit code as the step's status.
  - `.github/workflows/example-usage.yml` (or a documented snippet in README) —
    a minimal consumer workflow showing `uses: jysf/skillport@v0 with: path: skills`.
- **Files modified:**
  - `.github/workflows/ci.yml` — **add** jobs alongside `cost-data`:
    - `rust`: checkout → Rust toolchain → `cargo fmt --check` → `cargo clippy
      --all-targets -- -D warnings` → `cargo test`.
    - `dogfood`: checkout → build → run `skillport lint lint-fixtures/good`
      (expect exit 0) as a smoke test that the binary lints cleanly. (Do **not**
      lint `lint-fixtures/bad` in a gating job — it's intentionally full of errors.)
    - *(optional, if trivial)* `license`: `cargo-deny check licenses` with a
      minimal `deny.toml` (permissive-only) — satisfies the `license-policy`
      constraint. If not trivial, leave a follow-up note instead.
  - `README.md` — a "Use in CI" section with the `uses:` snippet + a note that
    findings appear in code-scanning.
- **No Rust source changes**, no new crate dependency (cargo-deny is a CI tool,
  not a crate dep). **Database changes:** none.

## Acceptance Criteria

- [x] `action.yml` exists at repo root, is **valid YAML**, and conforms to the
      composite-action schema (`runs.using: "composite"`, `runs.steps` with
      `shell` on `run` steps, declared `inputs`). It runs `skillport lint` with the
      `path`/`strict` inputs, produces SARIF, and (when `upload-sarif`) uploads via
      `github/codeql-action/upload-sarif`.
- [x] The Action's lint step **fails the job** when `skillport lint` returns
      non-zero (findings, or warnings under `strict`) — i.e. the exit code is
      propagated, not swallowed (the upload step should still run, e.g. via
      `if: always()`, but the job result reflects the lint exit).
- [x] `.github/workflows/ci.yml` keeps the existing `cost-data` job and **adds**
      `rust` (fmt-check + clippy `-D warnings` + test) and `dogfood` (lints
      `lint-fixtures/good`, expects exit 0). All jobs are valid YAML and reference
      real, pinned action versions.
- [x] The **commands** the workflows run are correct and verified against the local
      binary: `skillport lint lint-fixtures/good` exits 0; `skillport lint
      lint-fixtures/bad` exits 1; `--sarif` produces valid SARIF (already shipped —
      just confirm the invocation strings match).
- [x] `README.md` documents the Action usage (`uses:` snippet) and that findings
      surface in code-scanning.
- [x] Every YAML file parses; every referenced action is a real action pinned to a
      major version; no secret is required beyond the default `GITHUB_TOKEN` (with
      `permissions: security-events: write` for the SARIF upload).
- [x] No Rust source change; no new crate dependency; existing `cargo test` still
      green.

## Failing Tests

GitHub Actions can't run locally, so "tests" here are **local validity + command
checks**. Prefer a small script and/or notes the verifier can re-run.

- **YAML validity** — every file under `.github/` and `action.yml` parses:
  `python3 -c "import yaml,sys; [yaml.safe_load(open(p)) for p in sys.argv[1:]]"
  action.yml .github/workflows/*.yml` exits 0. (If `actionlint` is available, run
  it and it passes; if not, note that.)
- **Composite-action schema** — `action.yml` has `runs.using == "composite"`, a
  non-empty `runs.steps`, each `run` step has a `shell`, and declares the `path` /
  `strict` / `upload-sarif` inputs (assert via a `yaml.safe_load` check).
- **Command correctness (against the local binary)** — a check that mirrors what
  the workflows do: `cargo build` then `./target/debug/skillport lint
  lint-fixtures/good; test $? -eq 0` and `... lint-fixtures/bad; test $? -eq 1`,
  and `... lint-fixtures/bad --sarif | python3 -m json.tool >/dev/null`.
- **`ci.yml` shape** — `yaml.safe_load` shows jobs `cost-data`, `rust`, `dogfood`
  (+ `license` if included); the `rust` job's steps include `fmt`, `clippy`
  (`-D warnings`), and `test` invocations.
- (No Rust unit tests — this spec adds no Rust code. `cargo test` must stay green.)

## Implementation Context

### Decisions that apply

- `DEC-005` — the CI relies on `lint`'s **exit codes** (0/1/2) and **SARIF** being
  stable; the Action gates on the exit code and uploads the SARIF. Don't reinvent
  either — consume the shipped contract.

### Constraints that apply

- `license-policy` (advisory) — the optional `license` job + `deny.toml`
  (permissive-only: MIT/Apache-2.0/BSD/ISC/Zlib/Unicode/BSL-1.0) mechanizes it.
  Include if trivial; else record a follow-up. See `docs/license-policy.md`.
- `deterministic-stable-output` — not code here, but keep the workflows simple and
  pinned so runs are reproducible.

### Prior related work

- `SPEC-005` (exit codes), `SPEC-008` (`--sarif`) — the Action wraps these.
- The template `.github/workflows/ci.yml` already runs `cost-audit` via the
  `cost-data` job — **extend** it; don't remove that job.

### Out of scope (for this spec specifically)

- `--target claude` / per-platform verification — a separate STAGE-003 spec.
- Real-tokenizer `body.size` — a separate STAGE-003 spec.
- Publishing to the GitHub Marketplace, release automation, `cargo install
  skillport` from crates.io (nothing published yet) — the Action can build from
  source / `cargo install --git` for now; releases are a later concern.
- The README rule-id/severity table (that's the DX/README spec) — this spec's
  README change is only the "Use in CI" section.

## Notes for the Implementer

- **Composite action install step:** inside *this* repo's CI the binary is built
  from the checked-out source; for *external* consumers the action needs to fetch
  skillport. Pragmatic v0: the action does `cargo install --git
  https://github.com/jysf/skillport skillport --locked` (works before any crates.io
  release), with a comment that a released binary/`cargo install skillport` will be
  faster once published. Cache/perf optimization is out of scope.
- **Exit-code propagation:** in the composite `run` step, don't `|| true` the lint
  — let a non-zero exit fail the step. Put the `upload-sarif` step *before* the
  failing gate, or give upload `if: always()`, so results upload even on findings.
- **Permissions:** the SARIF-upload job needs `permissions: security-events:
  write` (and `contents: read`). Document this in the usage snippet.
- **Pin actions** to a major version (`@v4`, `@v3`, `@stable`). Use
  `dtolnay/rust-toolchain@stable` (widely used) or `actions-rs` — pick one and be
  consistent.
- **Validate locally:** run the YAML-parse + composite-schema + command-correctness
  checks above before declaring done. Do NOT claim any workflow "passed CI".
- Keep `cargo test` green — this spec adds no Rust; if you touch nothing in `src/`,
  the suite is unchanged.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-009-action`
- **PR (if applicable):** none opened this cycle (build only, per prompt)
- **All acceptance criteria met?** yes
- **New decisions emitted:**
  - none
- **Deviations from spec:**
  - Included the optional `license` job (named `licenses` to match the plural
    job-name convention already used) using the exact `deny.toml` +
    `cargo-deny-action@v2` snippet given in `docs/license-policy.md` — it was
    trivial, so no follow-up needed. Added `deny.toml` at the repo root (a CI
    config file, not a crate dependency) to back it.
  - `.github/workflows/example-usage.yml` is `workflow_dispatch`-only rather
    than wired to `push`/`pull_request`: this repo has no `skills/` directory
    and `jysf/skillport@v0` is not a tagged release yet, so triggering it
    automatically would fail this repo's own CI for reasons unrelated to
    skillport's correctness. It still documents the exact consumer-facing
    `uses:` shape from the spec, and the README's "Use in CI" snippet is the
    primary documented usage.
  - `action.yml`'s lint step interpolates `${{ inputs.strict }}` into a shell
    `if` rather than passing `--strict` unconditionally, since `strict` is an
    Action input (string `"true"`/`"false"`), not a shell boolean.
- **Follow-up work identified:**
  - Swap `cargo install --git ... --locked` in `action.yml` for a released
    binary / `cargo install skillport` once skillport is published to
    crates.io (noted inline in `action.yml` and the README) — real speed win,
    out of scope here per the spec's "Out of scope" section.
  - Actually tag a `v0` release of this repo so `jysf/skillport@v0` resolves
    for external consumers (currently only correct once tagged).

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — Nothing major; the spec's Notes section pre-answered the two likely
   judgment calls (install strategy, exit-code propagation), so implementation
   was mostly transcription. The only real judgment call was how to keep
   `example-usage.yml` from breaking this repo's own CI (see Deviations).

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No — `license-policy` (advisory) and `DEC-005` were both flagged and
   both directly usable (the license-policy doc even ships a copy-pasteable
   `deny.toml` + CI job, which made "include if trivial" an easy yes).

3. **If you did this task again, what would you do differently?**
   — Nothing structural; would just note upfront (as I did here) that a
   literal "example workflow" file living inside `.github/workflows/` of the
   *producer* repo needs a non-firing trigger, since it isn't actually
   runnable in this repo's own context.

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
