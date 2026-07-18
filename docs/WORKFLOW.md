# How this repo is built — the spec-driven workflow

skillport is built with the [`spec-driven-template`](https://github.com/jysf/spec-driven-template)
(claude-only variant): a spec-driven workflow where Claude plays every role
(architect, implementer, reviewer) across different sessions. This file documents
that *meta-process*. For what skillport **is**, see the [README](../README.md);
for conventions, see [`AGENTS.md`](../AGENTS.md).

## Hierarchy

```
Repo (skillport — the app)
 └─ Project (a wave of work: PROJ-001 lint, PROJ-002 audit)
     └─ Stage (a coherent chunk within a project)
         └─ Spec (an individual task)
              └─ Cycle (Frame → Design → Build → Verify → Ship)
```

## Getting started

**First time?** Read `GETTING_STARTED.md` — it walks through a project end-to-end.

**Daily work?** Run `just --list` to see available commands.

**Common workflow commands:**
```bash
just status                        # active project, stage, specs by cycle
just backlog                       # spec-grained: what's next in the active stage
just roadmap                       # stage-grained: where this project is going
just new-spec "title" STAGE-001    # scaffold a new spec
just advance-cycle SPEC-001 verify # update a spec's cycle
just archive-spec SPEC-001         # move a shipped spec to done/
just review                        # print the weekly review prompt
just report daily                  # generate today's daily report
just report weekly                 # generate this week's weekly report
just cost-audit                    # gate: every shipped spec has real build/verify cost
```
(App/build commands — `just build`, `just test`, `just clippy`, … — live in
`app.just`; run `just --list` to see both sets together.)

## Reports

`just report-daily` / `just report-weekly` generate quantitative snapshots under
`reports/` from spec front-matter and the git log — specs by cycle, value thesis,
cost by cycle/interface, cycle times, flags. They're stand-alone artifacts;
re-running overwrites, so they're always a current snapshot.

## Key discipline in this variant

Because Claude plays every role, context contamination is the biggest risk. Four
habits keep it at bay:

1. **New session per cycle** (especially design → build and build → verify). In
   skillport this is realized by running **build as a metered Sonnet subagent** and
   **verify as a metered Opus subagent** — fresh context each time, and real token
   cost is captured (see `guidance/signals.yaml` → `cost-metering-manual-sessions`).
2. **The spec file is the source of truth** between sessions — no "as I said earlier".
3. **Weekly review is non-optional** (`just review`).
4. **Honest confidence values** on decisions.

See `AGENTS.md` §15 for the full discipline.

## Where things live

| Path | Purpose |
|---|---|
| `README.md` | What skillport is + how to build/run/test it |
| `AGENTS.md` | Conventions for Claude working in this repo |
| `.repo-context.yaml` | Structured metadata about the app |
| `app.just` | skillport's build/test/run/demo commands (project-owned) |
| `justfile` | Template-managed workflow commands (`status`, `new-spec`, …) |
| `docs/` | Architecture, data model, CLI/output contract, this workflow doc |
| `guidance/` | Repo-level rules, open questions, and the signals ledger |
| `decisions/` | Decision log (DEC-*, accumulates across projects) |
| `projects/` | Each project (wave of work); `brief.md`, `stages/`, `specs/` |
| `src/` | The Rust library + (eventually) the CLI |
| `lint-fixtures/` | good/ + bad/ example skills used by tests and the demo example |
