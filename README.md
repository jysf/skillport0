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

skillport's `lint` command is **feature-complete for STAGE-003** and not yet
released as a binary.

| Piece | State |
|---|---|
| Parser (`SKILL.md` → canonical `Skill`, tolerant + lossless) | ✅ shipped (SPEC-001) |
| Collection tree-walker (`walk` → path-sorted `Collection`) | ✅ shipped (SPEC-002) |
| Report model (`Finding`/`Severity`/`Report`, exit codes) | ✅ shipped (SPEC-003) |
| Rule engine — frontmatter / `name.*` / `description.*` / `compatibility` | ✅ shipped (SPEC-004) |
| **`lint` CLI** (`skillport lint <path>`, `--json`, `--strict`, exit codes) | ✅ shipped (SPEC-005) |
| Full open-spec catalog (`metadata.*`, `allowed-tools.*`, `body.*`, `frontmatter.unknown`) | ✅ shipped (SPEC-006) |
| `dir.unreadable` / `file.unreadable` structural findings | ✅ shipped (SPEC-007) |
| `--sarif` output (GitHub code-scanning) | ✅ shipped (SPEC-008) |
| `--target claude` (Claude Code frontmatter awareness) | ✅ shipped (SPEC-011) |
| `body.size` via a real tokenizer + rule reference docs | ✅ shipped (SPEC-010/SPEC-012) |
| `audit` command | ⏳ PROJ-002 |

