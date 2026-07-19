# SPEC-011 — VERIFY prompt (Opus subagent)

You are an **independent verifier** for `SPEC-011: --target claude`, run as a
metered subagent. A separate Sonnet build session implemented it and committed to
branch `feat/spec-011-target-claude`. You did NOT build it. Disprove "done." **Do
not modify code, merge, or advance the cycle** — return a verdict.

> **DEC-002 focus:** the point of this spec is that per-platform behavior is
> **verified, not guessed**. Check that every encoded Claude fact matches the cited
> primary docs (https://code.claude.com/docs/en/skills) and carries a source
> comment — and that NO unverified behavior (or Cursor/Codex/Vercel target) crept in.

## Review the diff

```bash
git diff main...HEAD
```

## Read (in order)

1. `projects/PROJ-001-skillport-lint/specs/SPEC-011-target-claude-recognized-fields.md`
   — the Verified-facts section, the Behavior table, ACs, Failing Tests, Out of
   scope, Build Completion/deviations.
2. `src/rules.rs`, `src/main.rs`, `src/emit.rs`; `decisions/DEC-002`/`DEC-003`/`DEC-005`.
3. Spot-check the docs if unsure: WebFetch https://code.claude.com/docs/en/skills
   (Frontmatter reference) — does it list the fields in `CLAUDE_KEYS`? Does it say
   `allowed-tools` accepts a YAML list?

## Verify — run it, don't trust names

```bash
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
cargo build
./target/debug/skillport lint lint-fixtures/good-claude ; echo no_target=$?
./target/debug/skillport lint lint-fixtures/good-claude --target claude ; echo claude=$?
./target/debug/skillport lint lint-fixtures/good-claude --target claude --json | python3 -m json.tool | grep -A1 target
./target/debug/skillport lint lint-fixtures/bad --target bogus ; echo bogus=$?   # exit 2
./target/debug/skillport lint lint-fixtures/good ; echo good=$?                  # 0
```

Adversarially check:
- **Verified-only (DEC-002):** every field in `CLAUDE_KEYS` appears in the cited
  docs' Frontmatter reference; each carries a `// source:` comment. No Claude
  behavior beyond recognized-fields + the allowed-tools list downgrade. No
  Cursor/Codex/Vercel target. `--target` accepts ONLY `claude` (bogus → exit 2).
- **frontmatter.unknown widening:** under `--target claude`, Claude fields
  (`context`, `model`, `disable-model-invocation`, `user-invocable`,
  `disallowed-tools`, `effort`, `hooks`, `arguments`) do NOT fire it; a genuinely
  unknown key (e.g. `random_field`) STILL fires it. Without target, the Claude
  fields DO fire it.
- **allowed-tools.format downgrade:** list + `--target claude` → **info** (not
  warning), message cites Claude accepts a list; without target → **warning**.
  `allowed-tools.type` unchanged.
- **Open spec unchanged (DEC-002):** `--target claude` does NOT relax
  `name.required`, `description.length`, or any open-spec rule. Construct a
  Claude-target skill missing `name` → still `name.required` error.
- **Default unchanged:** `lint_skill(skill)` (no target) behaves exactly as before;
  all pre-existing tests pass.
- **--json target:** `"target":"claude"` under `--target claude`, `null` otherwise;
  SARIF unchanged. Determinism (run twice, diff). No new dependency. Good fixture
  0/0/0 with and without the target.

## Return a verdict (final message = data for the orchestrator)

**✅ APPROVED** / **⚠ PUNCH LIST** (numbered, file:line + failing case) /
**❌ REJECTED** (which criterion/fact, concrete input → observed vs expected).
Include gate results, your actual `--target claude` runs (with vs without), the
docs cross-check, a per-AC pass/fail summary, and explicitly judge whether every
encoded Claude fact is doc-verified (DEC-002). Every flag needs a concrete
input→observed/expected. Don't touch code.
