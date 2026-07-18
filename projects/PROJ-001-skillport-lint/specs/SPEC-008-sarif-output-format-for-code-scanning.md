---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-008
  type: story                      # epic | story | task | bug | chore
  cycle: design                    # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: M                    # S | M | L  (L means split it)

project:
  id: PROJ-001
  stage: STAGE-003
repo:
  id: skillport

agents:
  architect: claude-opus-4-8      # design cycle (this orchestrator session)
  implementer: claude-sonnet-4-6  # build runs as a Sonnet subagent (cost); updated with the real model
  created_at: 2026-07-18

references:
  decisions:
    - DEC-003   # severity -> SARIF level mapping
    - DEC-005   # deterministic, stable output (SARIF is another machine format)
  constraints:
    - deterministic-stable-output
    - test-before-implementation
  related_specs:
    - SPEC-003  # Report/Finding/Severity being rendered
    - SPEC-005  # the lint CLI + emit.rs (json/human) this extends

value_link: "CI-ergonomics differentiation — a third output format (SARIF 2.1.0) so `skillport lint` drops straight into GitHub code-scanning / any SARIF consumer"

# Self-reported AI cost per cycle. Each cycle (design, build, verify,
# ship) appends one entry to sessions[]. Totals are computed at ship.
# Record a REAL tokens_total for metered cycles (build/verify) — the
# orchestrator fills it from the Agent result's subagent_tokens at ship
# (or /cost interactively). Only un-metered main-loop cycles (design/ship)
# may be null-with-note. `just cost-audit` enforces this on shipped specs.
# See AGENTS.md §4 and docs/cost-tracking.md. interface: claude-code |
# claude-ai | api | ollama | other.
cost:
  sessions:
    - cycle: design
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-07-18
      notes: "main-loop, not separately metered (design cycle)"
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-008: sarif output format for code scanning

## Context

First STAGE-003 spec, and the cheapest CI-ergonomics win: a **third output
format**. `skillport lint` already emits human text and `--json` (SPEC-005). This
adds **`--sarif`** — [SARIF 2.1.0](https://sarifweb.azurewebsites.net/), the format
GitHub code-scanning and most CI security tooling ingest — so findings show up as
inline PR annotations. It's a pure render over the existing `Report` (no new
analysis, no new rules), mirroring the `emit::json` DTO pattern.

- Parent stage: `STAGE-003` (per-platform + DX), the SARIF backlog item.
- Reuses: `Report`/`Section`/`Finding`/`Severity` (SPEC-003), the `emit` module +
  the CLI dispatch (SPEC-005). `serde_json` is already a dependency (DEC-008) — **no
  new dependency**.
- Reference: `initial_stuff/emit.rs` if it has SARIF (likely not — the prototype
  had human/JSON); implement to the SARIF 2.1.0 spec regardless.

## Goal

Add `skillport lint <PATH> --sarif` emitting a valid **SARIF 2.1.0** log of the
findings (mutually exclusive with `--json`), via a new `emit::sarif(&Report) ->
String`, with the same exit-code behavior as the other formats.

## Inputs

- **Files to read (extend):** `src/emit.rs` (mirror the `json` DTO approach for
  `sarif`), `src/main.rs` (add the `--sarif` flag + dispatch), `src/report.rs`
  (`Report`/`Finding`/`Severity`).
- **Output contract to update:** `docs/api-contract.md` has a placeholder `--sarif`
  section — replace it with the shape below.
- **SARIF reference:** SARIF 2.1.0 (OASIS). The minimal valid shape is specified below.

## Outputs

- **Files modified:**
  - `src/emit.rs` — `pub fn sarif(report: &Report) -> String` + emitter-local
    `#[derive(Serialize)]` DTO structs (a `SarifLog` tree). Mirror `json`'s style;
    `report.rs` stays serde-free.
  - `src/main.rs` — add `--sarif` to `Lint` (**mutually exclusive with `--json`** via
    clap `conflicts_with`); dispatch to `emit::sarif` before `json`/human.
  - `src/lib.rs` — re-export `sarif` (next to `human`/`json`) if useful for tests.
  - `docs/api-contract.md` — replace the `--sarif` placeholder with the real shape.
