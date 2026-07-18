# skillport

**A fast Rust CLI that validates and audits agent Skills (`SKILL.md` files).**

skillport answers two questions about agent skills:

- **"Does this skill conform?"** — `lint` checks a single skill, a folder, or a
  whole tree against the open [Agent Skills spec](https://agentskills.io/specification),
  with three severities and CI-friendly exit codes. *(PROJ-001)*
- **"How healthy and how risky is this *collection* of skills?"** — `audit`
  produces a human-read report over a skill library: inventory, description
  overlap, a permissions manifest (what each skill can do), and hash-anchored
  provenance/drift detection. *(PROJ-002)*

The differentiated value is **validation + normalization + library/security
audit** with per-platform awareness and bulk/CI ergonomics — deliberately *not*
a converter (that lane is already crowded; see `decisions/DEC-001`). Only the
open spec is authoritative; per-platform constraints are advisory until
confirmed from that platform's primary docs (`decisions/DEC-002`).

Build: `cargo build --release` → `target/release/skillport`. See `AGENTS.md`
§5–6 for the toolchain and commands.

> **Status:** PROJ-001 (foundation + `lint`) is in Frame/Design. No `src/` yet.

---

*The rest of this file documents the spec-driven meta-process used to build
skillport, where Claude plays every role (architect, implementer, reviewer)
across different sessions.*

## Hierarchy

```
Repo (this app)
 └─ Project (a wave of work: "MVP", "v2 improvements")
     └─ Stage (a coherent chunk within a project)
         └─ Spec (an individual task)
              └─ Cycle (Frame → Design → Build → Verify → Ship)
```

## Getting started

**First time?** Read `GETTING_STARTED.md` — it walks you through your first project end-to-end.

**Daily work?** Run `just --list` to see available commands.

**Common commands:**
```bash
just status                        # See active project, stage, specs by cycle
just backlog                       # Spec-grained: what's next in the active stage
just roadmap                       # Stage-grained: where this project is going
just new-spec "title" STAGE-001    # Scaffold a new spec
just advance-cycle SPEC-001 verify # Update a spec's cycle
just archive-spec SPEC-001         # Move a shipped spec to done/
just review                        # Print the weekly review prompt
just report daily                  # Generate today's daily report
just report weekly                 # Generate this week's weekly report
just report status                 # Snapshot `just status` to reports/daily/<date>-status.md
```
`report-daily` / `report-weekly` remain as permanent aliases for
`report daily` / `report weekly`.

## Reports

`just report-daily` and `just report-weekly` generate quantitative
snapshots under `reports/daily/` and `reports/weekly/` from spec
front-matter and git log. Daily reports show specs by cycle, value
thesis, cost activity today, and flags. Weekly reports aggregate
ships, cycle times, cost by cycle and interface, and value
advancement. Reports are stand-alone artifacts — re-running
overwrites, so they're always a current snapshot.

## Key discipline in this variant

Because Claude plays every role, context contamination is the biggest risk. Four habits keep it at bay:

1. **New session per cycle** (especially design → build and build → verify)
2. **The spec file is the source of truth** between sessions — no "as I said earlier"
3. **Weekly review is non-optional** (`just review`)
4. **Honest confidence values** on decisions

See `AGENTS.md` section 15 for the full discipline.

## The app itself

skillport (described at the top of this file) is a Rust CLI for validating and
auditing agent `SKILL.md` files. Run it locally with `cargo run -- lint <path>`
and the release binary with `cargo build --release`; run tests with
`cargo test`. Full toolchain and command list: `AGENTS.md` §5–6.

## Where things live

| Path | Purpose |
|---|---|
| `AGENTS.md` | Conventions for Claude working in this repo |
| `.repo-context.yaml` | Structured metadata about the app |
| `docs/` | Architecture, data model, API contract |
| `guidance/` | Repo-level rules, open questions, and the signals ledger (`just dash signals`) |
| `decisions/` | Decision log (accumulates across projects) |
| `projects/` | Each project (wave of work) lives here |
| `projects/*/brief.md` | What this project is and why |
| `projects/*/stages/` | Stages within a project |
| `projects/*/specs/` | Specs within a project (with folded-in Implementation Context) |
| `src/` | The Rust CLI (created by PROJ-001 build specs) |

## License

Apache-2.0 (see `LICENSE`) — inherited from the template. (The prototype
declared MIT; the app's final license is a call to confirm before first
release.)
