# SPEC-008 — VERIFY prompt (Opus subagent)

You are an **independent verifier** for `SPEC-008: SARIF output format`, run as a
metered subagent. A separate Sonnet build session implemented it and committed to
branch `feat/spec-008-sarif`. You did NOT build it. Disprove "done." **Do not
modify code, merge, or advance the cycle** — return a verdict.

## Review the diff

```bash
git diff main...HEAD -- src/ tests/ docs/
```

## Read (in order)

1. `projects/PROJ-001-skillport-lint/specs/SPEC-008-sarif-output-format-for-code-scanning.md`
   — the SARIF 2.1.0 shape, level mapping, ACs, Failing Tests, Out of scope,
   Build Completion/deviations.
2. `src/emit.rs`, `src/main.rs`, `docs/api-contract.md`; `decisions/DEC-003`/`DEC-005`.

## Verify — run it, don't trust names

```bash
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
cargo build
./target/debug/skillport lint lint-fixtures/bad --sarif | python3 -m json.tool >/dev/null && echo SARIF_PARSES
./target/debug/skillport lint lint-fixtures/bad --sarif ; echo exit=$?      # exit 1
./target/debug/skillport lint lint-fixtures/good --sarif ; echo exit=$?     # exit 0, results []
./target/debug/skillport lint lint-fixtures --sarif --json ; echo exit=$?   # exit 2, empty stdout
```

Adversarially probe:
- **Valid SARIF 2.1.0:** parses as JSON; `version=="2.1.0"`; `$schema` present;
  `runs[0].tool.driver.name=="skillport"` + crate `version`.
- **Level mapping (DEC-003):** Error→`error`, Warning→`warning`, **Info→`note`**
  (NOT "info"). Check an info finding's result level is `note`.
- **results correctness:** one per finding; `ruleId`/`message.text`/`artifactLocation.uri`
  correct; `region.startLine` present iff `finding.line` is Some, omitted otherwise.
- **driver.rules:** distinct rule ids, sorted, no dups.
- **Order:** results follow the report's order (sections path-sorted); the emitter
  doesn't reorder. Determinism: run twice, diff byte-identical.
- **Mutual exclusion:** `--sarif --json` → exit 2 (clap), stderr message, empty stdout.
- **Exit codes:** `--sarif` uses `Report::exit_code(strict)`; the SARIF bytes are
  identical with/without `--strict` (only exit differs).
- **No regressions:** human + `--json` output byte-for-byte unchanged; no new
  dependency (`git diff Cargo.toml Cargo.lock`); no new rule/analysis; good fixture 0/0/0.

## Return a verdict (final message = data for the orchestrator)

**✅ APPROVED** / **⚠ PUNCH LIST** (numbered, file:line + failing case) /
**❌ REJECTED** (which criterion, concrete command → observed vs expected). Include
gate results, your actual `--sarif` runs + whether the SARIF parses, a per-AC
pass/fail summary, and judge any deviations. Every flag needs a concrete
input→observed/expected. Don't touch code.
