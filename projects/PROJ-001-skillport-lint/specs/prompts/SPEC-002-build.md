# SPEC-002 — BUILD prompt (Sonnet subagent)

You are the **implementer** for `SPEC-002: collection tree-walker` in the
skillport repo. You run as a metered subagent on the branch
`feat/spec-002-walker`, which the orchestrator has already created and checked
out — **commit to the current branch; do not create or switch branches, do not
open a PR, do not merge.** The spec is your source of truth.

## Read first (in order)

1. `projects/PROJ-001-skillport-lint/specs/SPEC-002-collection-tree-walker.md`
   — the whole spec: **Acceptance Criteria**, **Failing Tests**, **Outputs** (the
   fixed `walk`/`Collection`/`CollectionItem` shape), **Out of scope**, **Notes**.
2. `src/parse.rs` and `src/skill.rs` — SPEC-001 (shipped). You **reuse**
   `parse(path, raw) -> Skill` and the `Skill` model; do not re-implement parsing.
3. `decisions/DEC-004` (collection-first) and `DEC-005` (deterministic; never abort).
4. `guidance/constraints.yaml` (`deterministic-stable-output`,
   `collection-first-substrate`, `test-before-implementation`) and
   `guidance/toolchain-brief.md` (current toolchain, `clippy -D warnings`).

## Your job

1. Implement `src/walk.rs`: `walk(root: &Path) -> Collection` plus `Collection`
   and `CollectionItem`, exactly to the shape in the spec's **Outputs**. Use a
   small **std-only** recursion over `std::fs::read_dir` — do **not** add
   `walkdir`. Read each `SKILL.md` with `fs::read` + `String::from_utf8`; a UTF-8
   failure becomes `CollectionItem::Unreadable` (walk continues). Sort items by
   path for determinism. Don't follow directory symlinks (guard against loops).
2. Wire the module into `src/lib.rs`.
3. Write **every** test in the spec's **Failing Tests** (in `#[cfg(test)] mod
   tests` in `src/walk.rs`) and make them pass. Tests must be hermetic — build
   temp trees (adding `tempfile` as a **dev-dependency** is fine and sanctioned;
   note it in Build Completion). Include the non-UTF-8 `Unreadable` case and the
   `.git`/`node_modules`/`target` ignore case.
4. Stay in scope: **no** rules, severities, rule ids, findings, CLI, or emitters
   (all later specs). `walk` returns raw discovery only.

## Definition of done

- Every **Acceptance Criterion** met; every **Failing Test** passes.
- `cargo test` green · `cargo clippy --all-targets -- -D warnings` clean ·
  `cargo fmt --check` clean.
- No new **runtime** dependency (dev-only `tempfile` is OK). If you believe a
  runtime dep is genuinely needed, STOP and say so in Build Completion rather than
  adding it.
- Fill the spec's **## Build Completion** section (branch, AC met?, deviations,
  follow-ups) and append a **build** cost session to `cost.sessions` per
  `projects/_templates/prompts/cost-snippet.md` — leave the numerics **null**
  (the orchestrator writes the real `tokens_total` from your subagent result at
  ship). Set `agents.implementer` to the model you actually ran as.
- Commit your work to `feat/spec-002-walker` with a Conventional-Commits message
  (`feat(SPEC-002): …`). Do **not** advance the cycle, open a PR, or merge — the
  orchestrator drives verify and ship.

## Return (your final message = data for the orchestrator)

Report: files changed, whether all ACs/tests pass, the exact `cargo test` /
`clippy` / `fmt` results, any deviations from the spec and why, whether you added
`tempfile`, and any follow-up you noticed. Be concise and factual.
