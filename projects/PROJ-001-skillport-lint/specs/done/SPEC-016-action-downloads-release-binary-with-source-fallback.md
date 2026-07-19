---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-016
  type: story                      # epic | story | task | bug | chore
  cycle: ship  # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: M                    # S | M | L  (L means split it)

project:
  id: PROJ-001
  stage: STAGE-004
repo:
  id: skillport

agents:
  architect: claude-opus-4-8      # design cycle (this orchestrator session)
  implementer: claude-sonnet-5    # build runs as a Sonnet subagent (cost); updated with the real model
  created_at: 2026-07-19

references:
  decisions:
    - DEC-009   # distribution strategy — Attack-plan step 4 (Action downloads the release binary)
    - DEC-005   # frozen CLI/JSON/exit-code contract — the Action's lint invocation is unchanged
  constraints:
    - deterministic-stable-output
    - test-before-implementation
    - license-policy
  related_specs:
    - SPEC-014  # release.yml archive naming (skillport-<ver>-<triple>.<ext>) this must match
    - SPEC-009  # the Action being sped up (currently cargo install --git)
    - SPEC-015  # release pipeline (publish) — same release these binaries come from

value_link: "makes the shipped GitHub Action fast: it downloads the prebuilt release binary for the runner's OS/arch (verifying its sha256) instead of compiling from source on every run, with a source fallback so it keeps working before v0.1.0 exists and on unsupported platforms."

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
      recorded_at: 2026-07-19
      notes: "main-loop, not separately metered (design cycle); probed action.yml (current cargo-install-from-source), the SPEC-014 archive naming/internal layout, and the scripts/ convention"
    - cycle: build
      agent: claude-sonnet-5
      interface: claude-code
      tokens_total: 89323
      estimated_usd: 0.59
      duration_minutes: 25
      recorded_at: 2026-07-19
      notes: "metered Sonnet build subagent; tokens_total = subagent_tokens. estimated_usd = tokens x repo rate 6.60. duration wall-clock. scripts/install-release.sh + action.yml prebuilt step + README; shellcheck clean, --print-plan + real fallback verified."
    - cycle: verify
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: 70399
      estimated_usd: 0.46
      duration_minutes: 9
      recorded_at: 2026-07-19
      notes: "metered Opus verify subagent; ran shellcheck + all gates + --print-plan for every platform pair + a real fallback run, cross-checked asset naming vs SPEC-014, confirmed the happy path runs no Rust step. APPROVED, 0 punch-list."
    - cycle: ship
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-07-19
      notes: "main-loop, not separately metered (ship cycle)"
  totals:
    tokens_total: 159722
    estimated_usd: 1.05
    session_count: 4
shipped_at: 2026-07-19
---

# SPEC-016: Action downloads the release binary (with source fallback)

## Context

STAGE-004 Attack-plan step 4 (DEC-009). The shipped GitHub Action (SPEC-009) installs
skillport by compiling from source (`cargo install --git … --locked`) on **every**
consumer run — minutes per run. SPEC-014 now produces prebuilt, checksummed binaries per
release. This spec makes the Action **download** the right prebuilt binary for the
runner's OS/arch and verify its checksum, falling back to the from-source install when a
binary isn't available — so it stays green **before** the first release exists and on
unsupported platforms.

The download-success path can only be truly exercised once a release with assets exists
(the first `v0.1.0`, SPEC-017 — human-only). Until then every run legitimately takes the
**fallback**. So this spec is built to be verifiable now: the install logic lives in a
`scripts/install-release.sh` with a **`--print-plan` dry mode** that prints the resolved
platform → triple → asset URL without downloading, and a real run on a host with no
release simply reports "fall back" — both testable without a published binary.

## Goal

Replace the Action's from-source install with: download the prebuilt
`skillport-<version>-<triple>.<ext>` asset for the runner's OS/arch from the GitHub
Release, verify its `.sha256`, extract the binary onto `PATH`; and **fall back** to
`cargo install --git … --locked` (with a toolchain step that runs only then) when the
download can't succeed. Add a `version` input (default `latest`). No change to the
Action's `lint`/SARIF behavior; no `src/`/`Cargo.toml` change.

## Inputs

- **Files to create:** `scripts/install-release.sh` (the platform-mapping + download +
  checksum + extract + fallback-signal logic).
- **Files to modify:** `action.yml` (add `version` input; replace the install steps with
  a prebuilt-download step + a gated fallback), `README.md` (the "Use in CI" note that
  currently says "v0 builds skillport from source").
