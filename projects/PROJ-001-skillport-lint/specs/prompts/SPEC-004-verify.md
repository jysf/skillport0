# SPEC-004 — VERIFY prompt (Opus subagent)

You are an **independent verifier** for `SPEC-004: open-spec rule engine and
identity rules`, run as a metered subagent. A separate Sonnet build session
implemented it and committed to branch `feat/spec-004-rules`. You did NOT build
it. Disprove "done." **Do not modify code, merge, or advance the cycle** — return
a verdict.

## Review the diff

```bash
git diff main...HEAD -- src/
```

## Read (in order)

1. `projects/PROJ-001-skillport-lint/specs/SPEC-004-open-spec-rule-engine-and-identity-rules.md`
   — the Rules table (exact ids/severities), the two design decisions, Acceptance
   Criteria, Failing Tests, Out of scope, Build Completion/deviations.
2. `src/skill.rs`, `src/report.rs`, `src/walk.rs` (reused); `decisions/DEC-002`,
   `DEC-003`, `DEC-005`; the rule catalog in `stages/STAGE-002-*.md`.

## Verify — run it, don't trust names

```bash
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

Check **every Acceptance Criterion** and adversarially probe:
- **Every rule id + severity matches the table exactly** (grep the ids; confirm
  `name.dir-match` is warning, `description.detail` is info, everything else error).
- **No heuristic at error level (DEC-003):** only `description.detail`/`name.dir-match`
  are non-error. Confirm nothing crisp was made warning/info or vice-versa.
- **Frontmatter presence:** the three statuses map to the three ids and field
  rules are **skipped** when not `Present` (no `name.required` piling on top of
  `frontmatter.missing`).
- **The locked empty-`Present` decision:** empty map + `Present` → `name.required`
  + `description.required`, NOT `frontmatter.missing`. Try it.
- **name.* correctness:** boundary lengths (0/64/65), uppercase/space charset,
  hyphen edges, `--`; dir-match warning only on mismatch, skipped when `dir_name`
  is None.
- **description/compatibility:** thresholds (0, 1024/1025, 500/501) exact.
- **A valid skill → zero findings** (directly + the `lint-fixtures/good` integration
  test → `summary.errors == 0`).
- **Scope:** no `metadata.*`/`allowed-tools.*`/`body.*`/`frontmatter.unknown`, no
  CLI/emitters/target/tokenizer; no new dependency; determinism (no HashMap-order
  observable).

## Return a verdict (final message = data for the orchestrator)

**✅ APPROVED** / **⚠ PUNCH LIST** (numbered, file:line + failing case) /
**❌ REJECTED** (which criterion/decision, concrete input → observed vs expected).
Include gate results, a per-rule or per-AC pass/fail summary, and judge the two
design decisions (three `frontmatter.*` ids; empty-`Present` handling) — sound or
a hole? Every flag needs a concrete input→observed/expected. Don't touch code.
