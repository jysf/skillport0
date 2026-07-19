# SPEC-014 — VERIFY prompt (Opus subagent)

You are an **independent verifier** for `SPEC-014: release workflow — cross-platform
binaries on tag`, run as a metered subagent. A separate Sonnet build session added
`.github/workflows/release.yml` and committed to branch `feat/spec-014-release-workflow`.
You did NOT build it. Disprove "done." **Do not modify code, merge, advance the cycle,
push a tag, create a Release, or run `cargo publish`** — return a verdict.

> **The claim:** a `v*` tag will produce a GitHub Release with prebuilt, checksummed
> binaries for the DEC-009 5-target matrix, and a `workflow_dispatch` exercises the
> build/archive/checksum path WITHOUT creating a Release — all with no contract change.
> The full matrix can't run here (that needs a GitHub tag push, human-only), so verify
> the workflow **statically + by the local archive proof**, and reason hard about the
> logic that can't be executed.

## Review the diff

```bash
git diff main...HEAD --stat
git diff main -- src/ Cargo.toml Cargo.lock .github/workflows/ci.yml action.yml   # MUST be empty
```

## Read

1. `projects/PROJ-001-skillport-lint/specs/SPEC-014-release-workflow-cross-platform-binaries-on-tag.md`
   — Outputs table (5 targets/runners/exts), Acceptance Criteria, Failing Tests, Out of scope.
2. `.github/workflows/release.yml` (the whole file), `.github/workflows/ci.yml` (pattern
   reference), `scripts/build-info.sh`, `decisions/DEC-009`, `DEC-005`.

## Verify — run it, don't trust the report

```bash
actionlint .github/workflows/release.yml ; echo actionlint=$?   # install if needed; exit 0
cargo test ; cargo clippy --all-targets -- -D warnings ; cargo fmt --check
cargo publish --dry-run --allow-dirty ; echo dryrun=$?
# local archive round-trip on THIS host (prove the per-leg commands are real):
cargo build --release --locked
# then strip, tar czf skillport-<ver>-<hosttriple>.tar.gz (binary + README.md + LICENSE-MIT + LICENSE-APACHE),
# sha256sum/shasum -a 256 > .sha256, and verify -c → OK
```

Adversarially check each (concrete observed/expected):

- **actionlint clean** on the committed file (exit 0). If you can't install actionlint,
  say so and do a strict YAML parse + manual review.
- **Triggers:** `on:` has BOTH `push: tags: ['v*']` and `workflow_dispatch`.
- **Matrix = exactly the 5 targets** with the right runners/exts/binary names:
  `aarch64-apple-darwin`+`x86_64-apple-darwin` (macos-14, tar.gz, `skillport`),
  `x86_64-unknown-linux-gnu`+`aarch64-unknown-linux-musl` (ubuntu-latest, tar.gz,
  `skillport`), `x86_64-pc-windows-msvc` (windows-latest, zip, `skillport.exe`). No
  missing/extra leg.
- **Release job is tag-gated:** `if: startsWith(github.ref, 'refs/tags/v')`, `needs`
  the build, `permissions: contents: write`. **Reason about `workflow_dispatch`:** on a
  manual dispatch `github.ref` is a branch ref (`refs/heads/…`), so the release job is
  SKIPPED and no Release is created — confirm the guard actually achieves that (a
  dispatch must not publish anything).
- **`gh`-only release, no third-party action:** the release step uses `gh release
  create/upload`; grep confirms NO `softprops/action-gh-release` / `taiki-e/*`. Only
  first-party actions (`checkout`, `upload-artifact`, `download-artifact`) +
  `dtolnay/rust-toolchain` + `gh`.
- **Archives + checksums:** each leg archives `skillport-<version>-<triple>.<ext>` with
  the binary + `README.md` + `LICENSE-MIT` + `LICENSE-APACHE`, writes a `.sha256`, and
  uploads it. The `release` job attaches all archives + `.sha256` + a `build-info.txt`
  (from `scripts/build-info.sh`). Version derived deterministically (tag→strip `v`;
  dispatch→Cargo.toml) in one place, not hand-typed per leg.
- **musl cross-compile is real:** the `aarch64-unknown-linux-musl` leg uses a genuine
  cross path (`cross` or a musl linker) — not a plain `cargo build` that would fail on a
  gnu host. Sanity-check the strip approach for it (`RUSTFLAGS=-Cstrip=symbols` is
  acceptable).
- **No scope creep (DEC-005):** `git diff main -- src/ Cargo.toml Cargo.lock ci.yml
  action.yml` EMPTY. No new Cargo dependency. The workflow does NOT trigger on push/PR
  (so it won't run on every commit). No tag pushed, no Release created by you.
- **Local archive proof** round-trips (sha256 `-c` → OK) on your host.

## Return a verdict (final message = data for the orchestrator)

**✅ APPROVED** / **⚠ PUNCH LIST** (numbered, file:line + concrete issue) /
**❌ REJECTED** (which criterion, observed vs expected). Include the actionlint result,
gate results, your local archive round-trip output, the matrix/trigger/release-guard
checks, the `git diff main -- src/ … action.yml` empty confirmation, and a per-AC
pass/fail summary. Call out explicitly whether a `workflow_dispatch` can create a
Release (it must NOT) and whether the musl leg is a real cross-compile. Note that the
full 5-leg matrix is only confirmable at the first `workflow_dispatch`/tag (human) — say
what you could and couldn't execute. Don't touch code, tag, or publish.
