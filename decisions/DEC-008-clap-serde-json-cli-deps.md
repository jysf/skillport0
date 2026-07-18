---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-008
  type: decision
  confidence: 0.9
  audience:
    - developer
    - agent

agent:
  id: claude-sonnet-5
  session_id: null

project:
  id: PROJ-001
repo:
  id: skillport

created_at: 2026-07-18
supersedes: null
superseded_by: null

affected_scope:
  - "Cargo.toml"
  - "src/main.rs"
  - "src/emit.rs"

tags:
  - dependencies
  - cli
  - json
  - license
---

# DEC-008: CLI arg parsing with `clap` (derive), `--json` output with `serde` + `serde_json`

## Decision

The real `skillport lint` CLI (SPEC-005) uses **`clap`** (derive API,
`features = ["derive"]`) for argument parsing, and **`serde`** (derive) +
**`serde_json`** for the stable `--json` output. All three are added as direct
runtime dependencies with caret ranges (`clap = "4"`, `serde = "1"`,
`serde_json = "1"`), consistent with DEC-005/DEC-007's toolchain conventions
(current stable, no exact pins).

`serde` is declared **directly** even though it was already present
transitively via `serde_yaml_ng` (DEC-007): SPEC-005 uses it directly
(`#[derive(Serialize)]` on emitter-local DTOs in `src/emit.rs`), so it becomes
a first-class dependency of this crate, not just an implementation detail of
`serde_yaml_ng`.

## Context

SPEC-005 turns the substrate into a real `skillport lint <PATH> [--json]
[--strict]` command (`docs/api-contract.md`). Two runtime needs: (1) parse a
CLI with a subcommand structure that leaves room for `audit` later (DEC-001:
lint only, no converter subcommands), and (2) serialize the report into the
exact stable `--json` schema (DEC-005: a public CI contract). Both are new
runtime deps, so `no-new-top-level-deps-without-decision` requires this DEC in
the same build pass.

## Alternatives Considered

- **Option A: Hand-rolled arg parsing (`std::env::args`) + hand-rolled JSON
  string building**
  - What it is: what `initial_stuff/main.rs` avoided doing but
    `initial_stuff/emit.rs`'s `print_json` did — manual `push_str`/escaping.
  - Why rejected: brittle (no `--help`/usage errors, no escaping
    correctness guarantees for arbitrary skill content — e.g. control
    characters in a `message` string), more surface area to get wrong than a
    well-audited crate, and the exact `--json` schema needs typed,
    machine-checked serialization to stay a trustworthy CI contract.

- **Option B: `pico-args` / `lexopt` (minimal arg parsers) + hand-rolled JSON**
  - What it is: lighter-weight CLI parsing without clap's derive machinery.
  - Why rejected: saves little compile time for a single-subcommand CLI today,
    but loses clap's derive ergonomics for the `audit` subcommand PROJ-002
    will add, and doesn't solve the JSON serialization problem at all.

- **Option C (chosen): `clap` (derive) + `serde` (derive) + `serde_json`**
  - What it is: the standard, widely-audited Rust CLI/JSON stack; `clap`'s
    `Parser`/`Subcommand` derive gives `--help`, `--version`, and exit-2 usage
    errors for free; `serde_json::to_string` on `#[derive(Serialize)]` DTOs
    guarantees correct JSON escaping and stable field ordering (`serde_json`
    preserves struct field declaration order, matching the documented schema
    key order).
  - Why selected: least implementation risk for a contract that's explicitly
    "public, semver-governed" (DEC-005); both crates are the de-facto standard
    with large audiences, so bugs surface and get fixed fast; clap's
    `Subcommand` enum is exactly the shape needed to add `audit` later without
    restructuring.

## Consequences

- **Positive:** correct JSON escaping/schema for free; `--help`/usage-error
  handling (clap maps bad args to a non-zero exit, which the CLI layer then
  normalizes to exit 2 per the spec's exit-code table); a `Commands` enum
  ready for `audit` (PROJ-002) without touching `Lint`'s shape.
- **Negative:** two more direct deps (plus `serde`, elevated from transitive to
  direct) to track for license/security; slightly larger binary and compile
  time than a hand-rolled parser.
- **Neutral:** `serde` was already resolved transitively via `serde_yaml_ng`
  (DEC-007), so declaring it directly does not add a new crate to the
  dependency tree, only a direct edge to one already present.

## Validation

Right if `skillport lint` behaves like a normal Unix CLI (usage errors, exit
2, correct `--json` escaping/shape) without hand-maintained parsing/formatting
code. Revisit only if `clap`'s binary-size/compile-time cost becomes a real
constraint for distribution (unlikely for a static single-binary CI tool).

## License compliance (`license-policy`)

All resolved crates are permissive (verified via `cargo metadata`):

- `clap` — MIT OR Apache-2.0
- `clap_derive`, `clap_builder`, `clap_lex` (transitive) — MIT OR Apache-2.0
- `serde`, `serde_derive` — MIT OR Apache-2.0
- `serde_json` — MIT OR Apache-2.0
- `anstyle`, `anstream`, `strsim`, `heck`, `is_terminal_polyfill`, `utf8parse`,
  `colorchoice`, `anstyle-parse`, `anstyle-query` (transitive, via `clap`) —
  MIT OR Apache-2.0
- `itoa`, `ryu` (transitive, via `serde_json`; already present via
  `serde_yaml_ng`) — MIT OR Apache-2.0 (Apache-2.0 taken for `ryu`, as in
  DEC-007)

No copyleft dependency introduced.

## References

- Related specs: SPEC-005 (lint command with human and json output)
- Related decisions: DEC-001 (lint only, no converter subcommands), DEC-005
  (deterministic, stable `--json` schema), DEC-007 (permissive-dep precedent,
  `serde` transitive origin)
- Constraints: `no-new-top-level-deps-without-decision`, `license-policy`,
  `deterministic-stable-output`
- External: <https://crates.io/crates/clap>, <https://crates.io/crates/serde>,
  <https://crates.io/crates/serde_json>
