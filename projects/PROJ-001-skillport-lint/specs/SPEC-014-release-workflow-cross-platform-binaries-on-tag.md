---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-014
  type: story                      # epic | story | task | bug | chore
  cycle: build  # frame | design | build | verify | ship
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
  implementer: claude-sonnet-5    # build ran as a Sonnet subagent (cost)
  created_at: 2026-07-18

references:
  decisions:
    - DEC-009   # distribution strategy — this is Attack-plan step 2 (the release workflow)
    - DEC-005   # frozen CLI/JSON/exit-code contract — release is packaging, not behavior
  constraints:
    - deterministic-stable-output   # no behavior/schema change; reproducible artifact naming
    - license-policy                # no new runtime dep; workflow tooling stays policy-clean
    - test-before-implementation
  related_specs:
    - SPEC-013  # release Phase-0 prep (metadata/licenses this release ships)
    - SPEC-009  # the CI + Action patterns to mirror (dtolnay toolchain, first-party actions)

value_link: "infrastructure enabling STAGE-004's release: pushing a v* tag produces a GitHub Release carrying prebuilt, checksummed skillport binaries for the DEC-009 platform matrix — the foundation crates.io/Homebrew/the Action all lean on."

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
      notes: "main-loop, not separately metered (design cycle); includes probing build-info.sh (provenance stamp), the existing ci.yml action patterns, and the DEC-009 target matrix"
    - cycle: build
      agent: claude-sonnet-5
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-07-18
      notes: "metered subagent build; added .github/workflows/release.yml (5-target matrix + gh-based release job); orchestrator fills tokens_total/duration/estimated_usd from the Agent result at ship"
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-014: release workflow — cross-platform binaries on tag

## Context

STAGE-004 Attack-plan step 2 (DEC-009). SPEC-013 made the crate packageable; this
adds the machinery that turns a **`v*` tag** into a **GitHub Release** carrying prebuilt,
stripped, checksummed `skillport` binaries for the DEC-009 platform matrix. This is the
distribution foundation everything else leans on: crates.io is independent, but the
Action (SPEC-016) will download these binaries instead of building from source, and a
future Homebrew tap points at these tarballs.

The hard truth about verifying a release workflow: its true end-to-end behavior only
fires on a real `v*` tag push, which is **human-only** (DEC-009). So this spec is
designed to be **CI-exercisable without a release**: the workflow also runs on
`workflow_dispatch`, where it builds the whole matrix and uploads the archives as
**workflow artifacts** but does **not** create a GitHub Release. That makes the
build→strip→archive→checksum path testable on demand; only the final "attach to the
Release" step waits for the first real tag (SPEC-017).

## Goal

Add `.github/workflows/release.yml` that, on a `v*` tag, cross-compiles `skillport` for
the DEC-009 matrix (macOS arm64 + x86_64, Linux x86_64-gnu + aarch64-musl, Windows
x86_64), strips each binary, archives it (`.tar.gz` on unix, `.zip` on Windows),
computes a `.sha256`, and attaches all of them plus a build-provenance file to a GitHub
Release — with a `workflow_dispatch` path that does everything except create the Release
(uploads artifacts instead), so it is CI-testable without a tag. No runtime code change.

## Inputs

- **Files to create:** `.github/workflows/release.yml`.
- **Files to read:** `.github/workflows/ci.yml` (mirror its `actions/checkout@v4` +
  `dtolnay/rust-toolchain@stable` patterns), `scripts/build-info.sh` (the provenance
  stamp; `just build-info` / `--json`), `Cargo.toml` (crate name `skillport`, version),
  `decisions/DEC-009-distribution-strategy.md` (step 2 + the matrix), `action.yml`
  (the consumer that SPEC-016 will point at these artifacts — do NOT change it here).
- **No `src/` change. No new Cargo dependency.**

## Outputs

