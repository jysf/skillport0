---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-010
  type: story                      # epic | story | task | bug | chore
  cycle: ship  # frame | design | build | verify | ship
  blocked: false
  priority: medium
  complexity: M                    # S | M | L  (L means split it)

project:
  id: PROJ-001
  stage: STAGE-003
repo:
  id: skillport

agents:
  architect: claude-opus-4-8      # design cycle (this orchestrator session)
  implementer: claude-sonnet-5    # build runs as a Sonnet subagent (cost); updated with the real model
  created_at: 2026-07-18

references:
  decisions:
    - DEC-002   # the ~5000-token guidance is open-spec-backed
    - DEC-003   # body.size is soft/advisory -> INFO (never error/warning)
    - DEC-005   # stable rule id; deterministic
  constraints:
    - no-heuristic-error
    - only-verified-constraints-are-firm
    - deterministic-stable-output
    - no-new-top-level-deps-without-decision
    - license-policy
  related_specs:
    - SPEC-004  # lint_skill structure
    - SPEC-006  # check_body (body.empty/body.lines) this extends

value_link: "completes the open-spec body checks — an accurate token-count body.size (info) via a real tokenizer, the last catalog rule (answered Frame question: real tokenizer, info-level)"

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
    - cycle: build
      agent: claude-sonnet-5
      interface: claude-code
      tokens_total: 120656
      estimated_usd: 0.80
      duration_minutes: 31
      recorded_at: 2026-07-18
      notes: "metered Sonnet build subagent; tokens_total = subagent_tokens. estimated_usd = tokens x repo rate 6.60 (order-of-magnitude). duration wall-clock."
    - cycle: verify
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: 77254
      estimated_usd: 0.51
      duration_minutes: 9
      recorded_at: 2026-07-18
      notes: "metered Opus verify subagent (recomputed the tokenizer pins independently; APPROVED, 0 punch-list)."
    - cycle: ship
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-07-18
      notes: "main-loop, not separately metered (ship cycle)"
  totals:
    tokens_total: 197910
    estimated_usd: 1.31
    session_count: 4
shipped_at: 2026-07-18
---

# SPEC-010: body size rule via real tokenizer

## Context

The last open-spec rule. The catalog's `body.size` (~<5000 tokens recommended)
was deliberately deferred from SPEC-006 because it needs a **real tokenizer** (the
answered Frame question: "real tokenizer, info-level", not a chars/words
heuristic). `check_body` already does `body.empty` + `body.lines`; this adds
`body.size` (info) computing an accurate token count of the Markdown body.

- Parent stage: `STAGE-003`; extends `rules::check_body` (SPEC-006).
- After this, the open-spec catalog is 100% implemented; only `--target` widening
  remains of STAGE-003's rule work.

## Goal

Add the `body.size` rule (info) to `lint_skill`: count the tokens of the skill
body with a real tokenizer and, when the count exceeds the recommended ceiling
(~5000), emit an advisory `body.size` finding — introducing one tokenizer
dependency (authored as `DEC-010`).

## Inputs

- **Files to read (extend):** `src/rules.rs` (`check_body` — add `body.size`
  next to `body.empty`/`body.lines`; there's already a deferral comment where it
  goes), `src/skill.rs` (`Skill.body`), `src/report.rs` (`Finding`/`Severity`).
- **Reference:** the prototype's `body.size` used `chars/4` — this spec replaces
  that heuristic idea with a real tokenizer.

## Outputs

- **Files modified:**
  - `src/rules.rs` — a `body_token_count(&str) -> usize` helper (using the
    tokenizer, BPE loaded once via a `OnceLock`/lazy static) + a `body.size` info
    finding in `check_body` when the count exceeds `BODY_TOKENS_THRESHOLD` (5000).
  - `Cargo.toml` — the tokenizer crate (author **`DEC-010`** in the same pass).
- **New dep (author `DEC-010`):** a real tokenizer crate — recommended
  **`tiktoken-rs`** with an embedded BPE (`cl100k_base` or `o200k_base`), used as a
  **proxy** (there is no public Anthropic tokenizer; token counts across modern BPE
  encoders are within ~10–20% for prose, and the rule is info/advisory, so a proxy
  is appropriate). Permissive-licensed; verify with cargo-deny (the `licenses` CI job).
- **No CLI/emitter change** — the info finding flows through `emit` unchanged.
- **Database changes:** none.

## Rule (exact id & severity)

| Rule id | Sev | Check |
|---|---|---|
| `body.size` | info | body token count `> ~5000` (recommended ceiling; use progressive disclosure → move detail into `references/`) |

- **Severity = info** (DEC-003): it's a soft recommendation, never error/warning.
- **Threshold:** `const BODY_TOKENS_THRESHOLD: usize = 5000;` (a `>` comparison;
  document it as tunable, mirroring `BODY_LINES_THRESHOLD`).
- **Message:** e.g. `"body is ~{n} tokens; the spec recommends under 5000 — use
  progressive disclosure (move detail into references/)"`. "~" because the count is
  a proxy tokenizer, not Anthropic's exact one.