- **Files to read:** `.github/workflows/release.yml` (SPEC-014 — the **exact** archive
  names `skillport-<ver>-<triple>.<ext>` and their internal layout: a staged directory
  `skillport-<ver>-<triple>/` containing the binary + `README.md` + both LICENSE files),
  `decisions/DEC-009`.
- **No `src/`/`Cargo.toml`/`Cargo.lock` change. No new Cargo dependency.**

## Outputs

- **`scripts/install-release.sh`** (bash, `set -euo pipefail`, POSIX-ish, runs on GitHub
  Linux/macOS/Windows runners under `shell: bash`):
  - **Platform map** from `$RUNNER_OS` + `$RUNNER_ARCH` to the SPEC-014 target triple,
    archive ext, and binary name — exactly:

    | RUNNER_OS | RUNNER_ARCH | triple | ext | binary |
    |---|---|---|---|---|
    | Linux | X64 | `x86_64-unknown-linux-gnu` | `tar.gz` | `skillport` |
    | Linux | ARM64 | `aarch64-unknown-linux-musl` | `tar.gz` | `skillport` |
    | macOS | X64 | `x86_64-apple-darwin` | `tar.gz` | `skillport` |
    | macOS | ARM64 | `aarch64-apple-darwin` | `tar.gz` | `skillport` |
    | Windows | X64 | `x86_64-pc-windows-msvc` | `zip` | `skillport.exe` |

    Any other OS/arch → **unsupported → signal fallback** (do not error the job).
  - **Version resolution:** input `latest` → resolve the tag via the GitHub API
    (`GET https://api.github.com/repos/jysf/skillport/releases/latest` → `.tag_name`);
    else use `v<version>`. The asset version is the tag minus the leading `v`.
  - **Download + verify + extract:** fetch
    `https://github.com/jysf/skillport/releases/download/<tag>/skillport-<ver>-<triple>.<ext>`
    and its `.sha256`; verify the checksum (`sha256sum -c` / `shasum -a 256 -c`); extract
    (`tar xzf` / `unzip`); the binary is at `skillport-<ver>-<triple>/<binary>`. Move it
    to an install dir and append that dir to `$GITHUB_PATH` (so later steps find
    `skillport`).
  - **Fallback signal (never hard-fail on a recoverable miss):** if the platform is
    unsupported, the version can't be resolved (no release yet → API 404/empty), the
    asset 404s, or the checksum fails → **do not error**; emit `installed=false` to
    `$GITHUB_OUTPUT` (and a clear log line) so the Action runs the source fallback. On
    success emit `installed=true`. A genuinely unexpected error (e.g. corrupt archive
    after a valid download) may hard-fail.
  - **`--print-plan` dry mode:** print the resolved `{os, arch, triple, ext, version,
    asset, url}` as `key=value` lines and exit 0 **without** any network/download — for
    tests. Respects `RUNNER_OS`/`RUNNER_ARCH`/a `--version` arg from the environment.
- **`action.yml`:**
  - Add input `version` (default `"latest"`, description: which skillport release to
    install).
  - Replace the current `Ensure Rust toolchain` + `Install skillport` steps with:
    1. `Install skillport (prebuilt)` — `shell: bash`, `id: prebuilt`, runs
       `"$GITHUB_ACTION_PATH/scripts/install-release.sh"` with `RUNNER_OS`/`RUNNER_ARCH`
       and the `version` input; sets `prebuilt.outputs.installed`.
    2. `Ensure Rust toolchain (fallback)` — `uses: dtolnay/rust-toolchain@stable`,
       `if: steps.prebuilt.outputs.installed != 'true'`.
    3. `Install skillport from source (fallback)` — `shell: bash`,
       `if: steps.prebuilt.outputs.installed != 'true'`, runs the existing
       `cargo install --git https://github.com/jysf/skillport skillport --locked`.
  - The `Run skillport lint` and `Upload SARIF` steps are **unchanged**.
- **`README.md`:** update the "Use in CI" note — the Action now downloads a prebuilt
  binary for the runner's platform (fast), falling back to a from-source build when no
  release/asset is available (e.g. before v0.1.0) or on an unsupported platform;
  document the `version` input (default `latest`).

## Acceptance Criteria

- [x] `scripts/install-release.sh` exists and is executable; `shellcheck` clean (or a
      documented, justified disable). `--print-plan` prints correct
      `triple`/`ext`/`binary`/`url` for each supported `(RUNNER_OS, RUNNER_ARCH)` in the
      table, and reports **unsupported** (→ fallback) for an unknown pair — all without
      network.
- [x] The asset URL/name the script builds **exactly matches** SPEC-014's
      `skillport-<version>-<triple>.<ext>` scheme, and the extract path matches the
      staged-directory layout (`skillport-<ver>-<triple>/<binary>`).
