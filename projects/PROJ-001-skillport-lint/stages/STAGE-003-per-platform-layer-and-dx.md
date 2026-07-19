---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-003                     # stable, zero-padded, continuous across the repo
  status: active                  # proposed | active | shipped | cancelled | on_hold
  priority: medium                  # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-001                      # parent project
repo:
  id: skillport

created_at: 2026-07-17
shipped_at: null

value_contribution:
  advances: "Differentiation beyond the table-stakes open layer — verified per-platform awareness and CI ergonomics — plus the DX that makes lint trustworthy and adoptable."
  delivers:
    - "`--target claude` recognized-field widening, verified from primary docs"
    - "accurate body.size via a real tokenizer (info)"
    - "--sarif output and a ready-to-use GitHub Action"
    - "README (rule ids/severities), per-rule tests + good/bad fixtures"
  explicitly_does_not:
    - "verify Cursor/Codex/Vercel (advisory-only this wave; DEC-002)"
    - "ship --fix autofix (deferred)"
    - "add any audit/collection-level analysis (PROJ-002)"
---

# STAGE-003: Per-platform layer + DX

## What This Stage Is

The stage that turns a correct open-spec linter into a differentiated,
adoptable tool: a **verified** `--target claude` layer that widens which
frontmatter fields are "recognized" (so a real Claude extension isn't flagged
unknown), an **accurate** `body.size` using a real tokenizer, **SARIF** output
and a **GitHub Action** for CI, and the DX that makes it trustworthy — a README
documenting every rule id + severity, per-rule unit tests, good/bad fixtures, and
a test proving a spec-perfect skill yields zero findings.

## Why Now

Ships last because it widens and dresses a working validator (STAGE-002) rather
than creating one. The per-platform layer is where skillport's differentiation
begins (the open layer is table stakes), and it is also where the correctness
discipline matters most (DEC-002) — so it gets its own stage with real primary-
doc work, not a rushed add-on.

## Decisions locked at Frame (answered open questions)

1. **First `--target` = Claude Code.** Verify Claude's recognized frontmatter
   fields from **docs.claude.com** before encoding them. Other platforms
   (Cursor/Codex/Vercel) remain advisory field-recognizers until similarly
   verified — never emitted as errors/warnings (DEC-002).
2. **`body.size` uses a real tokenizer** (accurate token count), still emitted at
   **info** severity (DEC-003). The tokenizer is a new runtime dependency →
   author a DEC for the crate choice in the same pass
   (`no-new-top-level-deps-without-decision`).
3. **Extras pulled into this wave:** **SARIF** output and a **GitHub Action** are
   IN. **`--fix` autofix stays OUT** (deferred — needs a lossless round-trip
   writer + safe-fix selection).

## Success Criteria

- `--target claude` widens `frontmatter.unknown` (and downgrades
  `allowed-tools.format` to info **iff** Claude is confirmed to accept a list) —
  each backed by a cited docs.claude.com line + a source comment in code.
- Any per-platform behavior not confirmed from primary docs is emitted at **info**
  with a source note, never error/warning (DEC-002).
- `body.size` reports an accurate token count via a real tokenizer, info-level.
- `--sarif` emits valid SARIF over the same findings; schema stable (DEC-005).
- A GitHub Action / workflow snippet runs `lint` in CI with correct exit codes.
- README documents every rule id + severity + `--target`/`--strict`/output flags.
- Per-rule unit tests + good/bad fixtures; a test asserting a **spec-perfect skill
  yields zero findings**.

## Scope

### In scope
- `--target claude` recognized-field set, verified from docs.claude.com.
- Real-tokenizer `body.size` (info).
- `--sarif` emitter.
- GitHub Action / CI workflow.
- README (rule ids/severities/flags), per-rule tests, good/bad fixtures,
  zero-findings-on-a-perfect-skill test.

### Explicitly out of scope
- Verifying/encoding Cursor, Codex, Vercel constraints as firm (advisory-only;
  DEC-002) — a later wave can verify each from its primary docs.
- `--fix` autofix (deferred).
- Anything audit/collection-level (PROJ-002).

## Spec Backlog

> Proposed decomposition — the Design cycle turns these into specs via
> `just new-spec "<title>" STAGE-003`. Not yet scaffolded.