- **No new dependency** (`serde_json` already present). **No new rules/analysis.**
- **Database changes:** none.

## SARIF 2.1.0 shape (implement exactly)

```json
{
  "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
  "version": "2.1.0",
  "runs": [
    {
      "tool": {
        "driver": {
          "name": "skillport",
          "informationUri": "https://github.com/jysf/skillport",
          "version": "0.1.0",
          "rules": [
            { "id": "name.charset" },
            { "id": "description.required" }
          ]
        }
      },
      "results": [
        {
          "ruleId": "name.charset",
          "level": "error",
          "message": { "text": "'name' may only contain lowercase letters, digits, and hyphens (invalid: MS!)" },
          "locations": [
            { "physicalLocation": { "artifactLocation": { "uri": "lint-fixtures/bad/My-Skill/SKILL.md" } } }
          ]
        }
      ]
    }
  ]
}
```
- **`level` mapping (DEC-003):** `Error → "error"`, `Warning → "warning"`,
  `Info → "note"` (SARIF's levels).
- **`version`** = `env!("CARGO_PKG_VERSION")`; `driver.name` = `"skillport"`;
  `informationUri` = the repo URL.
- **`rules`** (`reportingDescriptor[]`): the **distinct** rule ids that appear in
  the report's findings, **sorted by id** (determinism). `{ "id": <ruleId> }` is
  enough (a `helpUri`/description can come later; out of scope here).
- **`results`**: one per finding, **in the report's existing order** (do not
  reorder — the report layer already sorts sections by path and findings within a
  section deterministically). `ruleId` = `finding.rule`; `message.text` =
  `finding.message`; `locations[0].physicalLocation.artifactLocation.uri` =
  `finding.path` display string. If `finding.line` is `Some(n)`, include
  `physicalLocation.region.startLine = n`; omit `region` when `line` is `None`.
- **Empty/clean report:** `results: []` and `rules: []` (still a valid SARIF log).

## Acceptance Criteria

- [ ] `skillport lint <PATH> --sarif` prints a valid SARIF 2.1.0 log (parses as
      JSON; `version == "2.1.0"`; `$schema` present; `runs[0].tool.driver.name ==
      "skillport"` with the crate `version`).
- [ ] Each finding becomes one `results[]` entry: `ruleId == finding.rule`,
      `level` mapped per DEC-003 (`info → "note"`), `message.text == finding.message`,
      and `artifactLocation.uri == finding.path`. `line` → `region.startLine` when present.
- [ ] `runs[0].tool.driver.rules` lists the **distinct** rule ids present, sorted;
      no duplicates.
- [ ] Results are in the report's order (sections path-sorted, findings within
      deterministic); the SARIF emitter does **not** reorder results.
- [ ] `--sarif` and `--json` are **mutually exclusive** — passing both is a clap
      usage error (exit 2, message on stderr, empty stdout).
- [ ] Exit codes unchanged: `--sarif` uses the same `Report::exit_code(strict)`
      (0 clean / 1 error-or-warning-under-strict); `--sarif` output itself is
      identical regardless of `--strict` (only the exit code differs).
- [ ] A clean report → SARIF with `results: []`, `rules: []`, exit 0.
- [ ] Deterministic: same input → byte-identical SARIF stdout.
- [ ] No new dependency; no new rule/analysis; human/`--json` unchanged.
- [ ] `docs/api-contract.md`'s `--sarif` section updated to the shipped shape.

## Failing Tests

Written now (design).

- **`src/emit.rs` (mod tests)** — build a `Report` in-memory; call `sarif(&report)`;
  parse it back with `serde_json::Value` and assert:
  - `"sarif envelope: version 2.1.0, $schema, driver name/version"`.
  - `"finding → result with ruleId, mapped level, message.text, artifact uri"` —
    include one Error, one Warning, one Info; assert levels `error`/`warning`/`note`.
  - `"line → region.startLine when present; absent otherwise"`.
  - `"driver.rules is the distinct rule ids, sorted, deduped"`.
  - `"results preserve report order"`.
  - `"clean report → results [] and rules [], still valid"`.
