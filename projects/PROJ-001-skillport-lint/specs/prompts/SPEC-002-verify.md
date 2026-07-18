# SPEC-002 — VERIFY prompt (Opus subagent)

You are an **independent verifier** for `SPEC-002: collection tree-walker`, run as
a metered subagent. A separate Sonnet build session implemented it and committed
to branch `feat/spec-002-walker`. You did not build it. Your job is to *disprove*
"done," not rubber-stamp it. **Do not merge, do not advance the cycle** — return a
verdict to the orchestrator.

## Review the change

The working tree is already on `feat/spec-002-walker`. Inspect the diff:
```bash
git diff main...HEAD -- src/ Cargo.toml
```

## Read (in order)

1. `projects/PROJ-001-skillport-lint/specs/SPEC-002-collection-tree-walker.md`
   — Acceptance Criteria, Failing Tests, Outputs shape, Out of scope, and the
   builder's **## Build Completion** (deviations).
2. `src/parse.rs` / `src/skill.rs` (SPEC-001, reused), `decisions/DEC-004`, `DEC-005`.
3. `guidance/constraints.yaml` (`deterministic-stable-output`,
   `collection-first-substrate`), `AGENTS.md` §12.

## Verify — run it, don't trust names

```bash
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

Check **every Acceptance Criterion** against the actual code (read assertions, not
just test names). Adversarially probe:
- **Never aborts (DEC-005):** find any input that panics or aborts the walk — a
  non-UTF-8 `SKILL.md`, a permission-denied file, a broken/circular **symlink**, a
  very deep tree. Confirm `walk` returns `Collection` (no `Result`) and one bad
  file doesn't lose the others.
- **Determinism (DEC-005):** items sorted by path; no reliance on `read_dir` order
  or `HashMap` iteration. Same tree → same order.
- **Ignore list:** `.git` / `node_modules` / `target` genuinely not descended
  (a `SKILL.md` inside each is absent from results).
- **Collection-first (DEC-004):** single file → 1-item collection; explicit
  non-`SKILL.md` file is honored; directory walk matches only exact `SKILL.md`.
- **Reuse:** parsing goes through SPEC-001's `parse` (not re-implemented);
  malformed frontmatter is a `Skill` item (not `Unreadable`); `dir_name` set.
- **Scope:** no rules/findings/CLI/emitters; **no `walkdir`** or other new runtime
  dep snuck in (dev-only `tempfile` is allowed). Tests are hermetic (no repo litter).

## Return a verdict (your final message = data for the orchestrator)

One of **✅ APPROVED** / **⚠ PUNCH LIST** (numbered fixes, each file:line +
failing case) / **❌ REJECTED** (which criterion/decision, with concrete
input → observed vs expected). Give the gate results (test/clippy/fmt), a per-AC
pass/fail summary, and judge each of the builder's deviations. Every flag needs a
concrete input→observed/expected, not a vibe. Do not modify code, merge, or
advance the cycle.
