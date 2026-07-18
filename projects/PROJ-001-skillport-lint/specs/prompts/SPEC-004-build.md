# SPEC-004 — BUILD prompt (Sonnet subagent)

You are the **implementer** for `SPEC-004: open-spec rule engine and identity
rules`. You run as a metered subagent on branch `feat/spec-004-rules`, already
created and checked out — **commit to the current branch; do not create/switch
branches, open a PR, or merge.** The spec is your source of truth.

## Read first (in order)

1. `projects/PROJ-001-skillport-lint/specs/SPEC-004-open-spec-rule-engine-and-identity-rules.md`
   — the whole spec: the **Rules to implement** table (exact ids + severities),
   the two **design decisions** (three `frontmatter.*` ids; empty-`Present` →
   name/description-required NOT frontmatter.missing), **Acceptance Criteria**,
   **Failing Tests**, **Out of scope**, **Notes**.
2. `src/skill.rs` (`Skill`, `Frontmatter`, `FrontmatterStatus`, `YamlValue`),
   `src/report.rs` (`Finding`, `Severity`, `Report::from_collection`), `src/walk.rs`
   (`Collection`, `walk`).
3. `decisions/DEC-002`, `DEC-003`, `DEC-005`.
4. `guidance/constraints.yaml` (`only-verified-constraints-are-firm`,
   `no-heuristic-error`, `deterministic-stable-output`, `test-before-implementation`)
   and `guidance/toolchain-brief.md`.
5. Reference (port, don't copy blindly): `initial_stuff/lint.rs`.

## Your job

1. Create `src/rules.rs` with `pub fn lint_skill(skill: &Skill) -> Vec<Finding>`
   implementing EXACTLY the rules in the spec's table, at the listed severities,
   with the stable rule id strings. Honor the two design decisions:
   - `FrontmatterStatus` `Missing`/`Unclosed`/`Invalid` → `frontmatter.missing` /
     `frontmatter.unclosed` / `frontmatter.invalid` (error), and then RETURN (skip
     field rules — don't read an empty map).
   - `Present` but empty map → `name.required` + `description.required` fire, NOT
     `frontmatter.missing`.
2. Wire the module into `src/lib.rs` (expose `rules` + `lint_skill`).
3. Write **every** test in the spec's **Failing Tests** (`#[cfg(test)] mod tests`
   in `src/rules.rs`), including the integration test:
   `walk("lint-fixtures/good")` → `Report::from_collection(&c, rules::lint_skill)`
   → `summary.errors == 0`. Make them all pass.
4. **Stay in scope:** only the rules in the table. NO `metadata.*`,
   `allowed-tools.*`, `body.*`, `frontmatter.unknown` (next spec). NO CLI, NO
   emitters, NO `--target`, NO tokenizer. NO heuristic at error level.

## Definition of done

- Every **Acceptance Criterion** met; every **Failing Test** passes.
- `cargo test` green · `cargo clippy --all-targets -- -D warnings` clean ·
  `cargo fmt --check` clean.
- **No new dependency.** If you think one is needed, STOP and report.
- Fill the spec's **## Build Completion** (branch, AC met?, deviations, follow-ups),
  append a **build** cost session to `cost.sessions` with **null** numerics (per
  `projects/_templates/prompts/cost-snippet.md`), set `agents.implementer` to the
  model you ran as, and commit to `feat/spec-004-rules` (`feat(SPEC-004): …`). Do
  **not** advance cycle, PR, or merge.

## Return (your final message = data for the orchestrator)

Concise + factual: files changed, all ACs/tests pass with exact `cargo test` /
`clippy` / `fmt` result lines, any deviations from the spec and why, confirmation
you added no dependency and only the in-scope rules, and any follow-ups noticed.
