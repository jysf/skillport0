# SPEC-016 — VERIFY prompt (Opus subagent)

You are an **independent verifier** for `SPEC-016: Action downloads the release binary
(with source fallback)`, run as a metered subagent. A separate Sonnet build session added
`scripts/install-release.sh`, edited `action.yml`, and updated the README, committed to
branch `feat/spec-016-action-download`. You did NOT build it. Disprove "done." **Do not
modify code, merge, advance the cycle, tag, or publish** — return a verdict.

> **The claim:** the Action now downloads a prebuilt binary for the runner's OS/arch
> (checksum-verified) instead of compiling, and falls back to `cargo install --git` when
> no release/asset exists (the reality until v0.1.0) or on an unsupported platform — with
> the lint/SARIF behavior and the contract untouched. The download-SUCCESS path can't run
> (no release exists yet), so verify the mapping/URL logic + the fallback path + the
> action wiring, and reason about the rest.

## Review the diff

```bash
git diff main...HEAD --stat
git diff main -- src/ Cargo.toml Cargo.lock .github/workflows/ci.yml .github/workflows/release.yml   # MUST be empty
```

## Read

1. `projects/PROJ-001-skillport-lint/specs/SPEC-016-action-downloads-release-binary-with-source-fallback.md`
   — the platform-map table, Outputs, Acceptance Criteria, Failing Tests, Build
   Completion (note its actionlint-invocation deviation), Out of scope.
2. `scripts/install-release.sh`, `action.yml`, the README "Use in CI" section,
   `.github/workflows/release.yml` (the SPEC-014 archive names/layout to cross-check),
   `decisions/DEC-009`, `DEC-005`.

## Verify — run it, don't trust the report

```bash
shellcheck scripts/install-release.sh ; echo shellcheck=$?
cargo test ; cargo clippy --all-targets -- -D warnings ; cargo fmt --check ; cargo publish --dry-run
# print-plan mapping for every supported pair + one unsupported:
for pair in "Linux X64" "Linux ARM64" "macOS X64" "macOS ARM64" "Windows X64" "Linux X86"; do
  set -- $pair; RUNNER_OS=$1 RUNNER_ARCH=$2 bash scripts/install-release.sh --print-plan --version 0.1.0; echo ---
done
# real fallback (no release exists → must exit 0, installed=false, NO cargo install):
tmp=$(mktemp); RUNNER_OS=Linux RUNNER_ARCH=X64 GITHUB_OUTPUT=$tmp bash scripts/install-release.sh --version 0.1.0 ; echo exit=$? ; cat "$tmp"
```
For **actionlint on a composite action**: `actionlint` only lints workflow files, not
`action.yml`. Validate `action.yml` by creating a throwaway `.github/workflows/_tmp.yml`
with `jobs.t.steps: [{uses: ./}]`, run `actionlint`, confirm 0 issues, then delete it
(don't commit it). Or do a strict YAML parse + manual step review. Say which you did.

Adversarially check, each with concrete observed/expected:

- **Platform map is EXACT** (cross-check against `release.yml`'s matrix): Linux/X64 →
  `x86_64-unknown-linux-gnu`; Linux/ARM64 → `aarch64-unknown-linux-musl`; macOS/X64 →
  `x86_64-apple-darwin`; macOS/ARM64 → `aarch64-apple-darwin`; Windows/X64 →
  `x86_64-pc-windows-msvc` (`.zip`, `skillport.exe`). Unknown pair → `supported=false`.
- **Asset name + extract path match SPEC-014:** the script builds
  `skillport-<ver>-<triple>.<ext>` and extracts the binary from
  `skillport-<ver>-<triple>/<binary>` — identical to `release.yml`'s `stage=` naming and
  archive layout. A mismatch here means a real release would fail to install.
- **Fallback never hard-fails a recoverable miss:** on the current host (no release), the
  script exits 0 and emits `installed=false` (unsupported platform, unresolved tag, asset
  404, or checksum fail all → fallback, not error). It must NOT run `cargo install`
  itself. Confirm a genuinely-unexpected error MAY still hard-fail (that's fine).
- **action.yml wiring:** a `version` input (default `latest`); the `prebuilt` step runs
  the script and sets `installed`; the `dtolnay/rust-toolchain` and `cargo install --git`
  steps are BOTH gated `if: steps.prebuilt.outputs.installed != 'true'` (so the happy path
  runs neither — the speedup); the `Run skillport lint` + SARIF steps and the
  `path`/`strict`/`upload-sarif` inputs are unchanged.
- **No scope creep (DEC-005):** `git diff main -- src/ Cargo.toml Cargo.lock ci.yml
  release.yml` EMPTY; no new Cargo dependency; no `--json`/SARIF/exit-code/rule-id change.
- **README** documents the download-with-fallback behavior + the `version` input; the
  stale "v0 builds skillport from source" line is gone.
- Gates green; `shellcheck` clean.

## Return a verdict (final message = data for the orchestrator)

**✅ APPROVED** / **⚠ PUNCH LIST** (numbered, file:line + concrete issue) /
**❌ REJECTED** (which criterion, observed vs expected). Include the shellcheck result,
gate results, the `--print-plan` outputs you observed (all pairs), the real fallback run
(exit 0 + `installed=false` + no cargo install), how you validated `action.yml`, the
SPEC-014 asset/extract cross-check, the empty `git diff main -- src/ … release.yml`
confirmation, and a per-AC pass/fail summary. State explicitly whether the happy path runs
any Rust step (it must not) and whether the asset naming matches SPEC-014. Note that the
download-success path is only confirmable once a real release exists (SPEC-017/human).
Don't touch code, tag, or publish.
