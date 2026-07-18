---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-001
  type: story                      # epic | story | task | bug | chore
  cycle: verify  # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: M                    # S | M | L  (L means split it)

project:
  id: PROJ-001
  stage: STAGE-001
repo:
  id: skillport

agents:
  architect: claude-opus-4-8
  implementer: claude-opus-4-8    # tier_map.build; updated by the build agent to the real model
  created_at: 2026-07-17

references:
  decisions:
    - DEC-004   # collection-first, order-preserving/lossless model
    - DEC-005   # deterministic output; malformed input never aborts
    - DEC-002   # frontmatter must stay typed so per-platform/verified rules can inspect it
  constraints:
    - deterministic-stable-output
    - collection-first-substrate
    - no-new-top-level-deps-without-decision
    - license-policy
  related_specs: []

value_link: "infrastructure enabling STAGE-001's canonical, lossless Skill model that every rule and the audit read"

cost:
  sessions:
    - cycle: design
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-07-17
      notes: "main-loop, not separately metered (design cycle)"
    - cycle: build
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-07-17
      notes: "parser substrate build; orchestrator fills real tokens_total/duration/usd at ship"
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-001: tolerant lossless SKILL.md parser

## Context

This is the first spec of STAGE-001 (Core substrate) and the base of the whole
tool: nothing can be linted or audited until a `SKILL.md` on disk becomes an
in-memory model. The parser must be **tolerant** (real-world files have BOMs,
blank lines, CRLF, and malformed or missing frontmatter — none may crash it or
abort a bulk run, per DEC-005) and **lossless / order-preserving** (frontmatter
key order preserved; original bytes recoverable — per DEC-004, so later
normalization and the audit can trust the model). It deliberately does **not**
judge the skill — it reports structural facts; the rule engine (STAGE-002)
assigns severities and rule ids.

- Parent stage: `STAGE-001-core-substrate` (spec 1 of 4).
- Project: `PROJ-001` (foundation + lean `lint`).
- Design docs: [`docs/architecture.md`](../../../docs/architecture.md) (parse
  stage), [`docs/data-model.md`](../../../docs/data-model.md) (the `Skill` type).

## Goal

Implement a pure function that turns the raw contents of a `SKILL.md` (plus its
path) into a canonical `Skill` — splitting typed, order-preserving YAML
frontmatter from the Markdown body — and that **never fails**: every tolerated or
malformed case is captured as a `FrontmatterStatus` on the returned `Skill`, not
an error that aborts.

## Inputs

- **Files to read (reference only):** `initial_stuff/parse.rs`, `initial_stuff/skill.rs`
  — the prototype's parser/model; a reasonable reference for the frontmatter
  split, but it is **not** collection-first, uses the deprecated `serde_yaml`, and
  does not model the tolerant `FrontmatterStatus` cases below. Port the idea, not
  the code verbatim.
- **Fixtures:** `initial_stuff/skillport/lint-fixtures/good|bad/**/SKILL.md` — real
  examples to fold into tests.
- **Related code paths:** none yet — this creates `src/parse.rs` and `src/skill.rs`.

## Outputs

- **Files created:**
  - `src/skill.rs` — the `Skill` model + `Frontmatter` + `FrontmatterStatus`.
  - `src/parse.rs` — `parse(...)` + its unit tests.
  - `Cargo.toml` — first real manifest (crate metadata + the YAML/ordered-map deps).
  - `src/lib.rs` (or `main.rs` stub) — enough to compile and expose the module for tests.
- **New exports (indicative signatures — final names are the build's call, but the
  shape is fixed):**

  ```rust
  pub struct Skill {
      pub path: PathBuf,
      pub dir_name: Option<String>,     // parent directory name, for name.dir-match later
      pub frontmatter: Frontmatter,     // order-preserving; empty unless status == Present
      pub body: String,                 // markdown after the frontmatter block (verbatim)
      pub raw: String,                  // original content, byte-for-byte (losslessness)
      pub frontmatter_status: FrontmatterStatus,
  }

  /// Order-preserving map of frontmatter key -> typed YAML value.
  pub type Frontmatter = /* an insertion-order-preserving map<String, YamlValue> */;

  pub enum FrontmatterStatus {
      Present,          // opening + closing fence, valid YAML mapping
      Missing,          // no opening fence — whole file is body
      Unclosed,         // opening fence, no closing fence
      Invalid(String),  // fenced block present but YAML invalid or not a mapping (msg = why)
  }

  /// Never returns Err — tolerance is expressed via FrontmatterStatus.
  pub fn parse(path: PathBuf, raw: &str) -> Skill;
  ```
- **Database changes:** none.

## Acceptance Criteria

- [ ] `parse` is **total** — it never panics and never returns an error type; every
      case below yields a `Skill` with an appropriate `frontmatter_status`.
- [ ] **Well-formed:** `---`\<yaml mapping\>`---`\<body\> → `Present`; frontmatter
      exposes the keys as **typed** values (string vs. sequence vs. mapping are
      distinguishable, so STAGE-002's `name.type` / `metadata.type` /
      `allowed-tools.format` rules can inspect them).
