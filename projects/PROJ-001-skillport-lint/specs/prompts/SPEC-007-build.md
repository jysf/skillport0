# SPEC-007 — BUILD prompt (Sonnet subagent)

You are the **implementer** for `SPEC-007: surface unreadable directories as
findings`. You run as a metered subagent on branch
`feat/spec-007-unreadable-dir`, already created and checked out — **commit to the
current branch; do not create/switch branches, open a PR, or merge.** The spec is
your source of truth.

## Read first (in order)

1. `projects/PROJ-001-skillport-lint/specs/SPEC-007-surface-unreadable-directories-as-findings.md`
   — the Goal, the Design decisions (warning severity; section-per-item shape),
   Acceptance Criteria, Failing Tests (note the `#[cfg(unix)]` permission tests),
   Out of scope, Notes.
2. `src/walk.rs` — the `CollectionItem` enum, `CollectionItem::path()`, `walk`, and
   the `collect` recursion with `let Ok(entries) = read_dir(dir) else { return; }`
   (the swallow site).
3. `src/report.rs` — `from_collection`'s `match` over `CollectionItem` and the
   `Unreadable`/`FILE_UNREADABLE` arm to mirror.
4. `decisions/DEC-003`, `DEC-004`, `DEC-005`; `guidance/constraints.yaml`.

## Your job

1. `src/walk.rs`: add `CollectionItem::UnreadableDir { path, error }`; update
   `CollectionItem::path()`. In `collect`, when `read_dir(dir)` errors, record the
   dir as an `UnreadableDir` (thread a `&mut Vec<(PathBuf, String)>` out-param, or
   push items directly) instead of silently returning. Include these in the
   path-sorted `items`. **Do not** emit for intentionally-ignored dirs
   (`.git`/`node_modules`/`target`) — those stay silently skipped.
2. `src/report.rs`: handle `UnreadableDir` in `from_collection` → one finding with
   `rule == "dir.unreadable"` (`const DIR_UNREADABLE = "dir.unreadable"`),
   `severity == Warning`, the dir's path, a message that the subtree wasn't
   checked; **do not** increment `summary.skills`. Mirror the `FILE_UNREADABLE` arm.
3. Write **every** Failing Test in the spec: `src/walk.rs` `#[cfg(unix)]` tests
   (chmod 000 a subdir, assert `UnreadableDir` + siblings still found, chmod back
   for cleanup), `src/report.rs` tests (construct `UnreadableDir` directly — id,
   severity, exit-code, summary), and the optional `tests/cli.rs` `#[cfg(unix)]`
   end-to-end. Make them pass.
4. **Stay in scope:** substrate/coverage finding only. NO new lint rule, NO
   `key.duplicate` (resolved-redundant — do not add), NO `body.size`/`--target`/
   `--sarif`, NO CLI/emitter change (except an optional test), NO new dependency.

## Definition of done

- Every **Acceptance Criterion** met; every **Failing Test** passes.
- `cargo test` green · `cargo clippy --all-targets -- -D warnings` clean ·
  `cargo fmt --check` clean.
- The good fixture still yields zero findings; `skillport lint` unchanged on normal
  trees.
- Fill the spec's **## Build Completion**, append a **build** cost session (null
  numerics, per `projects/_templates/prompts/cost-snippet.md`), set
  `agents.implementer` to your model, commit to `feat/spec-007-unreadable-dir`
  (`feat(SPEC-007): …`). Do **not** advance cycle, PR, or merge.

## Return (final message = data for the orchestrator)

Concise + factual: files changed, all ACs/tests pass with exact `cargo test`/
`clippy`/`fmt` lines, confirm no dep/CLI change and the good fixture stays clean,
paste a `skillport lint` run over a tree with a chmod-000 subdir showing the
`dir.unreadable` warning, note any deviations and follow-ups.