- **Gating:** runs inside `check_body`, i.e. only when frontmatter is `Present`
  (same as `body.empty`/`body.lines` — consistent with the existing skip discipline).

## Acceptance Criteria

- [ ] `body.size` is added with id `"body.size"` and severity **info**; it fires
      only when the body's real token count exceeds `BODY_TOKENS_THRESHOLD` (5000).
- [ ] A **real tokenizer** computes the count — NOT a chars/words heuristic. A test
      pins `body_token_count("<known string>")` to the tokenizer's actual output
      (a value that a `chars/4` heuristic would NOT produce), proving it's the real
      tokenizer.
- [ ] A short/normal body → no `body.size` finding; a body over the threshold →
      exactly one `body.size` info finding with the count in the message.
- [ ] `body.size` is **info** (never error/warning); no other rule's severity
      changes. DEC-003 upheld (no heuristic at error level; this soft rule is info).
- [ ] The good fixture still yields **zero findings** (its body is small).
- [ ] Deterministic: the tokenizer is deterministic; same body → same count → same
      output. The BPE is loaded once (not rebuilt per skill).
- [ ] `DEC-010` authored for the tokenizer crate (choice + proxy rationale + license);
      the dep is permissive and passes the `licenses` CI job (cargo-deny).
- [ ] No CLI/emitter change; `cargo test`/`clippy`/`fmt` green.

## Failing Tests

Written now (design). Location: `#[cfg(test)] mod tests` in `src/rules.rs`
(extend the existing module).

- **`src/rules.rs` (mod tests)**
  - `"body_token_count uses a real tokenizer (not chars/4)"` — assert
    `body_token_count("<a chosen sample>")` equals the tokenizer's known count for
    the chosen encoding (the build pins the exact number for its crate/encoding),
    and that this differs from `sample.chars().count()/4`.
  - `"short body → no body.size finding"`.
  - `"oversized body → one body.size info finding"` — a body whose token count
    exceeds 5000 (e.g. a long repeated passage); assert exactly one finding with
    `rule == "body.size"`, `severity == Info`, and the token count in the message.
  - `"body.size severity is info"` (guards DEC-003).
  - `"body.size is the exact stable id"`.
  - `"a body just under the threshold → no finding"` (boundary).
- **integration / fixture-backed**
  - `"lint-fixtures/good → still zero findings"` (via `walk` + `from_collection`).

## Implementation Context

### Decisions that apply

- `DEC-002` — the ~5000-token recommendation is open-spec-backed, so a firm rule is
  justified; but it's a *recommendation*, hence info, not error/warning.
- `DEC-003` — `body.size` is soft/advisory → **info**. No heuristic at error level.
- `DEC-005` — `body.size` is a stable rule id; the count must be deterministic.

### Constraints that apply

- `no-heuristic-error` — keep `body.size` at info.
- `only-verified-constraints-are-firm` — the token *estimate* is a proxy (no public
  Anthropic tokenizer); the "~" in the message + the info severity communicate that.
- `deterministic-stable-output` — deterministic tokenizer; load the BPE once.
- `no-new-top-level-deps-without-decision` — the tokenizer is a runtime dep →
  author `DEC-010` in the same pass (sanctioned).
- `license-policy` — the tokenizer + its transitive deps must be permissive
  (cargo-deny `licenses` job will check); note the license in DEC-010.

### Tokenizer choice (for DEC-010)

- Recommended: **`tiktoken-rs`** (`cl100k_base()` or `o200k_base()`), which embeds
  the BPE ranks (offline, no download at runtime). Encode the body with the
  ordinary encoder (no special tokens) and take `.len()`.
- Rationale to record: there is **no public Anthropic tokenizer crate**; a modern
  OpenAI BPE is a reasonable **proxy** for "how many tokens will this skill consume"
  (within ~10–20% for English prose), and the rule is **info/advisory**, so proxy
  precision is fine. If a maintained Anthropic tokenizer appears later, swap it.
- Check the crate + transitive licenses are permissive (MIT/Apache-2.0/…); the
  embedded vocab grows the binary by ~1–2 MB — acceptable for a lint tool; note it.

### Prior related work

- `SPEC-006` — `check_body` (`body.empty`, `body.lines`, `BODY_LINES_THRESHOLD`).
  Add `body.size` here in the same function; mirror the structure.
- `SPEC-004` — the `lint_skill` skip discipline (`check_body` runs only when
  frontmatter is `Present`).

### Out of scope (for this spec specifically)

