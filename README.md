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
| **`lint` CLI** (`skillport lint <path>`, `--json`, `--strict`, exit codes) | ✅ shipped (SPEC-005) |
| Remaining rules (`metadata.*`, `allowed-tools`, `body.*`, unknown-field) | ⏳ next (SPEC-006) |
| `--sarif` output, `--target <platform>` | ⏳ STAGE-003 |
| `audit` command | ⏳ PROJ-002 |

`skillport lint` **runs today.** The rules it enforces so far are the SPEC-004
batch (frontmatter presence, `name.*`, `description.*`, `compatibility.length`);
`metadata.*`, `allowed-tools`, `body.*`, and unknown-field checks arrive in SPEC-006.

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

skillport lint <path>          # a SKILL.md file, a skill folder, or a whole tree
skillport lint <path> --json   # machine-readable output for CI
skillport lint <path> --strict # treat warnings as failures (affects exit code)
```

Exit codes: **0** clean · **1** on any error (or any warning under `--strict`) ·
**2** usage error (path not found). Findings go to **stdout**; usage errors to
**stderr** — so `--json` on stdout is safe to pipe.

Example (`skillport lint lint-fixtures/bad`, exits 1):

```
lint-fixtures/bad/My-Skill/SKILL.md
  error   description.required [description] — 'description' is required
  error   name.charset [name] — 'name' may only contain lowercase letters, digits, and hyphens (invalid: MS!)
  error   name.hyphen-consecutive [name] — 'name' must not contain consecutive hyphens
  error   name.hyphen-edges [name] — 'name' must not start or end with a hyphen
  warning name.dir-match [name] — 'name' (-My--Skill!) should match the skill directory name (My-Skill)

1 skill(s): 4 error(s), 1 warning(s), 0 info(s)
```

And `--json` (stable `schema: 1`):

```json
{"tool":"skillport","version":"0.1.0","schema":1,"target":null,
 "summary":{"skills":1,"errors":0,"warnings":0,"infos":0},
 "sections":[{"path":"lint-fixtures/good/data-analysis/SKILL.md","findings":[]}]}
```

(The [`examples/lint_demo.rs`](examples/lint_demo.rs) library demo —
`cargo run --example lint_demo -- <path>` — also still works if you want to drive
the library directly.)

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

Apache-2.0 (see [`LICENSE`](LICENSE)) — inherited from the template. (The
prototype declared MIT; the app's final license is a call to confirm before first
release.)
