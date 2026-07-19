# SPEC-016 — BUILD prompt (Sonnet subagent)

You are the **implementer** for `SPEC-016: Action downloads the release binary (with
source fallback)`. You run as a metered subagent on branch
`feat/spec-016-action-download`, already created and checked out — **commit to the
current branch; do not create/switch branches, open a PR, or merge.** The spec is your
source of truth.

> **Change the install mechanism only.** Do NOT change the Action's `lint`/SARIF steps or
> its `path`/`strict`/`upload-sarif` inputs, do NOT touch `src/`, `Cargo.toml`,
> `Cargo.lock`, `.github/workflows/ci.yml`, or `.github/workflows/release.yml` (DEC-005).
> Do NOT cut a version, tag, or publish. The deliverable is `scripts/install-release.sh`
> (new) + `action.yml` + the README "Use in CI" note (+ spec bookkeeping).

## Read first (in order)

1. `projects/PROJ-001-skillport-lint/specs/SPEC-016-action-downloads-release-binary-with-source-fallback.md`
   — the platform-map table, Outputs, Acceptance Criteria, Failing Tests, Notes, Out of scope.
2. `.github/workflows/release.yml` (SPEC-014 — the EXACT archive names
   `skillport-<ver>-<triple>.<ext>` and the staged-dir internal layout
   `skillport-<ver>-<triple>/<binary>` your script must match), `action.yml` (current),
   the `README.md` "Use in CI" section, `decisions/DEC-009`.

## Your job

1. **Create `scripts/install-release.sh`** (bash, `set -euo pipefail`, runs under
   `shell: bash` on Linux/macOS/Windows runners):
   - Map `$RUNNER_OS`+`$RUNNER_ARCH` → triple/ext/binary per the spec table (Linux X64 →
     `x86_64-unknown-linux-gnu`/tar.gz/`skillport`; Linux ARM64 →
     `aarch64-unknown-linux-musl`; macOS X64/ARM64 → `x86_64`/`aarch64-apple-darwin`;
     Windows X64 → `x86_64-pc-windows-msvc`/zip/`skillport.exe`). Unknown pair →
     signal fallback (don't error).
   - Resolve version: `latest` → GitHub API `releases/latest` `.tag_name`; else `v<version>`.
   - Download `…/releases/download/<tag>/skillport-<ver>-<triple>.<ext>` + `.sha256`,
     verify checksum, extract (`tar xzf`/`unzip`), binary at
     `skillport-<ver>-<triple>/<binary>`; move to an install dir, append it to `$GITHUB_PATH`.
   - **Fallback signal, never hard-fail on a recoverable miss** (unsupported platform /
     no release / asset 404 / checksum fail): echo `installed=false` to `$GITHUB_OUTPUT` +
     a clear log line, exit 0. On success echo `installed=true`. The script must NOT run
     `cargo install` itself.
   - **`--print-plan`**: print `os/arch/triple/ext/version/asset/url` (and
     `supported=true|false`) as `key=value` lines and exit 0 with NO network — for tests.
2. **Edit `action.yml`:** add a `version` input (default `"latest"`). Replace the current
   `Ensure Rust toolchain` + `Install skillport` steps with:
   - `Install skillport (prebuilt)` — `id: prebuilt`, `shell: bash`, runs
     `"$GITHUB_ACTION_PATH/scripts/install-release.sh"` (pass the `version` input;
     `RUNNER_OS`/`RUNNER_ARCH` are in the env).
   - `Ensure Rust toolchain (fallback)` — `uses: dtolnay/rust-toolchain@stable`,
     `if: steps.prebuilt.outputs.installed != 'true'`.
   - `Install skillport from source (fallback)` — `shell: bash`,
     `if: steps.prebuilt.outputs.installed != 'true'`, the existing
     `cargo install --git https://github.com/jysf/skillport skillport --locked`.
   - Leave `Run skillport lint` + `Upload SARIF` and the other inputs unchanged.
3. **Update `README.md`** "Use in CI": the Action now downloads a prebuilt binary for the
   runner's platform (fast), falling back to a from-source build when no release/asset
   exists (e.g. before v0.1.0) or on an unsupported platform; document the `version` input.

## Definition of done

- Every **Acceptance Criterion** met; every **Failing Test** passes. Run and PASTE:
  `shellcheck scripts/install-release.sh` (exit 0); `actionlint action.yml` (exit 0); the
  `--print-plan` output for each supported pair + one unsupported pair (assert the triple/
  ext/binary/url); and a real no-`--print-plan` run on this host showing `installed=false`
  (the release doesn't exist yet) WITHOUT attempting `cargo install`.
- `git diff main -- src/ Cargo.toml Cargo.lock .github/workflows/ci.yml
  .github/workflows/release.yml` is EMPTY. Existing `cargo test`/`clippy`/`fmt`/`cargo
  publish --dry-run` still pass.
- Fill the spec's **## Build Completion**, append a **build** cost session (null numerics,
  per `projects/_templates/prompts/cost-snippet.md`), set `agents.implementer` to your
  model, commit to `feat/spec-016-action-download` (`feat(SPEC-016): …`). Do **not**
  advance cycle, PR, merge, tag, or publish.

## Return (final message = data for the orchestrator)

Concise + factual: PASTE `scripts/install-release.sh`, the new `action.yml` install steps,
and the README "Use in CI" diff; the `shellcheck` + `actionlint` results; the
`--print-plan` outputs (each supported pair + one unsupported) and the real fallback run
(`installed=false`, no cargo install attempted); confirm the asset-name/extract-path match
SPEC-014, `git diff main -- src/ Cargo.toml Cargo.lock ci.yml release.yml` empty, gates
green; any deviations/follow-ups.
