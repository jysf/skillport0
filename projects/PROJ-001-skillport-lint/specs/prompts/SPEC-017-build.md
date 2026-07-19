# SPEC-017 — BUILD prompt (Sonnet subagent)

You are the **implementer** for `SPEC-017: cut v0.1.0 — install matrix + release notes`.
You run as a metered subagent on branch `feat/spec-017-cut-v0-1-0`, already created and
checked out — **commit to the current branch; do not create/switch branches, open a PR,
or merge.** The spec is your source of truth.

> **Prepare the release; do not cut it.** Do NOT push a tag, create a Release, run
> `cargo publish`, or bump the version. Do NOT change `src/`, `Cargo.toml`, `Cargo.lock`,
> `.github/workflows/ci.yml`, `action.yml`, or `scripts/` (DEC-005). Do NOT create an app
> `CHANGELOG.md` (root CHANGELOG.md is the template's). The deliverable is README + a
> one-line release.yml notes change + a RELEASING.md line (+ spec bookkeeping).

## Read first (in order)

1. `projects/PROJ-001-skillport-lint/specs/SPEC-017-cut-v0-1-0-install-matrix-and-release-notes.md`
   — Outputs, Acceptance Criteria, Failing Tests, Notes, Out of scope.
2. `.github/workflows/release.yml` (SPEC-014 asset names `skillport-0.1.0-<triple>.<ext>`
   + the current `gh release create … --notes …` line), `scripts/install-release.sh`
   (SPEC-016 platform→triple map — the install matrix must mirror it), `README.md`
   (Status section + the `@v0` Action example + the `## Rule reference` table you must
   NOT disturb), `Cargo.toml` (version `0.1.0`), `docs/versioning.md`, `decisions/DEC-009`.

## Your job

1. **`README.md`:**
   - Add a **## Install** section (near the top): (a) crates.io `cargo install skillport`;
     (b) a 5-row prebuilt-binary table (platform → asset `skillport-0.1.0-<triple>.<ext>`,
     EXACTLY matching SPEC-014/016) with copy-pasteable download-from-`…/releases/download/v0.1.0/…`
     + `sha256sum -c`/`shasum -a 256 -c` + extract steps (binary at
     `skillport-0.1.0-<triple>/skillport`), and a macOS-unsigned note; (c) the Action
     `uses: jysf/skillport@v0.1.0`.
   - Add a CI badge and a crates.io version badge near the title.
   - Update the **Status** section: no longer "not yet released"/"feature-complete for
     STAGE-003" — reflect v0.1.0 as the first release + SPEC-001…016 shipped (`audit` =
     PROJ-002). **Do NOT touch the `## Rule reference` table** (SPEC-012's drift test).
   - Pin the "Use in CI" example `@v0` → `@v0.1.0`.
2. **`.github/workflows/release.yml`:** change the `release` job's `gh release create`
   from `--notes "skillport ${VERSION} — see build-info.txt for provenance."` to
   `--generate-notes`. Change ONLY that flag; leave the title, asset list, and the
   `|| gh release upload … --clobber` fallback intact.
3. **`RELEASING.md`:** add one line noting release notes are auto-generated
   (`--generate-notes`) and a release is cut at the current `Cargo.toml` version (the
   version-match guard enforces tag == Cargo.toml; bump Cargo.toml before a later tag).
4. **`Cargo.toml`:** confirm it's `0.1.0` — do NOT edit.

## Definition of done

- Every **Acceptance Criterion** met; every **Failing Test** passes. Run and PASTE:
  `actionlint .github/workflows/release.yml` (exit 0); `cargo test` (incl.
  `readme_rule_table_matches_catalog` — the README edits must not break it);
  `cargo clippy --all-targets -- -D warnings`; `cargo fmt --check`; `cargo publish
  --dry-run`; and a grep showing the 5 README asset names match `skillport-0.1.0-<triple>.<ext>`
  + the Action `@v0.1.0` pin.
- `git diff main -- src/ Cargo.toml Cargo.lock .github/workflows/ci.yml action.yml scripts/`
  is EMPTY. No app `CHANGELOG.md` created.
- Fill the spec's **## Build Completion**, append a **build** cost session (null numerics,
  per `projects/_templates/prompts/cost-snippet.md`), set `agents.implementer` to your
  model, commit to `feat/spec-017-cut-v0-1-0` (`chore(SPEC-017): …` or `feat(SPEC-017):
  …`). Do **not** advance cycle, PR, merge, tag, publish, or bump the version.

## Return (final message = data for the orchestrator)

Concise + factual: PASTE the new README **Install** section + the badges + the Status
diff + the `release.yml` notes-flag diff + the RELEASING.md line; the `actionlint` +
gate results (incl. `readme_rule_table_matches_catalog` passing); the asset-name/Action-pin
grep; confirm `Cargo.toml` unchanged at 0.1.0, no app CHANGELOG created, `git diff main --
src/ Cargo.toml Cargo.lock ci.yml action.yml scripts/` empty; any deviations/follow-ups.