`skillport lint` **runs today** and enforces the full open-spec rule catalog
(26 rule ids — see [Rule reference](#rule-reference) below) plus `--target
claude`, `--sarif`, and a real-tokenizer `body.size` check. Only the `audit`
command (library health/security auditing, PROJ-002) remains unbuilt.

## Build & run

Requires a current stable Rust toolchain (edition 2021). Build/dev commands live
in `app.just` (run `just --list` to see them all):

```bash
just build          # cargo build
just build-release  # release binary -> target/release/skillport
just test           # full test suite (unit + integration + fixtures)
just clippy         # cargo clippy --all-targets -- -D warnings
just verify-all     # the pre-ship gate: fmt-check + clippy + test
```

### Using `skillport lint`

```bash
cargo build --release          # -> target/release/skillport
alias skillport=./target/release/skillport   # optional

skillport lint <path>                  # a SKILL.md file, a skill folder, or a whole tree
skillport lint <path> --json           # machine-readable output for CI (stable schema: 1)
skillport lint <path> --sarif          # SARIF 2.1.0 for GitHub code-scanning (mutually exclusive with --json)
skillport lint <path> --strict         # treat warnings as failures (affects exit code only)
skillport lint <path> --target claude  # also recognize Claude Code's frontmatter fields (DEC-002, verified)
```

`--target claude` widens what's recognized (per SPEC-011/SPEC-012), it never
relaxes an open-spec rule: `frontmatter.unknown` also allows the 13
Claude-extension fields (see [Rule reference](#rule-reference)), and
`allowed-tools.format`'s list case downgrades from Warning to Info (Claude
Code accepts a list; the open spec still recommends a string). Every other
rule's severity is unchanged by `--target`.

Exit codes: **0** clean · **1** on any error (or any warning under `--strict`) ·
**2** usage error (path not found, or `--json`/`--sarif` used together).
Findings go to **stdout**; usage errors to **stderr** — so `--json`/`--sarif`
on stdout is always safe to pipe.

Example (`skillport lint lint-fixtures/bad/My-Skill`, exits 1 — regenerated
from the real binary):

```
lint-fixtures/bad/My-Skill/SKILL.md
  error   description.required [description] — 'description' is required
  error   name.charset [name] — 'name' may only contain lowercase letters, digits, and hyphens (invalid: MS!)
  error   name.hyphen-consecutive [name] — 'name' must not contain consecutive hyphens
  error   name.hyphen-edges [name] — 'name' must not start or end with a hyphen
  warning allowed-tools.format [allowed-tools] — the open spec defines 'allowed-tools' as a space-separated string, not a list
  warning name.dir-match [name] — 'name' (-My--Skill!) should match the skill directory name (My-Skill)
  info    frontmatter.unknown [random_field] — 'random_field' is not a recognized field; compliant agents ignore unknown keys
  info    metadata.values [metadata.version] — metadata.version is not a string; the spec defines metadata as string-to-string (quote values like "1.0")

1 skill(s): 4 error(s), 2 warning(s), 2 info(s)
```

And `--json` (stable `schema: 1`; `skillport lint lint-fixtures/good/data-analysis --json`):

```json
{"tool":"skillport","version":"0.1.0","schema":1,"target":null,"summary":{"skills":1,"errors":0,"warnings":0,"infos":0},"sections":[{"path":"lint-fixtures/good/data-analysis/SKILL.md","findings":[]}]}
```

And `--target claude` (`skillport lint lint-fixtures/good-claude --target claude`
— `allowed-tools` as a list downgrades to Info instead of disappearing):

```
lint-fixtures/good-claude/claude-extension/SKILL.md
  info    allowed-tools.format [allowed-tools] — 'allowed-tools' is a list; the open spec expects a space-separated string, but Claude Code accepts a list (source: code.claude.com/docs/en/skills)

1 skill(s): 0 error(s), 0 warning(s), 1 info(s)
```

(The [`examples/lint_demo.rs`](examples/lint_demo.rs) library demo —
`cargo run --example lint_demo -- <path>` — also still works if you want to drive
the library directly.)

## Rule reference

Every rule id `skillport lint` can emit — 24 engine rules (`src/rules.rs`) +
2 structural rules (`src/report.rs`) — with its **default** severity (the
severity with no `--target`) and what fires it. This table is checked against
a code-level catalog (`skillport::RULES`, re-exported from `src/lib.rs`) by
the `readme_rule_table_matches_catalog` test (`tests/cli.rs`), so it cannot
silently drift from the code (DEC-005: rule ids are a public contract).

| rule id | severity | fires when | notes |
|---|---|---|---|
| `frontmatter.missing` | error | no YAML frontmatter block | |
| `frontmatter.unclosed` | error | opening `---` but no closing `---` | |
| `frontmatter.invalid` | error | frontmatter is not a valid YAML mapping | |
| `frontmatter.unknown` | info | a key isn't recognized | open set; under `--target claude`, the 13 Claude-extension fields (`disable-model-invocation`, `user-invocable`, `disallowed-tools`, `model`, `effort`, `context`, `hooks`, `arguments`, `when_to_use`, `argument-hint`, `agent`, `paths`, `shell`) are also recognized |
| `name.required` | error | `name` is missing | |
| `name.type` | error | `name` is not a string | |
| `name.length` | error | `name` not 1–64 characters | |
| `name.charset` | error | `name` has chars outside `[a-z0-9-]` | strict ASCII |
| `name.hyphen-edges` | error | `name` starts or ends with `-` | |
| `name.hyphen-consecutive` | error | `name` contains `--` | |
| `name.dir-match` | warning | `name` ≠ the skill's directory name | |
| `description.required` | error | `description` is missing | |
| `description.type` | error | `description` is not a string | |
| `description.length` | error | `description` empty or > 1024 chars | |
| `description.detail` | info | `description` < 40 chars | recommends stating what + when |
| `compatibility.length` | error | `compatibility` > 500 chars | |
| `compatibility.type` | warning | `compatibility` is not a string | |
| `metadata.type` | warning | `metadata` is not a key-value map | |
| `metadata.values` | info | a `metadata` value is not a string | |
| `allowed-tools.format` | warning | `allowed-tools` given as a YAML list | open spec expects a space-separated string; downgrades to **info** under `--target claude` (Claude Code accepts a list) |
| `allowed-tools.type` | warning | `allowed-tools` is neither a string nor a list | |
| `body.empty` | warning | the `SKILL.md` body is blank | |
| `body.lines` | warning | body > 500 lines | recommends moving detail into `references/` |
| `body.size` | info | body > ~5000 tokens | real `cl100k_base` tokenizer (a proxy — see `decisions/DEC-010`), not a chars/words heuristic |
| `file.unreadable` | error | a `SKILL.md` couldn't be read (e.g. non-UTF-8) | structural — emitted by `report.rs`, not a field rule |
| `dir.unreadable` | warning | a directory in the tree couldn't be read | structural — a coverage gap, not a skill violation |

`lint-fixtures/good/data-analysis` is the designated **spec-perfect** fixture:
it yields `0 error(s), 0 warning(s), 0 info(s)` both with and without
`--target claude`. Every engine rule above is exercised by at least one
committed fixture under `lint-fixtures/` (the 2 structural ids are excused —
they need a non-UTF-8 file / an unreadable directory, already covered by
`report.rs`'s unit tests).

## Use in CI

skillport ships a reusable, composite [GitHub Action](action.yml) that runs
`skillport lint --sarif` and uploads the results to GitHub code-scanning:

```yaml
permissions:
  contents: read
  security-events: write   # required for the SARIF upload

jobs:
  lint-skills:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: jysf/skillport@v0
        with:
          path: skills        # default: "."
          strict: "false"      # treat warnings as failures
          upload-sarif: "true" # upload to code-scanning
```

Findings surface as annotations on the PR and in the repo's **Security ›
Code scanning** tab. The Action needs no secret beyond the default
`GITHUB_TOKEN`. See [`.github/workflows/example-usage.yml`](.github/workflows/example-usage.yml)
for a complete example workflow, and this repo's own [`.github/workflows/ci.yml`](.github/workflows/ci.yml)
for the `rust` / `dogfood` / `licenses` gates that verify skillport itself.

> v0 builds skillport from source (`cargo install --git`) since it isn't on
> crates.io yet — expect the first run to take a minute or two. A released
> binary will make this fast; see `action.yml` for the note.

## Layout

```
src/
  skill.rs    canonical, order-preserving, lossless Skill model
  parse.rs    SKILL.md -> frontmatter + body (tolerant: BOM/CRLF/missing/unclosed/invalid)
  walk.rs     a path -> a Collection of skills (skips .git/node_modules/target; never aborts)
  report.rs   Finding / Severity / sectioned N-skill Report + exit codes; Report::from_collection
  rules.rs    the open-spec rule engine (lint_skill = the rule_fn from_collection consumes)
  emit.rs     render a Report: human(&Report) and json(&Report) (the --json schema)
  main.rs     the `skillport lint` CLI (clap): walk -> rules -> report -> emit -> exit code
  lib.rs      library root tying the substrate together
examples/
  lint_demo.rs  `cargo run --example lint_demo` (drives the library directly)
lint-fixtures/  good/ + bad/ + warn-only/ example skills (tests + demo)
tests/
  cli.rs        end-to-end tests that run the built binary
```

Design docs: [`docs/architecture.md`](docs/architecture.md),
[`docs/data-model.md`](docs/data-model.md),
[`docs/api-contract.md`](docs/api-contract.md). Rationale:
[`decisions/`](decisions/) (DEC-001…008).

## How this repo is built

skillport is developed with a spec-driven workflow (Claude as architect /
implementer / reviewer across fresh sessions). That meta-process — the
Repo → Project → Stage → Spec → Cycle hierarchy and the `just` workflow commands —
is documented separately in [`docs/WORKFLOW.md`](docs/WORKFLOW.md).

## License

Dual-licensed under either of

- [MIT license](LICENSE-MIT)
- [Apache License, Version 2.0](LICENSE-APACHE)

at your option.
