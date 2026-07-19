# RELEASING

How to cut a skillport release: prebuilt binaries via GitHub Releases
(`.github/workflows/release.yml`, SPEC-014) and `cargo install skillport` via
crates.io (the `publish` job in the same workflow, SPEC-015). Distribution
strategy and rationale: `decisions/DEC-009`.

Steps marked **(human)** cannot be automated — they need a credential the
human holds or an irreversible action (per DEC-009).

## 1. One-time setup

Do this once, before the first release ever ships.

1. **(human)** Create a [crates.io](https://crates.io) account (GitHub OAuth
   is fine).
2. **(human)** Generate a scoped API token: crates.io → Account Settings →
   API Tokens → New Token. Scope it to `publish-update` (or `publish-new` +
   `publish-update`) for the `skillport` crate name if crates.io offers
   scoping at creation time.
3. **(human)** Add the token as a GitHub Actions secret named
   `CARGO_REGISTRY_TOKEN`: repo Settings → Secrets and variables → Actions →
   New repository secret. The `publish` job reads it as
   `${{ secrets.CARGO_REGISTRY_TOKEN }}` — never a literal in the workflow.
4. **(human)** Re-confirm the name `skillport` is still free:
   `cargo search skillport` or check
   `https://crates.io/api/v1/crates/skillport` returns 404 / the crates.io
   page 404s.
5. **(human)** Publish the first version manually to establish ownership,
   from a clean checkout at the release commit:
   ```
   cargo publish --locked
   ```
   After this first manual publish, the crate exists under your crates.io
   account and subsequent tag pushes auto-publish via the `publish` job.

## 2. Per-release flow

Repeat this for every release after the first.

1. Bump the version and update the CHANGELOG (SPEC-017 / `just
   next-version`).
2. Commit the bump.
3. Optional smoke test: trigger `release.yml` via `workflow_dispatch` (GitHub
   Actions UI or `gh workflow run release.yml`). This builds the binary
   matrix and uploads workflow artifacts but creates **no** GitHub Release
   and runs **no** publish — the `release` and `publish` jobs are both
   tag-gated (`if: startsWith(github.ref, 'refs/tags/v')`) and are skipped on
   a `workflow_dispatch` ref.
4. **(human)** Push the tag: `git push origin vX.Y.Z`. This is the real
   release trigger. It fires `release.yml`, which:
   - builds the cross-platform binary matrix (SPEC-014),
   - creates/updates the GitHub Release with the archives + checksums,
   - runs the `publish` job, which verifies the tag version matches
     `Cargo.toml` and then runs `cargo publish --locked` using
     `CARGO_REGISTRY_TOKEN`.

## 3. Guardrails

- **Version match is enforced.** The `publish` job derives the `Cargo.toml`
  version the same way the `version` job does (`awk` on `Cargo.toml`) and
  compares it to the tag version. A mismatch fails the job before anything
  is published — this stops a stale or wrong `v*` tag from publishing the
  wrong crate version.
- **A published version cannot be republished.** crates.io rejects
  re-publishing the same version. If a publish fails partway or needs a
  fix, bump the version and re-tag; you cannot overwrite `X.Y.Z` once it's
  live.
- **macOS binaries are unsigned** until an Apple Developer key is available
  (Gatekeeper will warn on first run). Homebrew distribution is deferred
  for the same reason (`decisions/DEC-009`).
- **The token gates the job, not a workflow check.** If
  `CARGO_REGISTRY_TOKEN` is unset, the `publish` job fails at the
  `cargo publish` step — that's expected until one-time setup (§1) is done.
- **Release notes are auto-generated** (`gh release create --generate-notes`,
  SPEC-017) from merged PRs since the last tag (the full history for the
  first tag) — no hand-written `--notes` string. A release is always cut at
  whatever version `Cargo.toml` currently holds; bump `Cargo.toml` (§2.1,
  `just next-version`) *before* pushing the next tag, since the version-match
  guard (above) enforces tag == `Cargo.toml`.