- **`.github/workflows/release.yml`:**
  - **Triggers:** `on: push: tags: ['v*']` **and** `on: workflow_dispatch` (manual).
  - **A `build` matrix job** over exactly these 5 targets, each producing one archive +
    one `.sha256`:

    | runner | target triple | archive | binary |
    |---|---|---|---|
    | `macos-14` | `aarch64-apple-darwin` | `.tar.gz` | `skillport` |
    | `macos-14` | `x86_64-apple-darwin` | `.tar.gz` | `skillport` |
    | `ubuntu-latest` | `x86_64-unknown-linux-gnu` | `.tar.gz` | `skillport` |
    | `ubuntu-latest` | `aarch64-unknown-linux-musl` | `.tar.gz` | `skillport` |
    | `windows-latest` | `x86_64-pc-windows-msvc` | `.zip` | `skillport.exe` |

    Steps per leg: checkout; install the Rust toolchain **with the matrix target**
    (`dtolnay/rust-toolchain@stable` `targets:` input); build `cargo build --release
    --locked --target <triple>`; **strip** the binary (native `strip` on the gnu/macos
    targets; for `aarch64-unknown-linux-musl` use a cross toolchain — `cross` (docker)
    or a musl cross-linker, the build's call — and its matching strip, or `--strip` via
    `RUSTFLAGS=-Cstrip=symbols` if a cross `strip` is awkward); archive as
    `skillport-<version>-<triple>.<ext>` containing the binary (+ `README.md`,
    `LICENSE-MIT`, `LICENSE-APACHE`); write `skillport-<version>-<triple>.<ext>.sha256`
    (a `sha256sum`-format line); upload both via `actions/upload-artifact@v4`.
    - `<version>` = the tag with the leading `v` stripped on a tag run; on
      `workflow_dispatch`, the `Cargo.toml` version (`0.1.0`) — derive it deterministically.
  - **A `release` job** that runs **only on a tag push** (`if: startsWith(github.ref,
    'refs/tags/v')`), `needs: build`: download all artifacts; generate a
    `build-info.txt` provenance file (`./scripts/build-info.sh` output — commit/ref/
    built_at); create-or-update the GitHub Release for the tag and upload every archive,
    every `.sha256`, and `build-info.txt`. **Use the `gh` CLI** (`gh release create` /
    `gh release upload`) with the default `GITHUB_TOKEN` — no third-party release action.
    Set `permissions: contents: write` on the job.
  - On `workflow_dispatch` the `release` job is skipped, so a manual run just produces
    downloadable workflow artifacts (the dry test path) and creates **no** Release.
- **No change to `ci.yml`, `action.yml`, `src/`, `Cargo.toml`** (version bump + CHANGELOG
  are SPEC-017; the Action swap is SPEC-016).

## Acceptance Criteria

- [x] `.github/workflows/release.yml` exists, is valid YAML, and passes `actionlint`
      (no errors). It triggers on `push` tags `v*` and on `workflow_dispatch`.
- [x] The `build` matrix lists exactly the 5 target triples above with the specified
      runners and archive extensions; each leg installs the Rust toolchain for its
      target, builds `--release --locked --target <triple>`, strips, archives as
      `skillport-<version>-<triple>.<ext>`, and writes a matching `.sha256`.
- [x] The archive for each leg contains the platform binary (`skillport` /
      `skillport.exe`) plus `README.md`, `LICENSE-MIT`, `LICENSE-APACHE`.
- [x] The `release` job runs **only** on a `v*` tag (`workflow_dispatch` skips it),
      `needs: build`, has `permissions: contents: write`, creates/updates the tag's
      GitHub Release via the `gh` CLI, and uploads all archives + `.sha256` files + a
      `build-info.txt` provenance file. No third-party release action; only
      first-party actions (`checkout`, `upload-artifact`, `download-artifact`) +
      `dtolnay/rust-toolchain` (already used) + `gh`.
- [x] Version is derived deterministically (`v*` tag → strip `v`; dispatch →
      `Cargo.toml` version) — asserted by a small helper/step, not hand-typed per leg.