- [x] SPEC-011 (shipped 2026-07-18, PR #11) — `--target claude` (first verified
  per-platform layer, DEC-002): recognized-field widening from
  **code.claude.com/docs/en/skills** (each fact source-commented) +
  `allowed-tools.format`→info (Claude accepts a YAML list, confirmed) + `--json`
  `target:"claude"`. Open-spec rules unchanged. Verify APPROVED (0 punch-list);
  flagged a non-blocking gap — 5 more documented fields (`when_to_use`,
  `argument-hint`, `agent`, `paths`, `shell`) not yet in `CLAUDE_KEYS`
  (enumeration-widening follow-up; decide at stage close).
- [x] SPEC-010 (shipped 2026-07-18, PR #10) — Real-tokenizer `body.size` (info):
  `body_token_count` via `tiktoken-rs` cl100k_base (proxy; DEC-010), threshold
  ~5000, in `check_body`. **Open-spec catalog now 100% implemented.**
- [x] SPEC-008 (shipped 2026-07-18, PR #8) — `--sarif` emitter (SARIF 2.1.0):
  `emit::sarif` + `--sarif` flag (mutually exclusive with `--json`); level map
  info→note; distinct/sorted rules; no new dep. Drops into GitHub code-scanning.
- [x] SPEC-009 (shipped 2026-07-18, PR #9) — reusable composite **GitHub Action**
  (`skillport lint --sarif` + upload to code-scanning) + this repo's **Rust CI**
  (fmt/clippy/test) + dogfood + `licenses` (cargo-deny) jobs + README "Use in CI".
  Mechanized the `license-policy` constraint. No Rust change.
- [x] SPEC-012 (shipped 2026-07-18, PR #12) — **DX capstone.** A code rule *catalog* (single source of
  truth over all 26 rule ids) + README refresh (current Status table, a **Rule
  reference** table checked against the catalog by a drift test, `--target`/`--sarif`/
  `--strict` flags, regenerated real example output) + per-rule fixtures + a
  spec-perfect zero-findings fixture/test. Also **folds in** the SPEC-011
  enumeration follow-up: completes `CLAUDE_KEYS` with the 5 remaining doc-verified
  Claude fields (`when_to_use`, `argument-hint`, `agent`, `paths`, `shell` → 13
  total). No new rules/severities/ids; no new dep. Design did the doc re-verification
  + full severity probe.

**Count:** 5 shipped / 0 active / 0 pending. **STAGE-003 backlog is COMPLETE** —
SPEC-008 (`--sarif`), SPEC-009 (GitHub Action + CI), SPEC-010 (tokenizer `body.size`),
SPEC-011 (`--target claude`), SPEC-012 (rule-reference README + fixtures + complete
CLAUDE_KEYS) all shipped. Ready to close the stage (Prompt 1c: stage reflection +
disposition the open watch signals). The `--target claude` enumeration follow-up was
folded into SPEC-012.

## Design Notes

- The prototype's `profiles.rs` is the shape to reuse but its
  claude/cursor/codex/vercel entries are **unverified guesses** — treat only the
  Claude entry, once confirmed from docs, as firm; leave the rest advisory
  (DEC-002).
- SARIF is the cheapest extra: another emitter over the existing findings model —
  no new analysis.
- Firm constraints: `only-verified-constraints-are-firm`, `no-heuristic-error`,
  `deterministic-stable-output`, `license-policy` (cargo-deny for the new deps);
  DEC-002, DEC-003, DEC-005.

## Dependencies

### Depends on
- STAGE-002 (the `lint` command + rule engine + `frontmatter.unknown` /
  `allowed-tools.format` seams this stage widens).
- External: docs.claude.com (primary-doc verification of Claude fields).

### Enables
- A shippable PROJ-001; PROJ-002 (`audit`) builds on the same substrate.

## Stage-Level Reflection

*Filled in when status moves to shipped.*

- **Did we deliver the outcome in "What This Stage Is"?** <not yet>
- **How many specs did it actually take?** <not yet>
- **What changed between starting and shipping?** <not yet>
- **Lessons that should update AGENTS.md, templates, or constraints?** <not yet>
- **Signals dispositioned at this close?** <not yet>
- **Should any spec-level reflections be promoted to stage-level lessons?** <not yet>