- [ ] **Key order preserved:** iterating `frontmatter` yields keys in source order,
      not sorted (DEC-004).
- [ ] **Lossless:** `skill.raw` equals the input byte-for-byte for every case
      (including BOM and CRLF).
- [ ] **BOM tolerated:** a leading UTF-8 BOM does not prevent frontmatter detection.
- [ ] **Leading blank lines tolerated:** blank lines before the opening `---` still
      detect frontmatter.
- [ ] **CRLF tolerated:** `\r\n` line endings parse the same as `\n`.
- [ ] **Missing frontmatter:** no opening fence → `Missing`, `frontmatter` empty,
      `body` == full content (after BOM). (STAGE-002 turns this into
      `frontmatter.missing`.)
- [ ] **Unclosed frontmatter:** opening fence, no closing fence → `Unclosed`,
      `frontmatter` empty. No panic, no infinite scan.
- [ ] **Invalid YAML / non-mapping root:** fenced block that fails to parse, or
      whose root is not a mapping (e.g. a list/scalar) → `Invalid(msg)`,
      `frontmatter` empty, `body` still correctly separated.
- [ ] **Empty file:** `""` → `Missing`, empty `body`, empty `frontmatter`.
- [ ] Output is deterministic: same input → identical `Skill` (no map reordering
      across runs) (DEC-005).
- [ ] The YAML and ordered-map dependencies are added to `Cargo.toml` **with an
      accompanying `DEC-007`** justifying the crate choice (see below).

## Failing Tests

Written now (design), before build. Build makes them pass. Location: a
`#[cfg(test)] mod tests` in `src/parse.rs` (plus a couple fixture-backed cases).
Use table-style cases where natural.

- **`src/parse.rs` (mod tests)**
  - `"wellformed: splits frontmatter and body"` — asserts status `Present`;
    `frontmatter["name"]` is the string `"foo"`; `body` starts with `# Body`.
  - `"frontmatter key order is preserved"` — input keys `name, description, license`;
    asserts the iterated key order is exactly `["name","description","license"]`.
  - `"typed values are distinguishable"` — `allowed-tools` given as a YAML **list**
    is observably a sequence (not a string); `metadata` given as a map is a mapping.
  - `"lossless: raw equals input"` — for a BOM+CRLF sample, `skill.raw == input`.
  - `"strips/ignores a UTF-8 BOM for detection"` — BOM + `---…---` → `Present`.
  - `"leading blank lines before frontmatter"` — `"\n\n---\nname: x\n---\nbody"` → `Present`.
  - `"CRLF endings parse like LF"` — same content with `\r\n` → `Present`, same keys.
  - `"missing frontmatter → Missing, full body"` — `"# Just markdown\n"` → `Missing`,
    empty frontmatter, `body == "# Just markdown\n"`.
  - `"unclosed frontmatter → Unclosed"` — `"---\nname: x\n\n# body, no close"` →
    `Unclosed`, empty frontmatter, no panic.
  - `"invalid YAML → Invalid, body still separated"` — `"---\nname: [oops\n---\n# b\n"`
    → `Invalid(_)`, empty frontmatter, `body == "# b\n"`.
  - `"non-mapping root → Invalid"` — `"---\n- a\n- b\n---\nbody"` → `Invalid(_)`.
  - `"empty file → Missing, empty body"` — `""` → `Missing`, `body == ""`.
- **fixture-backed**
  - `"good fixtures parse Present with expected keys"` — every
    `lint-fixtures/good/**/SKILL.md` parses to `Present` with `name` + `description`.

## Implementation Context

*Read this section (and the files it points to) before starting the build cycle.*

### Decisions that apply

- `DEC-004` — collection-first, **order-preserving & lossless**. The frontmatter
  map MUST preserve insertion order (an index-map-style structure, not `HashMap`),
  and `raw` must let you reproduce the file. This is the reuse base for the audit.
- `DEC-005` — **deterministic; never abort.** `parse` is total (no `Result`); a
  malformed skill becomes a `FrontmatterStatus`, which the walker/report later
  render as a per-file finding without stopping a bulk run.
- `DEC-002` — keep frontmatter **typed** (string / sequence / mapping
  distinguishable) so verified per-platform and open-spec rules can inspect value
  shapes; do not stringify everything.

### Constraints that apply

- `deterministic-stable-output` — no nondeterministic ordering in the model.
- `collection-first-substrate` — this spec builds the per-skill model that the
  walker (SPEC for the tree-walker) collects into N; keep `parse` a pure function
  of `(path, raw)` so the walker just maps it over discovered files.
- `no-new-top-level-deps-without-decision` — adding the YAML crate and the
  ordered-map crate are **runtime** deps → author `DEC-007` in the same build pass
  (allowed: dep + DEC in one pass). Explain the crate choice and why not `serde_yaml`.
- `license-policy` — new deps must be permissive (MIT/Apache-2.0/BSD/ISC/Zlib);
  wire `cargo-deny` (can be a later spec, but keep the deps in-policy now).

### Dependency guidance (for DEC-007)