- **`tests/cli.rs`** (integration, `env!("CARGO_BIN_EXE_skillport")`):
  - `"lint --sarif on bad fixture → valid JSON (parse), exit 1, contains name.charset"`.
  - `"lint --sarif on good fixture → exit 0, results empty"`.
  - `"--sarif and --json together → exit 2 (clap conflict), empty stdout"`.

## Implementation Context

### Decisions that apply

- `DEC-003` — the severity→level mapping (`Info → "note"`); exit codes come from the
  same `Report::exit_code`, unchanged.
- `DEC-005` — SARIF is a machine format that consumers parse; keep it deterministic
  (sorted rules, report-ordered results, byte-identical for identical input). SARIF's
  own version is `2.1.0` (the standard); our `--json schema:1` is separate.

### Constraints that apply

- `deterministic-stable-output`, `test-before-implementation`.

### Prior related work

- `SPEC-005` — `emit::json` (the DTO pattern to mirror) + the `Lint` clap command +
  the stdout/stderr + exit-code dispatch. `serde_json` (DEC-008) is already a dep.
- `SPEC-003` — `Finding { rule, severity, message, path, field, line }`,
  `Severity::label()`. Note SARIF uses `note` for info, so you can't reuse `label()`
  directly for the level — add a small `sarif_level(severity)` mapper.

### Out of scope (for this spec specifically)

- `--target claude` / per-platform verification — next STAGE-003 spec.
- Real-tokenizer `body.size` — a later STAGE-003 spec.
- The GitHub Action / CI workflow that *uploads* the SARIF — a later STAGE-003 spec
  (this spec just produces valid SARIF).
- Rich SARIF (`helpUri`, rule descriptions, `partialFingerprints`, `ruleIndex`) —
  keep it minimal-valid; enrich later if needed.

## Notes for the Implementer

- **Mirror `emit::json`:** local `#[derive(Serialize)]` DTOs (`SarifLog`, `Run`,
  `Tool`, `Driver`, `ReportingDescriptor`, `SarifResult`, `Message`, `Location`,
  `PhysicalLocation`, `ArtifactLocation`, `Region`), built by borrowing `&Report`,
  serialized with `serde_json::to_string`. Use `#[serde(rename = "$schema")]` for
  the schema field and `#[serde(skip_serializing_if = "Option::is_none")]` for
  optional `region`.
- **clap mutual exclusion:** `#[arg(long, conflicts_with = "json")]` on `sarif`
  (clap emits the usage error + exit 2 for you).
- **Determinism:** collect distinct rule ids into a `BTreeSet<&str>` (sorted) for
  `rules`; iterate `report.sections`/`findings` in order for `results` (don't sort
  results — they inherit the report's order).
- **`sarif_level`:** `Error→"error"`, `Warning→"warning"`, `Info→"note"` — a small
  local fn (do NOT reuse `Severity::label`, which returns `"info"`).
- Keep human/`--json` output byte-for-byte unchanged (a regression check).

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:**
- **PR (if applicable):**
- **All acceptance criteria met?** yes/no
- **New decisions emitted:**
  - `DEC-NNN` — <title> (if any)
- **Deviations from spec:**
  - [list]
- **Follow-up work identified:**
  - [any new specs for the stage's backlog]

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — <answer>

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — <answer>

3. **If you did this task again, what would you do differently?**
   — <answer>

---

## Reflection (Ship)

*Appended during the **ship** cycle. Outcome-focused reflection, distinct
from the process-focused build reflection above.*

1. **What would I do differently next time?**
   — <answer>

2. **Does any template, constraint, or decision need updating?**
   — <answer — if yes but not done this session, record it in
   `/guidance/signals.yaml`: `type: lesson` (with its N-count) for a recurring
   coding pattern, `type: process-debt` for tooling/process friction. A close
   then forces the decision. See `docs/signals.md`.>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>

4. **Where was the worst defect caught?** — one word from a fixed vocabulary so
   the defect-escape distribution is greppable across specs:
   `design` | `build` | `verify` | `ship` | `escaped` (reached prod/runtime) |
   `none` (clean first try).
   — <one word>
   *(Runtime/operational defects — the escape-prone class — only exist once the
   artifact meets its real host. `escaped` here is a signal to strengthen the
   §12 behavioral pre-flight for that surface.)*
