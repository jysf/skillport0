# SPEC-011 — BUILD prompt (Sonnet subagent)

You are the **implementer** for `SPEC-011: --target claude`. You run as a metered
subagent on branch `feat/spec-011-target-claude`, already created and checked out
— **commit to the current branch; do not create/switch branches, open a PR, or
merge.** The spec is your source of truth.

> **DEC-002 is the governing rule here.** Encode ONLY the Claude facts the spec
> verified from primary docs, and put a `// source: code.claude.com/docs/en/skills`
> comment on each. Do NOT add any Claude behavior not in the spec. Do NOT add
> Cursor/Codex/Vercel targets (unverified).

## Read first (in order)

1. `projects/PROJ-001-skillport-lint/specs/SPEC-011-target-claude-recognized-fields.md`
   — the **Verified from Claude's primary docs** section, the **Behavior under
   `--target claude`** table, Acceptance Criteria, Failing Tests, Out of scope, Notes.
2. `src/rules.rs` (`SPEC_KEYS`, `check_unknown_fields`, `check_allowed_tools`,
   `lint_skill`), `src/main.rs` (the `Lint` command + `emit::json(&report, None)`
   call), `src/emit.rs` (`json` already takes `target: Option<&str>`).
3. `decisions/DEC-002`, `DEC-003`, `DEC-005`.

## Your job

1. `src/rules.rs`: add `pub enum Target { Claude }` and a `CLAUDE_KEYS` const listing
   ONLY Claude-specific fields (`disable-model-invocation`, `user-invocable`,
   `disallowed-tools`, `model`, `effort`, `context`, `hooks`, `arguments`) with a
   `// source: code.claude.com/docs/en/skills` comment. Give `check_unknown_fields`
   and `check_allowed_tools` a `target: Option<Target>` param and route BOTH
   `pub fn lint_skill(skill)` (calls with `None`, behavior UNCHANGED) and a new
   `pub fn lint_skill_with_target(skill, target)` through the same body.
   - `frontmatter.unknown` recognizes `SPEC_KEYS ∪ CLAUDE_KEYS` when target is
     `Some(Claude)`; still fires on a truly-unknown key.
   - `allowed-tools.format` (list case) is `Info` (not `Warning`) when target is
     `Some(Claude)`, message noting Claude accepts a list (cite the source).
   - `allowed-tools.type` and every open-spec rule (name.required,
     description.length, …) are UNCHANGED by target.
2. `src/main.rs`: add `--target <TARGET>` to `Lint` as a clap `ValueEnum` with the
   single variant `claude` (unknown → clap usage error, exit 2); map to
   `rules::Target`; use `Report::from_collection(&c, |s| lint_skill_with_target(s,
   target))`; pass the target label to `emit::json` so `--target claude` →
   `"target":"claude"` (and `null` otherwise).
3. `src/lib.rs`: re-export `Target` + `lint_skill_with_target`.
4. Add the fixture `lint-fixtures/good-claude/<name>/SKILL.md` (valid; uses
   `allowed-tools:` as a YAML list + `context: fork`) — clean under `--target claude`
   (0/0/0) but would emit `frontmatter.unknown`(info)+`allowed-tools.format`(warning)
   without it. Write **every** Failing Test in the spec (rules unit tests + `tests/cli.rs`).

## Definition of done

- Every **Acceptance Criterion** met; every **Failing Test** passes.
- `cargo test` green · `cargo clippy --all-targets -- -D warnings` clean ·
  `cargo fmt --check` clean.
- `lint_skill` (no target) behaves exactly as before (existing tests unchanged).
- The good fixture stays 0/0/0 with and without `--target claude`. No new dependency.
- Fill the spec's **## Build Completion**, append a **build** cost session (null
  numerics, per `projects/_templates/prompts/cost-snippet.md`), set
  `agents.implementer` to your model, commit to `feat/spec-011-target-claude`
  (`feat(SPEC-011): …`). Do **not** advance cycle, PR, or merge.

## Return (final message = data for the orchestrator)

Concise + factual: files changed, all ACs/tests pass with exact `cargo test`/
`clippy`/`fmt` lines, PASTE `skillport lint lint-fixtures/good-claude` with and
without `--target claude` (showing the difference), confirm `--json` target label +
no dep + good fixture clean, any deviations, follow-ups.
