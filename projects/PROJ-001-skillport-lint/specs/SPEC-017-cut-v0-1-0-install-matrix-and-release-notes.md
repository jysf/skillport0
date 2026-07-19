---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-017
  type: chore                      # epic | story | task | bug | chore
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
  created_at: 2026-07-19

references:
  decisions:
    - DEC-009   # distribution strategy — Attack-plan step 5 (cut v0.1.0)
    - DEC-005   # frozen contract — cutting a release is packaging/docs, not behavior
  constraints:
    - deterministic-stable-output
    - test-before-implementation
  related_specs:
    - SPEC-014  # release.yml (asset names the install matrix documents; the notes tweak lands here)
    - SPEC-015  # crates.io publish + RELEASING.md (the publish this release triggers)
    - SPEC-016  # the Action (pin the README example to @v0.1.0)
    - SPEC-013  # crates.io metadata (cargo install works because of it)

value_link: "the last STAGE-004 spec: makes v0.1.0 installable-and-documented — a README install matrix (crates.io / prebuilt binaries / Action), auto-generated GitHub release notes, and a v0.1.0-ready repo — so the human's single `git push v0.1.0` cuts the first real release end-to-end."

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
      notes: "main-loop, not separately metered (design cycle); probed `just next-version` (v0.1.0, no bump), confirmed root CHANGELOG.md is TEMPLATE-owned (app release notes go in the GitHub Release), read the SPEC-014 asset names + the release.yml notes line"
    - cycle: build
      agent: claude-sonnet-5
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-07-19
      notes: "metered subagent build; orchestrator fills tokens_total/duration/estimated_usd from the Agent result at ship"
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-017: cut v0.1.0 — install matrix + release notes

## Context

The last spec of STAGE-004 (DEC-009 Attack-plan step 5) and the last of PROJ-001. The
release pipeline is fully built and verified (SPEC-013 metadata/licenses, SPEC-014
binaries-on-tag, SPEC-015 crates.io publish, SPEC-016 the Action downloads the binary).
Nothing user-facing yet documents **how to install** the released tool, and the release
notes the workflow attaches are a one-line placeholder. This spec makes the repo
**v0.1.0-ready**: a README install matrix, auto-generated GitHub release notes, badges,
and the Action example pinned to `@v0.1.0` — so the human's single `git push origin
v0.1.0` produces a first release that is both functional and documented.

**What this spec does NOT do:** push the tag, publish the crate, or create a Release —
those are human-only (DEC-009). It also does **not** bump the version: `just
next-version` reports `v0.1.0` and `Cargo.toml` is already `0.1.0`, so the first tag is
cut at the current version (no change). And it does **not** create an app `CHANGELOG.md`:
the root `CHANGELOG.md` is the **spec-driven-template's** changelog (template-managed);
per `docs/versioning.md` the app versions via git tags + `Cargo.toml`, so skillport's
release notes live in the **GitHub Release** (auto-generated from merged PRs).

## Goal

Make v0.1.0 installable-and-documented without cutting it: add a README **Install**
section (crates.io / prebuilt binaries / Action) + CI & crates.io badges, update the
Status section to "released", pin the Action example to `@v0.1.0`, and switch the
release job's notes to GitHub's auto-generated notes (`--generate-notes`) so v0.1.0 gets
real release notes. No version bump, no `src/` change, no tag/publish.

## Inputs

- **Files to modify:** `README.md` (add Install section + badges; update Status; pin the
  Action example `@v0` → `@v0.1.0`), `.github/workflows/release.yml` (the `gh release
  create` notes flag), `RELEASING.md` (one line noting auto-generated notes + no-bump).
- **Files to read:** `.github/workflows/release.yml` (SPEC-014 asset names
  `skillport-0.1.0-<triple>.<ext>` + the current `gh release create` line),
  `scripts/install-release.sh` (SPEC-016 platform table — the install matrix must match),
  `Cargo.toml` (version `0.1.0`, unchanged), `decisions/DEC-009`, `docs/versioning.md`.
- **No `src/`/`Cargo.toml`/`Cargo.lock` change. No new dependency.**

## Outputs

- **`README.md`:**
  - A new **## Install** section (place it near the top, before or just after "Status"),
    documenting three channels for **v0.1.0**:
    1. **crates.io:** `cargo install skillport` (works once published; SPEC-015).
    2. **Prebuilt binaries:** a table of the 5 platforms → asset name
       (`skillport-0.1.0-<triple>.<ext>`), matching SPEC-014/SPEC-016 exactly (macOS
       arm64 `aarch64-apple-darwin` / x86_64 `x86_64-apple-darwin` `.tar.gz`; Linux x86_64
       `x86_64-unknown-linux-gnu` / arm64 `aarch64-unknown-linux-musl` `.tar.gz`; Windows
       x86_64 `x86_64-pc-windows-msvc` `.zip`), with download-from-Release + verify
       `.sha256` + extract steps (the binary is inside the extracted
       `skillport-0.1.0-<triple>/` dir), and a note that macOS binaries are **unsigned**
       until an Apple Developer key exists (Gatekeeper: right-click → Open, or
       `xattr -d com.apple.quarantine`).
    3. **GitHub Action:** `uses: jysf/skillport@v0.1.0` (pin the existing "Use in CI"
       example from `@v0` to `@v0.1.0`).
  - **Badges** near the title: a CI-status badge
    (`https://github.com/jysf/skillport/actions/workflows/ci.yml/badge.svg`) and a
    crates.io version badge (`https://img.shields.io/crates/v/skillport.svg` → the
    crates.io page) — the crates badge populates once published (fine to add ahead).
  - **Status section:** update the "feature-complete for STAGE-003 and not yet released"
    wording to reflect **v0.1.0 as the first release** and `lint` complete; keep the
    per-SPEC table accurate (SPEC-001…016 shipped; `audit` = PROJ-002).
