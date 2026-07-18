# skillport

**A fast Rust tool that validates and audits agent Skills (`SKILL.md` files).**

skillport answers two questions about agent skills:

- **"Does this skill conform?"** — `lint` checks a single skill, a folder, or a
  whole tree against the open [Agent Skills spec](https://agentskills.io/specification),
  with three severities (error / warning / info) and CI-friendly exit codes.
  *(PROJ-001)*
- **"How healthy and how risky is this *collection* of skills?"** — `audit`
  produces a human-read report over a skill library: inventory, description
  overlap, a permissions manifest (what each skill can do), and hash-anchored
  provenance/drift detection. *(PROJ-002)*

The differentiated value is **validation + normalization + library/security
audit** with per-platform awareness and bulk/CI ergonomics — deliberately *not* a
converter (that lane is already crowded; see [`decisions/DEC-001`](decisions/DEC-001-not-a-converter.md)).
Only the open spec is authoritative; per-platform constraints are advisory until
confirmed from that platform's primary docs ([`decisions/DEC-002`](decisions/DEC-002-open-spec-authoritative.md)).

## Status

skillport is **mid-build** and not yet released.

| Piece | State |
|---|---|
| Parser (`SKILL.md` → canonical `Skill`, tolerant + lossless) | ✅ shipped (SPEC-001) |
| Collection tree-walker (`walk` → path-sorted `Collection`) | ✅ shipped (SPEC-002) |
| Report model (`Finding`/`Severity`/`Report`, exit codes) | ✅ shipped (SPEC-003) |
| Rule engine — frontmatter / `name.*` / `description.*` / `compatibility` | ✅ shipped (SPEC-004) |
| Remaining rules (`metadata.*`, `allowed-tools`, `body.*`, unknown-field) | ⏳ next (SPEC-005) |
| **`lint` CLI** (arg parsing, `--json`, `--strict`, exit codes) | ⏳ pending |
| `audit` command | ⏳ PROJ-002 |

There is **no `skillport lint` binary yet** — `src/main.rs` is a stub until the
CLI spec lands. To see the shipped substrate validate real skills today, use the
demo below.

## Build & run

Requires a current stable Rust toolchain (edition 2021). Build/dev commands live
in `app.just` (run `just --list` to see them all):

```bash
just build          # cargo build
just build-release  # release binary -> target/release/skillport (currently a stub)
just test           # full test suite (unit + integration + fixtures)
just clippy         # cargo clippy --all-targets -- -D warnings
just fmt-check      # formatting check
just verify-all     # the pre-ship gate: fmt-check + clippy + test
just doc            # open the API docs to eyeball the substrate
```

Plain cargo works too: `cargo test`, `cargo build --release`, etc.

### See it work today (the example)

Until the real CLI ships, run the example
([`examples/lint_demo.rs`](examples/lint_demo.rs)) — it drives the shipped library
(`walk` → `Report::from_collection(.., lint_skill)`) over a path and prints findings:

```bash
cargo run --example lint_demo                    # lints ./lint-fixtures (a good and a bad skill)
cargo run --example lint_demo -- path/to/skills  # lint any file / folder / tree
```

Sample output:

```
lint-fixtures/bad/My-Skill/SKILL.md
  error   description.required [description] — 'description' is required
  error   name.charset [name] — 'name' may only contain lowercase letters, digits, and hyphens (invalid: MS!)
  error   name.hyphen-consecutive [name] — 'name' must not contain consecutive hyphens
  error   name.hyphen-edges [name] — 'name' must not start or end with a hyphen
  warning name.dir-match [name] — 'name' (-My--Skill!) should match the skill directory name (My-Skill)

lint-fixtures/good/data-analysis/SKILL.md
  ✓ no findings

2 skill(s): 4 error(s), 1 warning(s), 0 info(s)
would-be CI exit code: 1 (non-strict) / 1 (--strict)
```

The rules shown are the SPEC-004 batch; `metadata.*`, `allowed-tools`, `body.*`,
and unknown-field checks arrive in SPEC-005.

## Layout

```
src/
  skill.rs    canonical, order-preserving, lossless Skill model
  parse.rs    SKILL.md -> frontmatter + body (tolerant: BOM/CRLF/missing/unclosed/invalid)
  walk.rs     a path -> a Collection of skills (skips .git/node_modules/target; never aborts)
  report.rs   Finding / Severity / sectioned N-skill Report + exit codes; Report::from_collection
  rules.rs    the open-spec rule engine (lint_skill = the rule_fn from_collection consumes)
  lib.rs      library root (the substrate); main.rs is a stub until the CLI ships
examples/
  lint_demo.rs  `cargo run --example lint_demo` (stand-in for the CLI)
lint-fixtures/  good/ + bad/ example skills (tests + demo)
```

Design docs: [`docs/architecture.md`](docs/architecture.md),
[`docs/data-model.md`](docs/data-model.md),
[`docs/api-contract.md`](docs/api-contract.md). Rationale:
[`decisions/`](decisions/) (DEC-001…007).

## How this repo is built

skillport is developed with a spec-driven workflow (Claude as architect /
implementer / reviewer across fresh sessions). That meta-process — the
Repo → Project → Stage → Spec → Cycle hierarchy and the `just` workflow commands —
is documented separately in [`docs/WORKFLOW.md`](docs/WORKFLOW.md).

## License

Apache-2.0 (see [`LICENSE`](LICENSE)) — inherited from the template. (The
prototype declared MIT; the app's final license is a call to confirm before first
release.)
