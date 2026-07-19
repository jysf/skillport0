---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-004                     # stable, zero-padded, continuous across the repo
  status: active                    # proposed | active | shipped | cancelled | on_hold
  priority: medium                  # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-001                      # parent project
repo:
  id: skillport

created_at: 2026-07-18
shipped_at: null

# What part of the project's value thesis this stage advances.
# If you can't articulate value_contribution, the stage may be
# infrastructure-only — acceptable but flag it.
value_contribution:
  advances: "Turns a CI-green-but-source-only tool into a distributed one — the last gap between 'skillport works' and 'a user can install and run it', which is where PROJ-001's value actually reaches people."
  delivers:
    - "`cargo install skillport` from crates.io"
    - "prebuilt cross-platform binaries attached to each GitHub Release (macOS arm64/x86_64, Linux x86_64/aarch64-musl, Windows x86_64), with sha256 sums"
    - "the shipped GitHub Action downloads the release binary instead of building from source (fast CI)"
    - "a tagged v0.1.0 with a CHANGELOG and a README install matrix"
  explicitly_does_not:
    - "ship a Homebrew tap (deferred until an Apple Developer key exists for signing/notarization — DEC-009)"
    - "sign/notarize the macOS binaries (same blocker; documented Gatekeeper friction until then)"
    - "add scoop/nix/AUR/deb channels (long tail, not worth pre-traction maintenance — DEC-009)"
    - "any lint/audit feature work (PROJ-001 lint is complete; audit is PROJ-002)"
---

# STAGE-004: Release & distribution

## What This Stage Is

The stage that makes skillport **installable**. `skillport lint` is complete and
CI-green, but the only way to get it today is to build from source. This stage stands
up the distribution foundation per DEC-009: a **GitHub Release** on each `v*` tag that
carries prebuilt, checksummed binaries for the common platforms; a **crates.io**
publish so `cargo install skillport` works; the shipped **Action** switched to
download the release binary instead of `cargo install --git`; and a cut **v0.1.0** with
a CHANGELOG and an install matrix in the README. Homebrew and macOS signing are
explicitly deferred (they need an Apple Developer key). When this ships, a user on any
of the target platforms can install skillport in one step.

## Why Now

It is last because distribution dresses a finished tool — there was nothing worth
releasing until the `lint` surface was complete, documented, and CI-verified
(STAGE-001…003). DEC-009 sequences the release explicitly **after** STAGE-003. The
GitHub Release is the foundation every other channel leans on (Homebrew points at the
release tarballs, the Action downloads them, cargo-binstall works for free), so it is
the right thing to build first.

## Success Criteria

- A user can run `cargo install skillport` and get the working `lint` CLI (crate
  published to crates.io under the confirmed-free name `skillport`).
- Pushing a `v*` tag produces a GitHub Release with prebuilt binaries for macOS
  (arm64 + x86_64), Linux (x86_64 + aarch64-musl), and Windows (x86_64), each stripped,
  archived, and accompanied by a sha256 sum.
- The repo is release-consistent: dual `LICENSE-MIT` + `LICENSE-APACHE` files matching
  `Cargo.toml`'s `MIT OR Apache-2.0`; crates.io metadata (`readme`, `keywords`,
  `categories`, `homepage`, `authors`) filled; identity = `github.com/jysf/skillport`
  everywhere.
- The shipped GitHub Action downloads the released binary (fast) rather than building
  from source, with a documented fallback.
- v0.1.0 is tagged with a CHANGELOG entry; the README documents the install matrix.
- The `--json`/SARIF/exit-code/rule-id contract (DEC-005) is unchanged by any of this.

## Scope

### In scope
- **Release Phase-0 prep** — dual-license files, crates.io Cargo metadata, identity
  consistency, confirm the crate name is free.
- **Release workflow** — `.github/workflows/release.yml`: on `v*`, cross-compile the
  platform matrix, strip, archive, sha256, attach to the GitHub Release; stamp
  `just build-info` provenance.
- **crates.io publish** — the packaging + a tag-triggered (or first-manual) publish.
- **Action speedup** — `action.yml` downloads the release binary with a build-from-source
  fallback.
- **Cut v0.1.0** — release spec (`just new-release-spec`), CHANGELOG, README install
  matrix; the human pushes the tag and triggers publish.

### Explicitly out of scope
- **Homebrew tap** and **macOS signing/notarization** (deferred — Apple Developer key;
  DEC-009). Homebrew revisited once the key exists.
- **scoop / nix / AUR / deb** and other long-tail channels (DEC-009).
- Any lint/audit **feature** work (PROJ-001 lint is done; audit = PROJ-002).

## Spec Backlog

Ordered per DEC-009's "Attack plan". First spec designed; the rest are proposed
decomposition (turned into specs via `just new-spec` / `just new-release-spec` as the
stage progresses).

