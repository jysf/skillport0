# SPEC-013 — VERIFY prompt (Opus subagent)

You are an **independent verifier** for `SPEC-013: release phase-0 prep — dual license
+ crates.io metadata`, run as a metered subagent. A separate Sonnet build session
implemented it and committed to branch `feat/spec-013-release-prep`. You did NOT build
it. Disprove "done." **Do not modify code, merge, advance the cycle, or run
`cargo publish` for real** — return a verdict.

> **The claim:** the crate is now packageable for crates.io (dual license on disk
> matches `Cargo.toml`, metadata present, README truthful) with NO behavior change. The
> proof is a clean `cargo publish --dry-run`. Attack: does the dry-run ACTUALLY pass on
> the committed tree? Do the license files actually match the declared license? Did any
> `src/`/contract change sneak in?

## Review the diff

```bash
git diff main...HEAD --stat
git diff main...HEAD -- src/ Cargo.lock      # MUST be empty
```

## Read (in order)

1. `projects/PROJ-001-skillport-lint/specs/SPEC-013-release-phase-0-prep-dual-license-and-crates-metadata.md`
   — Outputs, Acceptance Criteria, Failing Tests, Out of scope.
2. `Cargo.toml`, `LICENSE-MIT`, `LICENSE-APACHE`, `README.md` (`## License`),
   `.github/workflows/ci.yml` (if a guard was added), `decisions/DEC-009`, `DEC-005`.

## Verify — run it, don't trust the report

```bash
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
cargo publish --dry-run ; echo dryrun=$?          # MUST be 0
cargo package --list | grep -E 'LICENSE|README|Cargo.toml'
cargo metadata --no-deps --format-version 1 | \
  python3 -c "import json,sys; p=json.load(sys.stdin)['packages'][0]; print({k:p[k] for k in ['authors','keywords','categories','homepage','readme','license']})"
ls -la LICENSE* ; echo "bare LICENSE exists? "; test -e LICENSE && echo YES || echo NO
```

Adversarially check, each with a concrete observed/expected:

- **Dual license (matches `Cargo.toml`):** `LICENSE-MIT` AND `LICENSE-APACHE` exist; no
  bare `LICENSE`. `LICENSE-APACHE` is the original Apache-2.0 text (first line contains
  `Apache License`; unchanged from `main`'s `LICENSE` — `git log --follow` /
  `git show main:LICENSE | diff - LICENSE-APACHE` should be empty). `LICENSE-MIT` is the
  real MIT text with `Copyright (c) 2026 jysf` and the word `MIT`. `Cargo.toml` keeps
  `license = "MIT OR Apache-2.0"` and has NO `license-file` key.
- **Metadata:** `authors`, `readme`, `homepage`, `keywords` (≤ 5, each ≤ 20 chars),
  `categories` all present; `categories` are VALID crates.io slugs (`command-line-utilities`,
  `development-tools` are valid — an invalid slug would fail the dry-run, so the dry-run
  passing is corroboration).
- **Dry-run truly passes:** `cargo publish --dry-run` exits 0 on the clean committed
  tree (no `--allow-dirty`). The packaged list includes `LICENSE-MIT`, `LICENSE-APACHE`,
  `README.md`, `Cargo.toml`.
- **README truthful:** the `## License` section links both `LICENSE-MIT` and
  `LICENSE-APACHE`, states dual MIT-OR-Apache, and no longer contains "call to confirm"
  or "inherited from the template".
- **No scope creep (DEC-005):** `git diff main -- src/ Cargo.lock` is EMPTY (no runtime
  code, no dependency change). No `--json`/SARIF/exit-code/rule-id change. No real
  `cargo publish` was performed (the crate is NOT on crates.io — a `curl` to
  `https://crates.io/api/v1/crates/skillport` should still 404; do NOT publish). If a CI
  guard was added to `ci.yml`, confirm it uses `--dry-run` (never a real publish).
- **Gates green; full suite unchanged** (131 tests expected).

## Return a verdict (final message = data for the orchestrator)

**✅ APPROVED** / **⚠ PUNCH LIST** (numbered, file:line + failing check) /
**❌ REJECTED** (which criterion, concrete observed vs expected).
Include the gate results, your actual `cargo publish --dry-run` exit code + package
list, the metadata dump, the license-file checks (incl. the Apache-text-unchanged
diff), confirmation that `git diff main -- src/ Cargo.lock` is empty, and a per-AC
pass/fail summary. Do NOT publish, tag, or touch code.