- `--target claude` / per-platform verification — the next STAGE-003 spec.
- Making the token count *exact* for a specific model (impossible without that
  model's tokenizer) — it's an approximation by design (info-level).
- A `--max-body-tokens` flag / configurable threshold — later DX if wanted.
- Any CLI/emitter change.

## Notes for the Implementer

- **Load the BPE once:** construct the tokenizer in a `static`/`OnceLock` (or
  `once_cell::Lazy`) so it's built once, not per skill. Don't add `once_cell` if
  `std::sync::OnceLock` suffices (it does on current stable).
- **Encode ordinary:** count content tokens (`encode_ordinary`/`encode_with_special_tokens`
  — pick the content-token count; document which). Determinism is guaranteed by the
  fixed BPE.
- **Pin the tokenizer in a test:** choose a short sample whose token count you can
  hardcode for the chosen encoding (run it once locally to get the number). This is
  what proves "real tokenizer, not chars/4" and guards against a silent dep swap.
- **Threshold const** next to `BODY_LINES_THRESHOLD`; `>` comparison; comment it's tunable.
- Keep `body.size` **info**; a false positive is harmless. Confirm the good fixture
  stays 0/0/0 and `cargo run --example lint_demo -- lint-fixtures/bad` still behaves.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-010-tokenizer`
- **PR (if applicable):** none yet (build cycle only; not opened per this
  cycle's instructions)
- **All acceptance criteria met?** yes
  - `body.size` added, id `"body.size"`, severity info, fires only when the
    real token count exceeds `BODY_TOKENS_THRESHOLD` (5000).
  - Real tokenizer confirmed: `body_token_count_uses_a_real_tokenizer_not_chars_4`
    pins `body_token_count("tokenization")` == 2 (cl100k_base's actual
    output) and asserts it differs from `chars/4` (== 3).
  - Short/normal body -> no finding; over-threshold body (700 repeats of a
    47-char sentence, 7001 tokens) -> exactly one `body.size` info finding
    with `~7001` in the message.
  - `body.size` is info; no other rule's severity was touched.
  - `lint-fixtures/good` still yields 0 errors / 0 warnings / 0 infos
    (confirmed via `cargo run --example lint_demo -- lint-fixtures/good`
    and the existing `from_collection_over_lint_fixtures_good_has_zero_errors`
    test).
  - Deterministic: `cl100k_base()` ranks are `include_str!`'d at compile
    time (no network/filesystem I/O at runtime); the BPE is built once in a
    `static BPE: OnceLock<CoreBPE>`, not rebuilt per skill.
  - `DEC-010` authored; `cargo deny check licenses` reports "licenses ok"
    for the full resolved dependency graph including `tiktoken-rs`.
  - No CLI/emitter change; `cargo test` (110 passed), `cargo clippy
    --all-targets -- -D warnings` (no issues), `cargo fmt --check` (clean)
    all green.
- **New decisions emitted:**
  - `DEC-010` — `tiktoken-rs` (`cl100k_base`) as the `body.size` token
    counter
- **Deviations from spec:**
  - None. Used `cl100k_base` (the spec's first-listed option, over
    `o200k_base`) as the more conservative, longest-track-record encoding;
    documented as swappable in DEC-010.
- **Follow-up work identified:**
  - `--target claude` / per-platform verification (already tracked as the
    next STAGE-003 spec, out of scope here).
  - A `--max-body-tokens` flag for a configurable threshold, if wanted
    later (noted as out of scope in this spec too).

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — Nothing major; the spec's "Notes for the Implementer" section already
   flagged the exact traps (build the BPE once, pin a real sample, use
   `encode_ordinary`). The only judgment call was picking `cl100k_base` over
   `o200k_base` and choosing sample strings — both were explicitly left open
   by the spec ("cl100k_base or o200k_base"; "a chosen sample").

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. `no-heuristic-error`, `no-new-top-level-deps-without-decision`, and
   `license-policy` covered everything needed; DEC-002/003/005 gave the
   severity and determinism guardrails directly.

3. **If you did this task again, what would you do differently?**
   — Same approach. Running the encoder once locally via a throwaway
   `examples/pin_test.rs` (removed before commit) to get exact pinned counts
   for the sample strings and the oversized/boundary bodies was fast and
   avoided guessing at token counts.

---

## Reflection (Ship)

*Appended during the **ship** cycle. Outcome-focused reflection, distinct
from the process-focused build reflection above.*

1. **What would I do differently next time?**
   — The tokenizer-pin test (assert a known string's real token count, ≠ chars/4)
   was the key design move — it makes "is this actually a tokenizer" a mechanical
   check and guards a silent dep swap. Verify even recomputed it independently. Good
   pattern for any "we integrated a real X, not a fake" claim.

2. **Does any template, constraint, or decision need updating?**
   — No. DEC-010 records the tokenizer + the proxy rationale (no public Anthropic
   tokenizer); `license-policy` is enforced by the cargo-deny CI job (passes).

3. **Is there a follow-up spec I should write now before I forget?**
   — The open-spec catalog is now 100% complete. Two STAGE-003 specs remain:
   `--target claude` (primary-doc verification — the differentiated core), then the
   README rule-id/severity table + per-rule fixtures + the spec-perfect-skill
   zero-findings test. Then STAGE-004 (release, DEC-009).

4. **Where was the worst defect caught?** — one word from a fixed vocabulary so
   the defect-escape distribution is greppable across specs:
   `design` | `build` | `verify` | `ship` | `escaped` (reached prod/runtime) |
   `none` (clean first try).
   — <one word>
   *(Runtime/operational defects — the escape-prone class — only exist once the
   artifact meets its real host. `escaped` here is a signal to strengthen the
   §12 behavioral pre-flight for that surface.)*
