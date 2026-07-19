# SPEC-017 — VERIFY prompt (Opus subagent)

You are an **independent verifier** for `SPEC-017: cut v0.1.0 — install matrix + release
notes`, run as a metered subagent. A separate Sonnet build session edited `README.md`,
`.github/workflows/release.yml`, and `RELEASING.md`, committed to branch
`feat/spec-017-cut-v0-1-0`. You did NOT build it. Disprove "done." **Do not modify code,
merge, advance the cycle, tag, publish, or bump the version** — return a verdict.

> **The claim:** the repo is v0.1.0-ready — a README install matrix whose asset names
> exactly match what `release.yml` will actually produce, real auto-generated release
> notes, the Action pinned to `@v0.1.0` — with NO version bump, NO app CHANGELOG, and NO
> contract change. Attack: would a user following the install matrix download a
> **non-existent** asset (name mismatch)? Did the README edits break SPEC-012's rule-table
> drift test? Did anything outside docs+the one notes flag change?

## Review the diff

```bash
git diff main...HEAD --stat
git diff main -- src/ Cargo.toml Cargo.lock .github/workflows/ci.yml action.yml scripts/   # MUST be empty
git diff main -- .github/workflows/release.yml   # MUST be limited to the notes flag
```

## Read

1. `projects/PROJ-001-skillport-lint/specs/SPEC-017-cut-v0-1-0-install-matrix-and-release-notes.md`
   — Outputs, Acceptance Criteria, Failing Tests, Out of scope.
2. `README.md` (the new `## Install` + badges + Status + the Action example),
   `.github/workflows/release.yml` (the `gh release create` line + SPEC-014 asset names),
   `scripts/install-release.sh` (SPEC-016 platform map), `RELEASING.md`, `Cargo.toml`.

## Verify — run it, don't trust the report

```bash
cargo test ; cargo clippy --all-targets -- -D warnings ; cargo fmt --check ; cargo publish --dry-run
actionlint .github/workflows/release.yml ; echo actionlint=$?
grep -n 'version' Cargo.toml | head -1          # MUST be 0.1.0
just next-version                                # MUST report v0.1.0
grep -nE 'skillport-0\.1\.0-[a-z0-9_-]+\.(tar\.gz|zip)' README.md   # the 5 asset names
grep -n 'jysf/skillport@v0' README.md            # pin check (want @v0.1.0, not bare @v0)
```

Adversarially check, each with concrete observed/expected:

- **Install-matrix asset names EXACTLY match `release.yml`** — the 5 README asset names
  are `skillport-0.1.0-<triple>.<ext>` for the SPEC-014 triples: `aarch64-apple-darwin`,
  `x86_64-apple-darwin`, `x86_64-unknown-linux-gnu` (all `.tar.gz`),
  `aarch64-unknown-linux-musl` (`.tar.gz`), `x86_64-pc-windows-msvc` (`.zip`). Cross-check
  against `release.yml`'s matrix `stage=` naming and `scripts/install-release.sh`'s map. A
  single-char mismatch = a user downloading a 404.
- **Action pin:** the "Use in CI" example uses `jysf/skillport@v0.1.0`; no bare
  `uses: jysf/skillport@v0` remains **in the usage example** (a prose mention of the `v0`
  moving tag is fine — judge intent).
- **SPEC-012 drift guard intact:** `cargo test` passes, specifically
  `readme_rule_table_matches_catalog` — the Install/Status edits must not have disturbed
  the `## Rule reference` table (ids or severities).
- **Release notes:** `release.yml` uses `--generate-notes` and no longer contains the
  `see build-info.txt for provenance` placeholder `--notes` string. `actionlint` exit 0.
  The tag-guard / version-match / publish jobs are otherwise unchanged (diff limited to
  the notes flag).
- **No version bump / no app CHANGELOG:** `Cargo.toml` is still `0.1.0`; there is no new
  app `CHANGELOG.md` (the root `CHANGELOG.md`, if changed at all, must be untouched —
  `git diff main -- CHANGELOG.md` empty).
- **No scope creep (DEC-005):** `git diff main -- src/ Cargo.toml Cargo.lock ci.yml
  action.yml scripts/` EMPTY; no new dependency; no `--json`/SARIF/exit-code/rule-id
  change. Nothing tagged/published.
- **Badges** are well-formed (CI badge points at `ci.yml`; crates.io badge at
  `skillport`); a 404 crates badge pre-publish is expected, not a defect.
- **Status** no longer says "not yet released"/"feature-complete for STAGE-003".

## Return a verdict (final message = data for the orchestrator)

**✅ APPROVED** / **⚠ PUNCH LIST** (numbered, file:line + concrete issue) /
**❌ REJECTED** (which criterion, observed vs expected). Include the gate results (call
out `readme_rule_table_matches_catalog` explicitly), the actionlint result, the
asset-name cross-check (all 5, README vs release.yml), the Action-pin + Cargo.toml-0.1.0
checks, the release.yml-diff-limited-to-notes-flag confirmation, the empty `git diff main
-- src/ … scripts/` confirmation, and a per-AC pass/fail summary. State explicitly whether
any README asset name would 404 against what release.yml produces, and whether the version
was left at 0.1.0. Don't touch code, tag, publish, or bump.
