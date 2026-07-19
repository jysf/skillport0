# SPEC-015 — VERIFY prompt (Opus subagent)

You are an **independent verifier** for `SPEC-015: crates.io publish (tag-triggered) +
RELEASING doc`, run as a metered subagent. A separate Sonnet build session added a
`publish` job to `.github/workflows/release.yml` and a `RELEASING.md`, committed to
branch `feat/spec-015-crates-publish`. You did NOT build it. Disprove "done." **Do not
modify code, merge, advance the cycle, run `cargo publish`, add a secret, or push a
tag** — return a verdict.

> **The claim:** the publish is prepared but NOT fired — a tag-gated `publish` job that
> can only run on a `v*` tag with the human's token, guarded against a version mismatch,
> and a runbook for the human. Attack: could this job publish on a dry `workflow_dispatch`
> or with a mismatched version? Is the token handled safely (secret, not literal)? Was
> anything actually published, or any contract file touched?

## Review the diff

```bash
git diff main...HEAD --stat
git diff main -- src/ Cargo.toml Cargo.lock .github/workflows/ci.yml action.yml   # MUST be empty
```

## Read

1. `projects/PROJ-001-skillport-lint/specs/SPEC-015-crates-io-publish-tag-triggered-plus-releasing-doc.md`
   — Outputs, Acceptance Criteria, Failing Tests, Out of scope.
2. `.github/workflows/release.yml` (the whole file — the new `publish` job in context),
   `RELEASING.md`, `Cargo.toml`, `decisions/DEC-009`, `DEC-005`.

## Verify — run it, don't trust the report

```bash
actionlint .github/workflows/release.yml ; echo actionlint=$?
cargo test ; cargo clippy --all-targets -- -D warnings ; cargo fmt --check
cargo publish --dry-run ; echo dryrun=$?
curl -s -o /dev/null -w '%{http_code}' -H 'User-Agent: skillport-verify' https://crates.io/api/v1/crates/skillport ; echo   # MUST be 404
grep -n 'CARGO_REGISTRY_TOKEN' .github/workflows/release.yml   # only via secrets.
```

Adversarially check, each with concrete observed/expected:

- **Tag-only publish (the critical safety property):** the `publish` job has
  `if: startsWith(github.ref, 'refs/tags/v')`. **Reason about `workflow_dispatch`:** a
  dispatch runs with a branch ref, so the job is SKIPPED — no publish on a dry run.
  Confirm the guard is present and identical in intent to the `release` job's.
- **Version-match guard:** a step derives the `Cargo.toml` version and `exit 1`s if it
  ≠ the tag version (`needs.version.outputs.version`), BEFORE `cargo publish`. Trace the
  logic: a `v0.2.0` tag against a `0.1.0` Cargo.toml must fail, not publish.
- **Token safety:** `CARGO_REGISTRY_TOKEN` is referenced ONLY as
  `${{ secrets.CARGO_REGISTRY_TOKEN }}` (grep finds no literal token value). Publish uses
  `cargo publish --locked`. No third-party publish action (no `katyo/publish-crates`,
  etc.) — only `checkout` + `dtolnay/rust-toolchain` + cargo.
- **`needs: [version, build]`** — publish only after the binary matrix succeeds.
- **Nothing published / contract untouched:** the crates.io API still returns **404**
  for `skillport`; `git diff main -- src/ Cargo.toml Cargo.lock ci.yml action.yml` is
  EMPTY; no new Cargo dependency; the SPEC-013 `cargo publish --dry-run` CI guard is
  still present in `ci.yml` (not weakened/removed).
- **`RELEASING.md` is a real runbook:** documents the `CARGO_REGISTRY_TOKEN` secret setup,
  the manual first-publish to establish ownership, the tag-driven per-release flow, and
  the version-match / already-published / unsigned-macOS guardrails; marks the human-only
  steps; links DEC-009.
- **actionlint clean**, gates green, `cargo publish --dry-run` exit 0.

## Return a verdict (final message = data for the orchestrator)

**✅ APPROVED** / **⚠ PUNCH LIST** (numbered, file:line + concrete issue) /
**❌ REJECTED** (which criterion, observed vs expected). Include the actionlint result,
gate results, the crates.io 404 check, the token-safety grep, the tag-guard +
version-guard trace, the empty `git diff main -- src/ … action.yml` confirmation, and a
per-AC pass/fail summary. State explicitly: (1) can a `workflow_dispatch` publish? (must
NOT), and (2) is the token a secret, never a literal? Don't touch code, publish, tag, or
add secrets.