- [x] On a host where the release/asset does not exist (the current reality — crate/
      release absent), the script **signals fallback** (`installed=false`) and exits 0 —
      it does not hard-fail the job.
- [x] `action.yml` gains the `version` input, installs via the prebuilt step, and runs
      the Rust toolchain + `cargo install --git` steps **only** when
      `steps.prebuilt.outputs.installed != 'true'`. `actionlint` passes on `action.yml`.
      The `Run skillport lint` + SARIF steps are unchanged.
- [x] README "Use in CI" documents the download-with-fallback behavior + the `version`
      input; the stale "v0 builds skillport from source" note is corrected.
- [x] No `src/`/`Cargo.toml`/`Cargo.lock`/`ci.yml`/`release.yml` change; no new Cargo
      dependency; existing `cargo test`/`clippy`/`fmt`/`cargo publish --dry-run` gates
      pass; no `--json`/SARIF/exit-code/rule-id change (DEC-005).

## Failing Tests

Static + local-script checks (a composite action can't be unit-tested in-repo; this
satisfies `test-before-implementation`):

- **`--print-plan` mapping** — for each supported pair, run
  `RUNNER_OS=<os> RUNNER_ARCH=<arch> scripts/install-release.sh --print-plan --version 0.1.0`
  and assert the printed `triple`/`ext`/`binary`/`url` match the table (e.g. Linux/X64 →
  `x86_64-unknown-linux-gnu`, `tar.gz`, `skillport`, url ends
  `download/v0.1.0/skillport-0.1.0-x86_64-unknown-linux-gnu.tar.gz`). An unknown pair
  (e.g. `Linux`/`X86`) prints `supported=false`.
