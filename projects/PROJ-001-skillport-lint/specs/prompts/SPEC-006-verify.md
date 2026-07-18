# SPEC-006 — VERIFY prompt (Opus subagent)

You are an **independent verifier** for `SPEC-006: remaining open-spec rules`, run
as a metered subagent. A separate Sonnet build session implemented it and
committed to branch `feat/spec-006-rules2`. You did NOT build it. Disprove "done."
**Do not modify code, merge, or advance the cycle** — return a verdict.

## Review the diff

```bash
git diff main...HEAD -- src/ tests/
```

## Read (in order)

1. `projects/PROJ-001-skillport-lint/specs/SPEC-006-remaining-open-spec-rules-metadata-tools-body-unknown.md`
   — the Rules table (exact ids/severities), the `name.charset` ASCII change, the
   open field set, ACs, Failing Tests, Out of scope, Build Completion/deviations.
2. `src/rules.rs`, `src/skill.rs`, `src/report.rs`; `decisions/DEC-002`/`DEC-003`/`DEC-005`;
   the catalog in `stages/STAGE-002-*.md`.

## Verify — run it, don't trust names

```bash
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
cargo build
./target/debug/skillport lint lint-fixtures/bad     # should now include allowed-tools.format, metadata.values, frontmatter.unknown
./target/debug/skillport lint lint-fixtures/good     # must stay clean, exit 0
```

Check **every Acceptance Criterion** and adversarially probe:
- **Every new rule id + severity matches the table exactly** (grep the ids). No
  heuristic at error level; the only error touched is `name.charset` (still error).
- **`name.charset` ASCII:** a `name` with `café` or a non-ASCII digit now → error;
  a valid ASCII kebab name → no charset finding. Confirm no over-rejection of
  legit ASCII (hyphen, digits ok).
- **Skip discipline:** field rules only run when frontmatter is `Present`.
- **metadata / allowed-tools / body / unknown / compatibility** each behave per the
  table (mapping vs not; list vs string vs other; empty/>500 lines; unknown key;
  non-string compatibility). Absent fields → no finding.
- **The good fixture is genuinely clean** — 0 errors AND 0 warnings AND 0 infos
  (not just 0 errors). If a new rule fires on it, that's a regression.
- **Determinism** (no HashMap-order leakage); **no CLI/emitter change**
  (`main.rs`/`emit.rs` unchanged except tests); **no new dependency**.
- Judge the `allowed-tools.type` extension (neither string nor list) — reasonable
  or scope creep?

## Return a verdict (final message = data for the orchestrator)

**✅ APPROVED** / **⚠ PUNCH LIST** (numbered, file:line + failing case) /
**❌ REJECTED** (which criterion, concrete input → observed vs expected). Include
gate results, the actual `skillport lint` runs on good+bad, a per-rule pass/fail
summary, and judge any deviations. Every flag needs a concrete
input→observed/expected. Don't touch code.