- [x] **Local build-path proof** (the part verifiable without GitHub): on the host
      target, `cargo build --release --locked` then strip + `tar czf` + `sha256sum`
      reproduces a `skillport-<version>-<host-triple>.tar.gz` + `.sha256` whose checksum
      verifies. (Documents that the per-leg commands are real; the full matrix is
      confirmed at first `workflow_dispatch` / tag.)
- [x] No `src/`/`Cargo.toml`/`ci.yml`/`action.yml` change; no new dependency; the
      existing `cargo test`/`clippy`/`fmt`/`cargo publish --dry-run` gates still pass.

## Failing Tests

A GitHub Actions workflow can't be unit-tested in-repo; the checks are static +
local-command proofs (this satisfies `test-before-implementation` by pre-specifying the
exact assertions the build must make pass).

- **actionlint clean** — `actionlint .github/workflows/release.yml` exits 0 (install via
  `go install`/`brew`/the pinned binary if not present; if actionlint truly can't be
  obtained, fall back to a YAML-parse check + a documented manual review, and say so).
- **Triggers + matrix present** — grep/parse asserts: `on:` has `push: tags: ['v*']`
  and `workflow_dispatch`; the matrix `target` list is exactly the 5 triples; the
  `release` job has `if: startsWith(github.ref, 'refs/tags/v')` and
  `permissions: contents: write`.
- **No third-party release action** — the workflow contains no `softprops/action-gh-release`
  / `taiki-e/*` upload action; the release step uses `gh release`.
- **Local archive proof** — a scripted check builds `--release --locked` for the host
  triple, strips, `tar czf skillport-<ver>-<host>.tar.gz` (with the 3 doc/license files),
  `sha256sum > …sha256`, then `sha256sum -c …sha256` succeeds. Exit 0.
- **Contract untouched** — `git diff main -- src/ Cargo.toml Cargo.lock .github/workflows/ci.yml action.yml`
  is empty.

## Implementation Context

*Read this section before starting the build cycle.*

### Decisions that apply

- `DEC-009` — this is Attack-plan **step 2** only: the release workflow. Do **not** do
  step 3 (crates.io publish — SPEC-015), step 4 (Action swap — SPEC-016), or step 5
  (cut v0.1.0 / CHANGELOG / tag — SPEC-017). **Do not push a tag or create a real
  Release** in this spec — the workflow is added but first fired by the human later.
  Matrix is the full DEC-009 5-target set (confirmed with the user 2026-07-18).
- `DEC-005` — the CLI/JSON/exit-code/rule-id contract is frozen. Releasing is packaging;
  no `src/` change. The binary already self-reports its version via clap (`--version` =
  Cargo `0.1.0`); provenance ships as an attached `build-info.txt`, **not** embedded
  into the binary (that would be a code/build change — out of scope).

### Constraints that apply

- `license-policy` — no new Cargo runtime dep; `cross` (if used) is a build-time CI tool,
  not a crate dependency, and the shipped archives include both license files.
- `deterministic-stable-output` — artifact names are a stable scheme
  `skillport-<version>-<triple>.<ext>`; the version is derived, not hand-typed.
- `test-before-implementation` — the static + local-archive checks above are the
  pre-written verification.

### Prior related work