- **`.github/workflows/release.yml`:** change the `release` job's `gh release create`
  from `--notes "skillport ${VERSION} — see build-info.txt for provenance."` to
  **`--generate-notes`** (GitHub auto-generates notes from the merged PRs since the last
  tag; for the first tag it uses the repo history). Keep everything else — the
  idempotent `|| gh release upload … --clobber` fallback, the asset list, the title. The
  build-info.txt asset still ships (provenance remains attached).
- **`RELEASING.md`:** add a one-line note that release notes are auto-generated
  (`--generate-notes`) and that a release is cut at the current `Cargo.toml` version (the
  version-match guard enforces tag == Cargo.toml; bump `Cargo.toml` *before* tagging a
  later release, per `just next-version`).
- **`Cargo.toml`:** unchanged (already `0.1.0`). Confirm, don't edit.

## Acceptance Criteria

- [ ] README has an **## Install** section covering crates.io (`cargo install
      skillport`), the 5 prebuilt-binary assets (names **exactly**
      `skillport-0.1.0-<triple>.<ext>` matching SPEC-014/016) with checksum-verify +
      extract steps and the macOS-unsigned note, and the Action pinned to `@v0.1.0`.
- [ ] The README Action example uses `jysf/skillport@v0.1.0` (no remaining bare `@v0` in
      the usage example); the crates.io + CI badges are present and well-formed.
- [ ] README Status no longer says "not yet released"/"feature-complete for STAGE-003";
      it reflects v0.1.0 + the SPEC-001…016 shipped set. The rule-reference /
      drift-guard test from SPEC-012 still passes (README rule table unchanged/consistent).
- [ ] `release.yml` uses `--generate-notes` (not the placeholder `--notes` string);
      `actionlint` passes; the tag-guard/version-match/publish jobs are otherwise
      unchanged. `git diff` on `release.yml` is limited to the notes flag.
- [ ] `Cargo.toml` version is still `0.1.0` (unchanged); no app `CHANGELOG.md` created;
      `RELEASING.md` notes the auto-notes + version-match/no-bump behavior.
- [ ] No `src/`/`Cargo.toml`/`Cargo.lock`/`ci.yml`/`action.yml`/`scripts/` change; no new
      dependency; `cargo test`/`clippy`/`fmt`/`cargo publish --dry-run` all pass; no
      `--json`/SARIF/exit-code/rule-id change (DEC-005). Nothing is tagged or published.

## Failing Tests

Docs + a one-line workflow change → static/consistency checks (satisfies
`test-before-implementation` by pre-specifying the assertions):

- **Install-matrix accuracy** — grep the README Install table for each of the 5 asset
  names and assert they equal `skillport-0.1.0-<triple>.<ext>` for the SPEC-014 triples
  (cross-checked against `scripts/install-release.sh`'s map). A mismatch = a user
  downloading a non-existent file.
- **Action pin** — the README "Use in CI" example contains `jysf/skillport@v0.1.0` and
  no bare `uses: jysf/skillport@v0` remains in that example.
- **Release notes** — `release.yml`'s `gh release create` uses `--generate-notes` and no
  longer contains the placeholder `see build-info.txt for provenance` `--notes` string;
  `actionlint .github/workflows/release.yml` exits 0.
- **Version unchanged** — `Cargo.toml` version == `0.1.0`; `just next-version` still
  reports `v0.1.0`.
- **SPEC-012 drift guard intact** — `cargo test` passes, including
  `readme_rule_table_matches_catalog` (the Install-section edits must not disturb the
  Rule reference table).
- **Contract untouched** — `git diff main -- src/ Cargo.toml Cargo.lock
  .github/workflows/ci.yml action.yml scripts/` is empty.

## Implementation Context

*Read this section before starting the build cycle.*

### Decisions that apply

- `DEC-009` — Attack-plan step 5 (cut v0.1.0). Prepare the release; do **not** push a
  tag, create a Release, or `cargo publish` (human-only). macOS-unsigned + Homebrew-later
  are expected and documented.
- `DEC-005` — the CLI/JSON/exit-code/rule-id contract is frozen; this is docs + a
  release-notes flag. No `src/` change.

### Constraints that apply

- `deterministic-stable-output` — the documented asset names are the fixed SPEC-014
  scheme; the install matrix mirrors `scripts/install-release.sh`'s map.
- `test-before-implementation` — the consistency checks above are the pre-written
  verification.

### Prior related work

- `SPEC-014` (PR #14) — the asset names `skillport-<ver>-<triple>.<ext>` and the
  `gh release create` line this touches. `SPEC-016` (PR #16) — the platform→triple map
  the install matrix must mirror. `SPEC-015` (PR #15) — `RELEASING.md` + the publish job.
  `SPEC-013` (PR #13) — crates.io metadata (so `cargo install skillport` works).
- `SPEC-012` (PR #12) — the README rule-reference table + its drift test; keep it intact.

### Out of scope (for this spec specifically)

- **No tag / Release / publish** (human-only). **No version bump** (first tag is at
  `0.1.0`). **No app `CHANGELOG.md`** (template owns root `CHANGELOG.md`; app notes =
  GitHub Release).
- **No `src/`/`Cargo.toml`/`ci.yml`/`action.yml`/`scripts/` change.** No Homebrew, no
  signing (deferred, DEC-009). No new platform assets beyond SPEC-014's five.

## Notes for the Implementer

- The install matrix's asset names must match `.github/workflows/release.yml` and
  `scripts/install-release.sh` **exactly** — read both; if they disagree with what you
  write, you're wrong, not them.
- For the prebuilt-binary steps, show a real example (e.g. Linux x86_64): download the
  `.tar.gz` + `.sha256` from `…/releases/download/v0.1.0/…`, `sha256sum -c`, `tar xzf`,
  and note the binary is at `skillport-0.1.0-<triple>/skillport`. Keep it copy-pasteable.
- `--generate-notes`: for the very first tag (no prior release) GitHub generates notes
  from the full history; that's fine. Don't combine with a `--notes` string (mutually
  awkward) — `--generate-notes` alone. The PR titles (`feat(SPEC-0xx): …`) make good notes.
- Keep the README edits surgical so `readme_rule_table_matches_catalog` (SPEC-012) still
  passes — don't touch the `## Rule reference` table.
- The crates.io badge will 404 until the crate is published; that's expected and fine.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:**
- **PR (if applicable):**
- **All acceptance criteria met?** yes/no
- **New decisions emitted:**
  - `DEC-NNN` — <title> (if any)
- **Deviations from spec:**
  - [list]
- **Follow-up work identified:**
  - [any new specs for the stage's backlog]

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — <answer>

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — <answer>

3. **If you did this task again, what would you do differently?**
   — <answer>

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