- **Do NOT use `serde_yaml`** — deprecated/unmaintained. Evaluate current
  maintained options (e.g. a maintained `serde_yaml` fork, `saphyr`/`yaml-rust2`,
  or `serde_yaml_ng`/`serde_yml`) — pick one that (a) is maintained, (b) exposes a
  **typed `Value`** with string/sequence/mapping discrimination, and (c) preserves
  **mapping insertion order** (or pair it with an ordered map like `indexmap`).
- Pin with normal caret ranges on the **current stable** toolchain — do NOT reuse
  the prototype's `=`-exact pins (Rust-1.75 artifact).

### Prior related work

- None shipped yet. Reference-only: `initial_stuff/parse.rs`, `initial_stuff/skill.rs`.

### Out of scope (for this spec specifically)

- The tree-walker / collection (separate STAGE-001 spec) — `parse` is per-file.
- Any **rules** or severities or rule ids (STAGE-002). This spec exposes structural
  status only; it must not import or assume the rule engine.
- Emitters (human/JSON/SARIF), the CLI, `--target`, the tokenizer.
- Mapping `FrontmatterStatus` → finding rule ids (STAGE-002's job).
- `...` as an alternate YAML end-marker (only `---` fences recognized here).

## Notes for the Implementer

- Fence detection: a delimiter line is a line equal to `---` after trimming a
  trailing `\r` and trailing spaces; it must be at column 0. Match on lines, not a
  regex over the whole string, so CRLF and blank lines are cheap to handle.
- Compute the split on the BOM-stripped, but keep `raw` as the untouched input.
- Keep `body` **verbatim** (don't normalize line endings) for losslessness; a
  single newline immediately after the closing fence may be consumed so `body`
  doesn't start with a stray blank line — assert whichever you choose in a test.
- Prefer table-driven tests; fold in the `lint-fixtures/good` samples.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-001-parser`
- **PR (if applicable):** opened against `main`, referencing PROJ-001 / STAGE-001 / SPEC-001.
- **All acceptance criteria met?** yes — every AC and all 12 unit cases + the
  fixture-backed case pass; `cargo test` (14 tests) green, `cargo clippy -- -D
  warnings` clean, `cargo fmt --check` clean.
- **New decisions emitted:**
  - `DEC-007` — YAML crate (`serde_yaml_ng`) + ordered-map (`indexmap`) choice,
    with the license check and why not `serde_yaml`.
- **Deviations from spec:**
  - Frontmatter modeled as `indexmap::IndexMap<String, serde_yaml_ng::Value>`
    (the spec left the concrete type to the build; this satisfies the fixed
    shape). `Skill` derives `Debug, Clone` but not `PartialEq` (tests compare
    fields individually; `serde_yaml_ng::Value` does impl `PartialEq` if a
    whole-`Skill` compare is wanted later).
  - **Body-after-close:** the single newline immediately after the closing fence
    is consumed, so `body` never starts with a stray blank line (per the spec's
    "assert whichever you choose" note — asserted in
    `wellformed_splits_frontmatter_and_body` and `invalid_yaml_still_separates_body`).
  - **Empty/whitespace-only fenced block** (`---`\n`---`) is treated as `Present`
    with an empty frontmatter map (tolerant), not `Invalid` — an explicit choice
    not covered by a failing test.
  - **`Unclosed`** yields an empty `body` (the spec fixes frontmatter empty +
    no-panic but leaves body unspecified for this case).
  - Good fixture folded in at repo-root `lint-fixtures/good/data-analysis/SKILL.md`
    (copied from the prototype's `lint-fixtures/good`, which lived only inside
    `initial_stuff/skillport.tar.gz`); the fixture-backed test discovers it via a
    small local recursive walk (the real walker is a later spec).
- **Follow-up work identified:**
  - Wire `cargo-deny` + a license policy into CI (`license-policy`; deferred by
    the spec to a later spec — deps are already in-policy, verified in DEC-007).
  - The tree-walker / collection spec (STAGE-001) will consume `parse` per file;
    it can reuse the test's `collect_skill_files` idea.
  - A `key.duplicate` rule (STAGE-002) — the parser currently lets a duplicate
    frontmatter key take last-write-wins rather than flagging it.

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?** — Only the body shape
   of the non-`Present` statuses (`Unclosed` body, empty-block status) was left
   open; the spec anticipated this ("assert whichever you choose"), so it cost a
   decision, not time. The `lint-fixtures/` referenced by path existed only
   inside a tarball, which took a step to materialize.
2. **Was there a constraint or decision that should have been listed but
   wasn't?** — No. DEC-002/004/005 + the two dep constraints covered the design
   space exactly; DEC-007 was the only new decision needed.
3. **If you did this task again, what would you do differently?** — Extract the
   prototype fixtures first (before reading `parse.rs`), so the fixture-backed
   test target is on disk from the start. Otherwise the offset-based line split
   (verbatim body, CRLF-safe) held up cleanly.

---

## Reflection (Ship)

1. **What would I do differently next time?** — <answer>
2. **Does any template, constraint, or decision need updating?** — <answer>
3. **Is there a follow-up spec I should write now before I forget?** — <answer>
4. **Where was the worst defect caught?** — <one word> `design | build | verify | ship | escaped | none`