- **Fallback on missing release** — run the script for real (no `--print-plan`) with
  `version=latest` (or a bogus version) on this host; assert it exits 0 and reports
  `installed=false` / a fallback log line (the release genuinely doesn't exist yet). It
  must NOT attempt the multi-minute `cargo install` itself (that's the Action's fallback
  step, not the script's job).
- **actionlint** — `actionlint action.yml` (or the repo's action) exits 0.
- **shellcheck** — `shellcheck scripts/install-release.sh` exits 0.
- **URL scheme match** — a check that the script's constructed asset name equals
  `skillport-<ver>-<triple>.<ext>` for the SPEC-014 triples (guards against drift from
  the release workflow's naming).
- **Contract untouched** — `git diff main -- src/ Cargo.toml Cargo.lock
  .github/workflows/ci.yml .github/workflows/release.yml` is empty.

## Implementation Context

*Read this section before starting the build cycle.*

### Decisions that apply

- `DEC-009` — Attack-plan step 4 only: point the Action at the release binary. Do NOT
  cut a version / CHANGELOG / tag (step 5 = SPEC-017), do NOT publish, do NOT change the
  release workflow (SPEC-014) or the publish job (SPEC-015).
- `DEC-005` — the Action's `lint`/`--sarif` invocation and the CLI contract are frozen.
  Only the *install* mechanism changes.

### Constraints that apply

- `deterministic-stable-output` — the platform→asset mapping is a fixed table; the script
  is deterministic given `(RUNNER_OS, RUNNER_ARCH, version)`.
- `test-before-implementation` — the `--print-plan` + fallback checks above are the
  pre-written verification.
- `license-policy` — no new dependency; the downloaded archive already bundles both
  license files (SPEC-014).

### Prior related work

- `SPEC-014` (shipped, PR #14) — `release.yml`; the archive names
  `skillport-<ver>-<triple>.<ext>` and the staged-dir internal layout this script must
  match **exactly**. If they disagree, the script (this spec) is wrong, not the workflow.
- `SPEC-009` (shipped, PR #9) — the Action being sped up; keep its `lint`/SARIF steps and
  its inputs (`path`/`strict`/`upload-sarif`) intact, just add `version`.
- `SPEC-015` (shipped, PR #15) — the publish pipeline; these binaries come from the same
  `v*` release.

### Out of scope (for this spec specifically)

- **No version bump / CHANGELOG / tag / publish** (SPEC-017 / human-only).
- **No `release.yml`/`ci.yml`/`src/`/`Cargo.toml` change.**
- No new platform targets beyond SPEC-014's five (add more only if SPEC-014's matrix
  grows). No Homebrew/signing (deferred, DEC-009).
- No change to the Action's lint/SARIF behavior or its existing inputs.

## Notes for the Implementer

- **`$GITHUB_ACTION_PATH`** is where the action repo (jysf/skillport@ref) is checked out
  in a consumer run — reference the script as `"$GITHUB_ACTION_PATH/scripts/install-release.sh"`.
- **`$GITHUB_PATH`**: append the install dir (one path per line) so later steps see
  `skillport` on `PATH`. **`$GITHUB_OUTPUT`**: `echo "installed=true|false" >> "$GITHUB_OUTPUT"`.
- The public release assets download without auth (`curl -fsSL`); the `releases/latest`
  API also needs no token for a public repo (mind rate limits — acceptable). `curl -f`
  makes a 404 a non-zero exit you can catch to signal fallback.
- Windows runners provide `bash`, `curl`, `unzip`, `shasum`/`sha256sum` — keep the script
  bash + coreutils; branch the checksum/extract tool by ext/OS. The Windows binary is
  `skillport.exe` inside a `.zip`.
- Keep the fallback identical to today's behavior (`cargo install --git … --locked`) so
  nothing regresses for consumers pre-v0.1.0.
- Don't reintroduce a top-level `dtolnay/rust-toolchain` step on the happy path — that's
  the point of the speedup; it must be gated to the fallback only.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-016-action-download`
- **PR (if applicable):** none yet (build cycle only; not opened by this agent)
- **All acceptance criteria met?** yes
- **New decisions emitted:**
  - none
- **Deviations from spec:**
  - `actionlint action.yml` cannot be run as a bare CLI arg — actionlint 1.7.12
    parses any directly-passed YAML file as a *workflow* (requires `on`/`jobs`),
    not as a composite action, so `actionlint action.yml` alone always fails
    with "jobs section is missing" regardless of the action.yml's content.
    Verified instead via actionlint's documented local-action resolution: a
    temporary workflow with `uses: ./` (referencing this repo's action) was
    added, linted (0 issues), then removed — not part of the committed diff.
    This is a tooling-invocation gap in the spec's Failing Tests wording, not
    a defect in `action.yml`.
- **Follow-up work identified:**
  - none

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — Nothing about the deliverable itself; the only friction was `actionlint
   action.yml` not being a literal runnable command against this actionlint
   version (see Deviations) — the spec's Failing Tests line assumes direct
   top-level linting works, but actionlint only validates composite actions
   through a referencing workflow's `uses: ./`.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No — DEC-009/DEC-005 and the SPEC-014 archive/layout reference were
   sufficient to build this without ambiguity.

3. **If you did this task again, what would you do differently?**
   — Same approach; would just note the actionlint local-action-resolution
   trick up front instead of discovering it empirically.

---

## Reflection (Ship)

*Appended during the **ship** cycle. Outcome-focused reflection, distinct
from the process-focused build reflection above.*

1. **What would I do differently next time?**
   — Extracting the install logic into `scripts/install-release.sh` with a `--print-plan`
   dry mode was the move that made an otherwise-unverifiable Action (its happy path needs
   a real release that doesn't exist yet) fully testable now: the platform→asset mapping
   and URL construction are asserted per-pair with no network, and a real run exercises
   the fallback. This is the same shape as SPEC-014's `workflow_dispatch` dry path — pull
   the untestable-in-place logic into a script/flag with a no-side-effect mode.

2. **Does any template, constraint, or decision need updating?**
   — No. The `dry-trigger-for-privileged-automation` lesson (N=2) is adjacent but this is
   a slightly different flavor (a *dry inspection mode on a script* vs a *dry trigger on a
   pipeline*); both are instances of "give the untestable-in-place thing a safe
   observe-only mode." I'm folding this under that lesson's evidence rather than opening a
   near-duplicate — noted at its next stage-close walk. Also worth remembering: `actionlint`
   only lints workflow files, not composite `action.yml` — validate a composite action via
   a throwaway `uses: ./` workflow (both build and verify hit this).

3. **Is there a follow-up spec I should write now before I forget?**
   — One STAGE-004 spec remains: **SPEC-017 (cut v0.1.0)** — version bump (`just
   next-version`), CHANGELOG, README install matrix, then the **human-only `v0.1.0` tag
   push** that fires the whole pipeline (binaries + crates.io publish) end-to-end for the
   first time — which is also the first real exercise of SPEC-014's matrix and SPEC-016's
   download path. Per the user's steer, I pause before SPEC-017 (it needs their tag push
   and, ideally, the crates.io token setup first).

4. **Where was the worst defect caught?** — one word from a fixed vocabulary so
   the defect-escape distribution is greppable across specs:
   `design` | `build` | `verify` | `ship` | `escaped` (reached prod/runtime) |
   `none` (clean first try).
   — none
   *(Runtime/operational defects — the escape-prone class — only exist once the
   artifact meets its real host. `escaped` here is a signal to strengthen the
   §12 behavioral pre-flight for that surface.)*