- `SPEC-013` (shipped, PR #13) — dual licenses + crates.io metadata + the
  `cargo publish --dry-run` CI guard; those license files are what the archives bundle.
- `SPEC-009` (shipped, PR #9) — the Action + CI; mirror its `actions/checkout@v4` +
  `dtolnay/rust-toolchain@stable` usage. The Action's `cargo install --git` line is
  swapped for a release-binary download in SPEC-016 (not here).
- `scripts/build-info.sh` — the provenance stamp (`just build-info`); use its output for
  `build-info.txt`.

### Out of scope (for this spec specifically)

- **No tag push, no real Release, no `cargo publish`** (all later / human-only).
- **No `action.yml` change** (SPEC-016), **no version bump / CHANGELOG** (SPEC-017),
  **no `ci.yml` change**, **no `src/`/`Cargo.toml` change**.
- **No binary-embedded build-info / no `build.rs`** — provenance is an attached file.
- **No Homebrew, no signing/notarization** (deferred — Apple key; DEC-009). macOS
  binaries are unsigned; that's expected and documented at the v0.1.0 cut (SPEC-017).

## Notes for the Implementer

- **The `aarch64-unknown-linux-musl` leg is the only real cross-compile.** Pick the
  simplest robust route and comment it: `cross` (`taiki-e/install-action` or
  `cargo install cross`, then `cross build --release --target …`) is the low-friction
  choice; a native musl cross-linker also works. If `strip` for the cross target is
  awkward, `RUSTFLAGS=-Cstrip=symbols` (or `[profile.release] strip` — but that edits
  Cargo.toml, which is out of scope, so prefer the env var in the workflow) strips at
  link time. macOS `x86_64` cross-compiles natively from the arm64 runner with just the
  added target. Windows + the two native Linux/mac legs are straightforward.
- **`gh release create` idempotency:** use `gh release create <tag> … || gh release
  upload <tag> … --clobber`, or `gh release create` with all files at once, so a re-run
  doesn't hard-fail. Draft vs published is the build's call; a published release on tag
  is fine (the human controls when the tag is pushed).
- **Pin action versions** to major (`@v4`) like `ci.yml` does. Keep third-party surface
  to `dtolnay/rust-toolchain` only (already trusted in this repo); everything else
  first-party + `gh`.
- **Don't gate `ci.yml` on this** — the release workflow is tag/dispatch-triggered and
  must not run on every push/PR.
- Verify locally with `actionlint` and the archive round-trip; note in Build Completion
  that the full 5-way matrix is confirmed at the first `workflow_dispatch` (which the
  human can trigger safely — it creates no Release).

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-014-release-workflow`
- **PR (if applicable):** none yet (build cycle only; not advanced to PR/merge)
- **All acceptance criteria met?** yes
- **New decisions emitted:**
  - none
- **Deviations from spec:**
  - Added a small `version` job (upstream of `build`/`release`) to hold the
    version-derivation step once instead of repeating it in each of the 5
    matrix legs, and exposed it via `needs: version` / job outputs. This is
    the "small helper/step, not hand-typed per leg" the AC asks for; it's a
    job rather than a composite step purely so every downstream job can read
    `needs.version.outputs.version` without recomputing it.
  - Windows archiving uses `7z` (preinstalled on `windows-latest` runners)
    instead of a PowerShell `Compress-Archive` call, since 7z's CLI syntax is
    closer to the unix `tar czf` step and keeps the archive step symmetric.
  - `strip` is skipped on Windows (MSVC binaries aren't stripped the same way
    via GNU `strip`; PDBs aren't shipped and the release binary is already
    built with the default release profile) — this matches "native `strip`
    on the gnu/macos targets" in the Notes, which doesn't list Windows.
- **Follow-up work identified:**
  - SPEC-016 (Action swap) and SPEC-017 (version bump/CHANGELOG/tag) remain
    as previously planned; no new spec needed from this build.

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — Nothing major; the Outputs table and Notes for the Implementer were
   detailed enough to write the workflow in one pass. The only judgment call
   was how to structure version derivation as "a step, not hand-typed per
   leg" — I read that as license to factor it into its own upstream job.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — The spec doesn't say how to archive on Windows (zip via PowerShell vs.
   `7z` vs. an action); I picked `7z` since it ships on `windows-latest` and
   needs no extra action, but a `windows-latest`-specific tool note in a
   future spec's Notes section would remove that judgment call.

3. **If you did this task again, what would you do differently?**
   — Same approach; I'd only add a comment inline (already done) explaining
   why `strip` doesn't run on Windows so a future reader doesn't think it was
   an oversight.

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