- [x] SPEC-013 (shipped 2026-07-18, PR #13) — **Release Phase-0 prep** (S): dual
  `LICENSE-MIT` + `LICENSE-APACHE`; crates.io Cargo metadata (`readme`, `keywords`,
  `categories`, `homepage`, `authors`); README License section updated; a
  `cargo publish --dry-run` CI guard. Proven by dry-run exit 0. No runtime code / no
  contract change. Crate name `skillport` confirmed free on crates.io (re-confirm at
  SPEC-015). Verify APPROVED, 0 punch-list, clean first try.
- [x] SPEC-014 (shipped 2026-07-18, PR #14) — **Release workflow** (M):
  `.github/workflows/release.yml` cross-compiling the DEC-009 5-target matrix (macOS
  arm64+x86_64, Linux x86_64-gnu + aarch64-musl via `cross`, Windows x86_64) on `v*`,
  strip + archive + sha256 + attach to the Release via `gh` (no third-party release
  action); provenance `build-info.txt`. `workflow_dispatch` dry path builds+uploads
  artifacts without creating a Release. No src/contract change. Verify APPROVED,
  0 punch-list. **Human smoke-test recommended:** trigger a `workflow_dispatch` run
  once (creates no Release) to exercise the full 5-leg matrix before SPEC-017.
- [x] SPEC-015 (shipped 2026-07-19, PR #15) — **crates.io publish** (S): a tag-gated
  `publish` job on `release.yml` (`cargo publish --locked` with `CARGO_REGISTRY_TOKEN`
  secret, version-match guard, skipped on `workflow_dispatch`) + a `RELEASING.md` runbook.
  Verify APPROVED, 0 punch-list. **Human-only (unblocked):** set the token secret + do
  the first `cargo publish` + push the tag — per RELEASING.md. Crate name re-confirmed
  free (404).
- [x] SPEC-016 (shipped 2026-07-19, PR #16) — **Action speedup** (M): a testable
  `scripts/install-release.sh` maps runner OS/arch → the SPEC-014 archive, downloads +
  sha256-verifies + extracts the prebuilt binary onto PATH; `action.yml` runs it, with
  dtolnay + `cargo install --git` gated to fallback only (no release yet / unsupported
  platform). `--print-plan` dry mode + `version` input (default `latest`). README updated.
  Verify APPROVED, 0 punch-list. Download-success path first exercised at v0.1.0 (SPEC-017).
- [ ] (not yet written) SPEC-017 — **Cut v0.1.0** (S, release-spec): CHANGELOG + README
  install matrix + `just next-version`; verify each channel installs. **Human-only**:
  push the `v0.1.0` tag, trigger publish.

**Count:** 4 shipped / 0 active / 1 pending (SPEC-013…016 shipped; SPEC-017 = cut v0.1.0, the last spec — needs the human-only tag push).

## Design Notes

- **DEC-009** is the governing decision (GitHub Releases + crates.io first; Homebrew
  deferred; dual MIT/Apache; canonical `github.com/jysf/skillport`). This stage
  executes its Attack plan; read it before each spec.
- **Human-only guardrail steps** (Claude prepares, the human triggers — publish/credential/
  irreversible): `cargo publish`, pushing any `v*` tag, creating the future
  `homebrew-tap` repo. Every spec that reaches one of these must stop at the boundary
  and hand off, not attempt it.
- **The contract is frozen** (DEC-005): none of the release work may change the
  `--json` schema, SARIF, exit codes, or rule ids. Releasing is packaging, not behavior.
- **`build-info` provenance** (`docs/versioning.md`, `just build-info`) should stamp
  release artifacts so a downloaded binary is traceable to a commit.
- The crate name `skillport` was confirmed **free** on crates.io at 2026-07-18
  (API 404) — re-confirm immediately before first publish (SPEC-015), since names can
  be claimed at any time.

## Dependencies

### Depends on
- STAGE-003 (a complete, documented, CI-green `lint` — there is now something worth
  releasing) and SPEC-009 (the Action this stage points at the release binary).
- External: crates.io (account + API token, human-held), GitHub Releases, the GitHub
  Actions cross-compile toolchains. Apple Developer key is a dependency only for the
  **deferred** Homebrew/signing work, not this stage.

### Enables
- A shippable PROJ-001 (its value finally reaches users) and its close.
- A future Homebrew tap (points at these release tarballs) once the Apple key exists.
- PROJ-002 (`audit`) ships through the same release machinery.

## Stage-Level Reflection

*Filled in when status moves to shipped. Run Prompt 1d (Stage Ship) in
FIRST_SESSION_PROMPTS.md to draft this.*

- **Did we deliver the outcome in "What This Stage Is"?** <yes/no + notes>
- **How many specs did it actually take?** <number vs. plan>
- **What changed between starting and shipping?** <one sentence>
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - <one-line updates>
- **Signals dispositioned at this close?** (Prompt 1d step 7) Every
  `type: lesson` signal in `/guidance/signals.yaml` owned by this stage close
  was walked — codified (at its bar), left `watch`, or dropped. No silent carry.
  - <note what codified / what's still watch + its N>
- **Should any spec-level reflections be promoted to stage-level lessons?**
  - <one-line items — record below-bar ones as `watch` signals; don't codify yet>

## Stage-Level Reflection

*Filled in when status moves to shipped. Run Prompt 1c (Stage Ship) in
FIRST_SESSION_PROMPTS.md to draft this.*

- **Did we deliver the outcome in "What This Stage Is"?** <yes/no + notes>
- **How many specs did it actually take?** <number vs. plan>
- **What changed between starting and shipping?** <one sentence>
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - <one-line updates>
- **Signals dispositioned at this close?** (Prompt 1d step 7) Every
  `type: lesson` signal in `/guidance/signals.yaml` owned by this stage close
  was walked — codified (at its bar), left `watch`, or dropped. No silent carry.
  - <note what codified / what's still watch + its N>
- **Should any spec-level reflections be promoted to stage-level lessons?**
  - <one-line items — record below-bar ones as `watch` signals; don't codify yet>
