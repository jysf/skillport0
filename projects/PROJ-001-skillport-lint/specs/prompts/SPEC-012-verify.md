# SPEC-012 — VERIFY prompt (Opus subagent)

You are an **independent verifier** for `SPEC-012: rule reference readme, per-rule
fixtures, complete claude keys`, run as a metered subagent. A separate Sonnet build
session implemented it and committed to branch `feat/spec-012-rule-reference-readme`.
You did NOT build it. Disprove "done." **Do not modify code, merge, or advance the
cycle** — return a verdict.

> **The heart of this spec is anti-drift.** The claim is that skillport's rule surface
> is now *documented and can't silently drift from the code*. Attack that: is the
> README rule table ACTUALLY checked against a real code catalog, or is the test a
> tautology / trivially green? Does the catalog ACTUALLY enumerate every id the engine
> can emit? Is the "spec-perfect skill = zero findings" claim real?

## Review the diff

```bash
git diff main...HEAD
```

## Read (in order)

1. `projects/PROJ-001-skillport-lint/specs/SPEC-012-rule-reference-readme-per-rule-fixtures-complete-claude-keys.md`
   — the authoritative catalog table (26 ids + severities), the 5 new CLAUDE_KEYS,
   Acceptance Criteria, Failing Tests, Out of scope, Build Completion/deviations.
2. `src/rules.rs` (`CLAUDE_KEYS`, the `RULES`/`RuleDoc` catalog, `all_rule_ids`),
   `src/report.rs` (the structural ids), `src/lib.rs` (re-exports), `README.md`
   (Status table, `## Rule reference`, usage flags, example blocks), `tests/cli.rs`.
3. `decisions/DEC-002` (verified-only), `DEC-005` (rule ids = public contract),
   `DEC-003` (severity discipline).
4. Spot-check the docs only if a CLAUDE_KEYS fact looks off: WebFetch
   https://code.claude.com/docs/en/skills — are `when_to_use`, `argument-hint`,
   `agent`, `paths`, `shell` all in the Frontmatter reference?

## Verify — run it, don't trust names

```bash
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
cargo build
# spec-perfect fixture is truly clean, both ways:
./target/debug/skillport lint lint-fixtures/good/data-analysis ; echo perfect=$?
./target/debug/skillport lint lint-fixtures/good/data-analysis --target claude ; echo perfect_claude=$?
# the 5 new Claude fields no longer flag under the target, still flag without it:
# (construct or find a fixture using when_to_use/paths/etc., or drive lint_skill in a test)
```

Adversarially check each, with a concrete input→observed/expected:

- **CLAUDE_KEYS complete (DEC-002):** all 5 new fields present, each with a
  `// source:` comment; count is 13. Under `--target claude` a skill using
  `when_to_use`/`argument-hint`/`agent`/`paths`/`shell` fires NO `frontmatter.unknown`;
  without the target it DOES; a genuinely-unknown key still fires it under the target.
- **Catalog is the real source of truth:** `all_rule_ids()`/`RULES` enumerates all 26
  ids (24 engine + 2 structural), no duplicates, re-exported from `lib.rs`. The
  `catalog_is_locked` test pins exact contents (would fail if an id were
  added/removed/renamed — the DEC-005 tripwire). Confirm the catalog severities match
  the REAL emitted severities (cross-check a few against `src/rules.rs` `push(...)`
  sites — e.g. `body.size`=info, `description.detail`=info, `name.dir-match`=warning,
  `description.required`=error).
- **No orphan / full coverage:** `no_orphan_rule_ids` — every finding any fixture
  emits has its `rule` in the catalog. `every_engine_rule_has_a_fixture` — every engine
  id (catalog minus the 2 structural) is emitted by some committed fixture. Verify the
  test would actually FAIL if a rule lost its fixture (is the assertion real, or does
  it silently pass an empty set?). Confirm structural ids are excused with a comment.
- **README drift guard is not a tautology:** `readme_rule_table_matches_catalog`
  parses the real `## Rule reference` table and compares ids AND severities to the
  catalog. Sanity-check by reasoning about what happens if you edited a severity in the
  README (or dropped a row) — would the test fail? If the test regenerates the table
  from the catalog and compares, confirm it reads the actual README file (not a
  from-catalog string compared to itself).
- **README is actually refreshed:** Status table shows SPEC-001…011 shipped, no
  "⏳ next (SPEC-006)"/"arrive in SPEC-006" stale prose; usage documents `--target
  claude`, `--sarif` (mutually exclusive w/ `--json`), `--strict`; the example output
  block(s) match REAL current binary output (re-run the exact command and diff).
- **No scope creep (DEC-005):** no rule id renamed/removed, no severity changed, no
  `--json`/SARIF/exit-code change vs `main`. `git diff main -- src/emit.rs src/main.rs`
  should show no behavioral rule change. No new dependency (`Cargo.toml`/`Cargo.lock`
  diff empty). Determinism: run a lint twice, diff. Full pre-existing suite passes.

## Return a verdict (final message = data for the orchestrator)

**✅ APPROVED** / **⚠ PUNCH LIST** (numbered, file:line + failing case) /
**❌ REJECTED** (which criterion, concrete input → observed vs expected).
Include gate results, your actual runs (the perfect-fixture both ways; a
CLAUDE_KEYS-under-target check), a per-AC pass/fail summary, and an explicit judgment
on the two load-bearing claims: (1) is the README↔catalog drift test REAL (not a
tautology), and (2) does the catalog truly enumerate every emittable id? Every flag
needs a concrete input→observed/expected. Don't touch code.
