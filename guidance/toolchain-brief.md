# Toolchain Brief

> **Per-repo toolchain facts a cold build sub-agent needs.** A fresh
> build/verify sub-agent re-imports its model's generic tool-priors and burns
> loops rediscovering this repo's specifics. Fill it in once, keep it short and
> current, and inject it into every build prompt (see AGENTS.md §15). Prune
> aggressively — a stale fact here wastes the loop it was meant to save.

> **Status:** skillport has no `src/` yet (first build spec is STAGE-001). Facts
> below are the *intended* toolchain from Project Design; the first build spec
> establishes them for real — update this file if reality diverges.

## Package manager

Cargo (Rust, **edition 2021**). Use the **current stable** toolchain. Add deps
with `cargo add`. `Cargo.lock` is committed and CI-enforced.

- **Do NOT** copy the prototype's `=`-exact version pins (`clap = "=4.5.20"`,
  `serde_yaml = "=0.9.34"`, `indexmap = "=2.2.6"`) — they were a Rust-1.75
  artifact. Use current caret ranges (DEC-005).
- **Do NOT** use `serde_yaml` — it is **deprecated/unmaintained**. Pick a current
  maintained YAML crate during the parser spec and record the choice in a DEC
  (`no-new-top-level-deps-without-decision`). Frontmatter must stay
  **order-preserving** (an index-map-style structure, not `HashMap`) and lossless.

## Test framework + assertion library

Built-in `cargo test` — no external test runner. Unit tests in a
`#[cfg(test)] mod tests` beside the code; integration tests in `tests/`.

- Full suite: `cargo test`. Single test: `cargo test <name>`.
- Rule tests exercise real `SKILL.md` files under `lint-fixtures/good/` and
  `lint-fixtures/bad/` (seed set exists in the prototype under `initial_stuff/`).
- Required invariant test (STAGE-003): a **spec-perfect skill yields zero findings**.

## Lint / format quirks

- `cargo fmt` owns formatting — don't hand-align; run `cargo fmt --check` before ship.
- `cargo clippy -- -D warnings` runs in CI and **fails on any warning**. No
  `unwrap`/`expect` on fallible IO/parse paths — a malformed skill is a *finding*,
  never a panic (DEC-005).

## Runtime globals / environments

Single native binary; no runtime env. **stdout = the report** (human/JSON/SARIF),
**stderr = diagnostics** — never print diagnostics to stdout or machine consumers
break. Output must be deterministic and **path-sorted** (DEC-005).

## Installed test/dev utilities (don't re-add)

Nothing yet (no `Cargo.toml` in the repo root — the prototype's is under
`initial_stuff/`). When STAGE-001 lands, list here what's already a dependency so
a cold agent doesn't re-add it. Likely core deps: a CLI parser (`clap` 4, derive),
`serde` + a maintained YAML crate, an ordered-map crate, an error crate
(`anyhow` at the CLI boundary). A tokenizer crate arrives in STAGE-003 (`body.size`).

## Known gotchas

- The prototype under `initial_stuff/` is **converter-first** (`inspect`/`convert`/
  `push`/`profiles`) — that machinery is out of scope (DEC-001). Reuse only
  `lint.rs` + `lint-fixtures/`; port them onto the collection-first substrate.
- The prototype's `claude/cursor/codex/vercel` profiles are **unverified guesses**
  — do not encode them as errors/warnings; Claude fields must be verified from
  docs.claude.com first, others stay advisory (DEC-002).
- Rule ids and the `--json`/`--sarif` schema are a **public contract** — renaming a
  rule id or changing a field is a MAJOR bump, not a casual edit (DEC-005).
