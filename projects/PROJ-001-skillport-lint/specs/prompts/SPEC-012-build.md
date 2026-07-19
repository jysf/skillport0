# SPEC-012 — BUILD prompt (Sonnet subagent)

You are the **implementer** for `SPEC-012: rule reference readme, per-rule fixtures,
complete claude keys`. You run as a metered subagent on branch
`feat/spec-012-rule-reference-readme`, already created and checked out — **commit to
the current branch; do not create/switch branches, open a PR, or merge.** The spec is
your source of truth.

> **This spec adds NO new rules and changes NO severities or rule ids.** It (a)
> completes `CLAUDE_KEYS` with 5 already-verified Claude fields, (b) adds a code-level
> rule *catalog* enumerating the existing 26 ids, (c) refreshes the README with a rule
> reference + current status + flags, and (d) adds fixtures + drift/coverage tests. If
> you find yourself renaming an id or changing a severity, STOP — that's out of scope
> (DEC-005: rule ids are a public contract).

## Read first (in order)

1. `projects/PROJ-001-skillport-lint/specs/SPEC-012-rule-reference-readme-per-rule-fixtures-complete-claude-keys.md`
   — especially **The authoritative rule catalog** table (the 26 ids + real
   severities, probed from code at design), **The 5 new CLAUDE_KEYS**, Acceptance
   Criteria, Failing Tests, Out of scope, Notes for the Implementer.
2. `src/rules.rs` (`SPEC_KEYS`, `CLAUDE_KEYS`, every `push(...)` site, the thresholds
   `DESCRIPTION_DETAIL_THRESHOLD`/`BODY_LINES_THRESHOLD`/`BODY_TOKENS_THRESHOLD`),
   `src/report.rs` (`FILE_UNREADABLE`/`DIR_UNREADABLE`), `src/lib.rs`, `README.md`.
3. `decisions/DEC-002`, `DEC-005`, `DEC-003`.
4. The existing `lint-fixtures/` tree and `tests/cli.rs`.

## Your job

1. **Complete `CLAUDE_KEYS`** (`src/rules.rs`): append `"when_to_use"`,
   `"argument-hint"`, `"agent"`, `"paths"`, `"shell"`, each on its own line with a
   `// source: code.claude.com/docs/en/skills` comment (match the existing 8). Result:
   13 Claude-extension fields.
2. **Add the rule catalog** (single source of truth). Prefer a
   `pub struct RuleDoc { pub id: &'static str, pub severity: Severity, pub summary:
   &'static str }` and a `pub const RULES: &[RuleDoc]` (or `pub fn rules() -> &[RuleDoc]`)
   enumerating **all 26** ids from the spec's catalog table — the 24 engine ids
   (`src/rules.rs`) plus the 2 structural ids (`file.unreadable`=Error,
   `dir.unreadable`=Warning from `src/report.rs`). Mark the 2 structural ones (a
   `structural: bool` field, or a separate `STRUCTURAL_RULES` list) so tests can
   exclude them from fixture coverage. `allowed-tools.format`'s catalog severity is its
   **default** `Warning` (document the `--target claude`→`Info` downgrade in the README
   notes column, not as a second entry). Re-export the catalog + `RuleDoc` from
   `src/lib.rs`. Put the catalog wherever it reads cleanly (e.g. `src/rules.rs`, with
   the structural entries sourced from `report.rs` constants) — but there must be ONE
   list the tests and README both derive from.
3. **Refresh `README.md`:**
   - Rewrite the **Status** table so SPEC-001…011 read as shipped and remove the
     "⏳ next (SPEC-006)" / "arrive in SPEC-006" stale lines. State that `lint`
     enforces the full open-spec catalog + `--target claude` + `--sarif` +
     real-tokenizer `body.size`.
   - Add a **## Rule reference** section with a table: rule id | severity | fires when
     | notes. Include ALL 26 ids. Notes must cover: `allowed-tools.format` list→info
     under `--target claude`; `frontmatter.unknown` recognizes Claude's fields under
     `--target claude`; `body.lines` > 500, `body.size` ~> 5000 tokens (real
     tokenizer); the two structural ids. Severities MUST match the catalog defaults.
   - Document `--target claude`, `--sarif` (mutually exclusive with `--json`), and
     `--strict` in the usage section.
   - **Regenerate the example output block(s) from the REAL binary** — build, run
     `./target/debug/skillport lint lint-fixtures/bad` (and a `--json` and/or
     `--target claude` example), paste actual output. Do not hand-edit.
4. **Fixtures** (`lint-fixtures/`): ensure a designated **spec-perfect** skill under
   `lint-fixtures/good/` yields 0/0/0 with AND without `--target claude` (reuse
   `good/data-analysis` if it already does; otherwise add one — `allowed-tools` as a
   *space-separated string*, dir name == `name`, description ≥ 40 chars, non-empty body
   under thresholds, only recognized fields). Add the minimum focused fixtures so that
   **every engine rule id** in the catalog is emitted by some fixture (structural ids
   excused). Keep each fixture small + commented.
5. **Write every Failing Test in the spec:** `claude_keys_complete`,
   `catalog_is_locked`, `no_orphan_rule_ids`, `every_engine_rule_has_a_fixture`,
   `spec_perfect_skill_is_clean`, `readme_rule_table_matches_catalog`. Make the README
   parser robust (parse only the `## Rule reference` region; match backtick-wrapped
   ids + severity words; don't assert on prose). If README parsing proves too fragile,
   the spec permits generating the table body from the catalog into a fenced block and
   comparing byte-for-byte instead — but try parse-and-compare-sets first.

## Definition of done

- Every **Acceptance Criterion** met; every **Failing Test** passes.
- `cargo test` green · `cargo clippy --all-targets -- -D warnings` clean ·
  `cargo fmt --check` clean.
- No rule id renamed/removed, no severity changed, no `--json`/SARIF/exit-code change.
  No new dependency. Deterministic.
- The full pre-existing suite still passes unchanged.
- Fill the spec's **## Build Completion**, append a **build** cost session (null
  numerics, per `projects/_templates/prompts/cost-snippet.md`), set
  `agents.implementer` to your model, commit to `feat/spec-012-rule-reference-readme`
  (`feat(SPEC-012): …`). Do **not** advance cycle, PR, or merge.

## Return (final message = data for the orchestrator)

Concise + factual: files changed; confirm all ACs/tests pass with exact
`cargo test` / `clippy` / `fmt` lines; PASTE the new README **Rule reference** table
and the regenerated example output; state the final `CLAUDE_KEYS` count (13) and the
catalog id count (26); list any new fixtures; confirm no dep / no id-or-severity
change / determinism; note any deviations + follow-ups.
