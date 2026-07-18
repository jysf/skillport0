# SPEC-006 — BUILD prompt (Sonnet subagent)

You are the **implementer** for `SPEC-006: remaining open-spec rules`. You run as
a metered subagent on branch `feat/spec-006-rules2`, already created and checked
out — **commit to the current branch; do not create/switch branches, open a PR,
or merge.** The spec is your source of truth.

## Read first (in order)

1. `projects/PROJ-001-skillport-lint/specs/SPEC-006-remaining-open-spec-rules-metadata-tools-body-unknown.md`
   — the **Rules to implement** table (exact ids/severities), the `name.charset`
   ASCII tightening, the open field set, Acceptance Criteria, Failing Tests, Out of
   scope, Notes.
2. `src/rules.rs` (extend `lint_skill` + its tests — keep the SPEC-004 skip
   discipline), `src/skill.rs` (`Frontmatter`/`YamlValue` accessors, `body`,
   `keys()`), `src/report.rs` (`Finding`/`Severity`).
3. `decisions/DEC-002`, `DEC-003`, `DEC-005`.
4. Reference: `initial_stuff/lint.rs` (`check_metadata`, `check_allowed_tools`,
   `check_body`, `check_unknown_fields`, `SPEC_KEYS`) — port onto the current types.

## Your job

1. Extend `rules::lint_skill` with, at the exact severities in the table:
   `metadata.type` (warn), `metadata.values` (info), `allowed-tools.format` (warn),
   `allowed-tools.type` (warn), `body.empty` (warn), `body.lines` (warn, >500),
   `frontmatter.unknown` (info, vs the open field set `name`/`description`/`license`/
   `compatibility`/`metadata`/`allowed-tools`), and `compatibility.type` (warn).
   Keep the **run-only-when-`Present`** skip discipline.
2. **Tighten `name.charset`** to strict ASCII: predicate becomes
   `c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'`. Keep id/severity
   (`name.charset`/error). Update the existing charset test + add non-ASCII cases.
3. Write **every** Failing Test in the spec (extend `#[cfg(test)] mod tests` in
   `src/rules.rs`; optionally add a `tests/cli.rs` assertion that
   `lint lint-fixtures/bad` stdout now contains `allowed-tools.format` and
   `frontmatter.unknown`).
4. **Do NOT implement `body.size`** (tokenizer → STAGE-003). **No** `--target`,
   `--sarif`, CLI/emitter changes, or new deps. `main.rs`/`emit.rs` stay untouched
   (except an optional test).

## Definition of done

- Every **Acceptance Criterion** met; every **Failing Test** passes.
- `cargo test` green · `cargo clippy --all-targets -- -D warnings` clean ·
  `cargo fmt --check` clean.
- `lint-fixtures/good` still yields **zero findings** (errors, warnings, infos all 0).
- No new dependency.
- Fill the spec's **## Build Completion**, append a **build** cost session (null
  numerics, per `projects/_templates/prompts/cost-snippet.md`), set
  `agents.implementer` to your model, commit to `feat/spec-006-rules2`
  (`feat(SPEC-006): …`). Do **not** advance cycle, PR, or merge.

## Return (final message = data for the orchestrator)

Concise + factual: files changed, all ACs/tests pass with exact `cargo test`/
`clippy`/`fmt` lines, confirm the good fixture is still clean and no dep/CLI change,
paste the new `skillport lint lint-fixtures/bad` output, note any deviations
(e.g. `allowed-tools.type` extension) and follow-ups.
