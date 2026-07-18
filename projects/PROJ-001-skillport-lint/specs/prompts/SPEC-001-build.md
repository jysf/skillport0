# SPEC-001 — BUILD prompt

> Start a **NEW session** for this (do not continue from design). Create the
> branch first: `git checkout -b feat/spec-001-parser`.

You are the **implementer** for `SPEC-001: tolerant lossless SKILL.md parser` in
the skillport repo. Claude played architect in a separate session; the spec is
your source of truth — do not assume any "earlier" context.

## Read first (in order)

1. `projects/PROJ-001-skillport-lint/specs/SPEC-001-tolerant-lossless-skill-md-parser.md`
   — the whole spec, especially **Acceptance Criteria**, **Failing Tests**, and
   **Implementation Context**.
2. The decisions it references: `decisions/DEC-004`, `DEC-005`, `DEC-002`.
3. `guidance/constraints.yaml` — esp. `deterministic-stable-output`,
   `collection-first-substrate`, `no-new-top-level-deps-without-decision`,
   `license-policy`.
4. `guidance/toolchain-brief.md` — Rust/cargo facts (current toolchain, **no
   `serde_yaml`**, order-preserving frontmatter, `clippy -D warnings`).
5. `docs/architecture.md` (parse stage) and `docs/data-model.md` (the `Skill` type).
6. Reference only (do not copy verbatim): `initial_stuff/parse.rs`,
   `initial_stuff/skill.rs`, and `initial_stuff/skillport/lint-fixtures/`.

## Your job

1. Create the first real `Cargo.toml` (edition 2021, current stable toolchain,
   crate name `skillport`). Add the YAML crate + ordered-map crate — **and author
   `DEC-007`** (`decisions/DEC-007-*.md`, copy `decisions/_template.md`) in the
   same pass justifying the crate choice and why not `serde_yaml`
   (`no-new-top-level-deps-without-decision` explicitly allows dep + DEC in one
   pass). Keep deps permissive-licensed (`license-policy`).
2. Implement `src/skill.rs` (`Skill`, `Frontmatter`, `FrontmatterStatus`) and
   `src/parse.rs` (`parse(path, raw) -> Skill`, **total**, never `Result`, never
   panics) exactly to the exported shape in the spec's **Outputs**.
3. Write the tests from the spec's **Failing Tests** (as `#[cfg(test)] mod tests`
   in `src/parse.rs`, plus the fixture-backed case) and make them pass.
4. Keep `parse` a pure function of `(path, raw)` — no filesystem walking here
   (that's a later spec). Add a minimal `src/lib.rs`/`main.rs` so it compiles.

## Definition of done

- Every **Acceptance Criterion** in the spec is met; every **Failing Test** passes.
- `cargo test` green; `cargo clippy -- -D warnings` clean; `cargo fmt --check` clean.
- `DEC-007` written. No rules, emitters, CLI, walker, or tokenizer (all out of scope).
- Fill the spec's **## Build Completion** section; append your **build** cost
  session per `projects/_templates/prompts/cost-snippet.md` (leave numerics null —
  the orchestrator fills them at ship). Then `just advance-cycle SPEC-001 verify`
  and open a PR referencing PROJ-001 / STAGE-001 / SPEC-001.

## Guardrails

- A malformed skill is a `FrontmatterStatus`, **never** a panic or an aborting
  error (DEC-005). Frontmatter stays **order-preserving and typed** (DEC-004/002).
  `raw` is byte-for-byte the input (losslessness). Output deterministic (DEC-005).
