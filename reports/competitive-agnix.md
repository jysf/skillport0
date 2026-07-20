# Competitive benchmark: skillport vs. agnix

**Date:** 2026-07-19
**Method:** Ran `agnix@0.40.0` (published 2026-07-16) against all 14 skillport lint
fixtures, each copied into an isolated `/tmp/agnix-bench/<name>/<name>/SKILL.md`
tree so no parent dir named `good/`/`bad/`/`warn-only/` could confound
directory-name inference. Read agnix's own source (`github.com/agent-sh/agnix`,
`crates/agnix-core/src/rules/skill/`) as ground truth, not the README. Rule
baseline is `skillport::RULES` in [src/rules.rs](src/rules.rs) (26 ids).

---

## One-line verdict

**BEHIND.** As a general-purpose SKILL.md linter, skillport's lint layer is a
near-subset of agnix. agnix is also Rust, also deterministic (verified
byte-identical), also offline, also emits `--format json`/`sarif` + CI exit
codes — *and* adds autofix, 5-tool coverage, and ~36 SKILL.md rules (vs our 24
engine rules) including deep Claude-specific checks we don't have. Our
CI/determinism/SARIF story is **not** a differentiator; agnix ships all of it.
Our only genuine lint niches are minor (token-based `body.size`, `body.empty`,
the `<40 char` description nudge, and granular type diagnostics). **The skills
supply-chain lane, however, is still open** — agnix does no content hashing,
provenance, or drift/baseline detection of the skills it lints.

---

## Setup facts (observed)

- `bin: agnix`, `agnix 0.40.0`, zero runtime deps, public repo (also Rust).
- **Offline / no telemetry:** `telemetry status` → `Configured: disabled`,
  `Effective: disabled`; "Opt-in only (disabled by default)… No file paths or
  contents ever collected." Ran every fixture with no credentials, no network
  needed. **Fully offline confirmed.**
- **Deterministic:** two JSON runs of the same fixture were byte-identical. No
  LLM in the path (pure Rust rule engine; `eval` subcommand scores rules against
  labeled cases, still deterministic).
- CLI: `validate` (default), `--fix`/`--fix-safe`/`--fix-unsafe`, `--format
  text|json|sarif|github`, `-t/--target generic|claude-code|cursor|codex|kiro`
  (note: `-t` is now deprecated in favor of a `tools` array), `explain <id>`,
  `eval`, `schema`, `--watch`, `--locale` (12 locales).
- Rule IDs are stable strings: `AS-*` (Agent Skills spec), `CC-SK-*` (Claude Code
  skill), `XP-SK-*` (cross-platform), plus `VER-001` (version-pin info).

---

## How much of agnix is actually SKILL.md? (spec grounding + depth)

The "437 rules across 5 tools / 5 artifact types" headline is **broad, not deep**
in any one place — 449 distinct rule ids in source, spread across MCP (26), Copilot
(25), Cursor (19), Kiro (14), Codex, OpenCode, hooks, CLAUDE.md, etc. The
**SKILL.md-specific** surface is:

- **14 active `AS-*`** rules (the agentskills.io open-spec baseline — the same spec
  skillport targets), and
- **21 `CC-SK-*`** rules (Claude Code skill semantics), + **1 `XP-SK-*`**.

So ~**36 SKILL.md rules**. That is *deeper than skillport* on Claude semantics
and equal-or-deeper on the open spec. The `AS-*` rules are genuinely
spec-grounded (source cites the agentskills.io 1024-char baseline, Codex's
`quick_validate.py`, etc.), not vibes. skillport is not "more rigorous about the
spec" than agnix; agnix is rigorous *and* broader.

---

## Rule-by-rule: skillport's 26 ids vs agnix

Legend: **YES** = agnix has an equivalent check; **PARTIAL** = caught, but coarser
(usually folded into a whole-file `AS-016` parse error); **NO** = agnix does not flag.

| skillport rule | agnix | Evidence |
|---|---|---|
| frontmatter.missing | **YES** `AS-001` | frontmatter-missing → `AS-001 error: SKILL.md must have YAML frontmatter between --- markers` |
| frontmatter.unclosed | **YES** `AS-001` | frontmatter-unclosed → same `AS-001` (agnix folds unclosed into "no frontmatter") |
| frontmatter.invalid | **YES** `AS-016` | frontmatter-invalid → `AS-016 error: invalid type: sequence, expected struct SkillFrontmatter` |
| frontmatter.unknown | **YES** `CC-SK-017` + `XP-SK-001` | My-Skill → `warning: Unknown frontmatter field 'random_field'` + `info: client-specific field … not part of the universal Agent Skills spec` |
| name.required | **YES** `AS-002` | name-required → `AS-002 error: missing required 'name' field` |
| name.type | **PARTIAL** `AS-016` | field-type-errors (`name: 42`) → whole-file `AS-016` parse error, not a dedicated name.type finding |
| name.length | **YES** `AS-004` | name-too-long → `AS-004 error: Name '…' must be 1-64 characters per segment` |
| name.charset | **YES** `AS-004` | My-Skill (`-My--Skill!`) → `AS-004` (charset folded into the same "lowercase letters, digits, and hyphens" rule) |
| name.hyphen-edges | **YES** `AS-005` | My-Skill → `AS-005 error: Name … cannot start or end with hyphen` |
| name.hyphen-consecutive | **YES** `AS-006` | My-Skill → `AS-006 error: Name … cannot contain consecutive hyphens` |
| name.dir-match | **YES** `AS-017` | mismatched-name → `AS-017`. **But agnix emits it as ERROR; skillport as WARNING** (a real severity difference) |
| description.required | **YES** `AS-003` | My-Skill → `AS-003 error: missing required 'description' field` |
| description.type | **PARTIAL** `AS-016` | non-string description → coarse parse error, not description.type |
| description.length | **YES** `AS-008` | description-issues (`""`) → `AS-008 error: Description must be 1-1536 characters, got 0`. (agnix cap: 1024 generic / **1536** claude-code; skillport: 1024 flat) |
| description.detail (<40, info) | **NO** | description-short (`short desc`) → agnix reports **0/0**. `AS-008` only fires outside `1..=max`; there is no short-but-legal nudge. **skillport-unique.** |
| compatibility.length | **YES** `AS-011` | description-issues → `AS-011 error: Compatibility must be 1-500 characters, got 600` |
| compatibility.type | **PARTIAL** `AS-016` | non-string → coarse parse error |
| metadata.type | **PARTIAL** `AS-016` | field-type-errors (`metadata: flat-string`) → `AS-016 error: metadata: invalid type: string …, expected a map`. Coarse, not a warning-level metadata.type. |
| metadata.values | **NO** | agnix does not check that each metadata value is a string |
| allowed-tools.format (list vs string) | **NO** | agnix treats a YAML list as canonical (it's valid in Claude); it does not carry the open-spec "should be a space-separated string" nudge |
| allowed-tools.type | **PARTIAL** | wrong type → parse error / `CC-SK-008`, not a dedicated allowed-tools.type |
| body.empty | **NO** | body-empty → agnix reports **0/0**. **skillport-unique.** |
| body.lines (>500) | **YES** `AS-012` | body-oversized → `AS-012 warning: Skill content exceeds 500 lines (got 602)` |
| body.size (>~5000 tokens, real cl100k_base) | **NO** | agnix has `AS-012` (lines) and `AS-015` (8MB upload) but **no token-count rule**. **skillport-unique.** |
| file.unreadable (structural) | **Untested** | internal to agnix's walker; not fixture-observable |
| dir.unreadable (structural) | **Untested** | same |

**Coverage tally:** of skillport's 24 engine rules, agnix fully matches ~15,
partially covers ~5 (coarse parse-error instead of granular typed finding), and
**misses 4**: `description.detail`, `metadata.values`, `allowed-tools.format`,
`body.empty`, `body.size`.

### What agnix catches that skillport MISSES (the bigger list)

This is where agnix pulls ahead — mostly Claude-semantic checks skillport has no
equivalent for. Observed live on our own fixtures where noted:

- `AS-009` Description contains angle brackets (Codex breakage)
- `AS-013` File reference too deep
- `AS-015` Upload size exceeds 8MB
- `CC-SK-003` **Context without agent** — *fired on our claude-extension fixture*
  (`error: Context 'fork' requires an 'agent' field`), which skillport passes clean
- `CC-SK-007` **Unrestricted Bash** — *fired on claude-extension* (`warning:
  Unrestricted Bash access detected… use scoped version`)
- `CC-SK-013` **Fork context without actionable instructions** — *fired on
  claude-extension*
- `CC-SK-001/002/005` invalid model / context / agent-type values
- `CC-SK-006` **Dangerous auto-invocation**
- `CC-SK-008` Unknown tool name in allowed-tools
- `CC-SK-009` Too many injections; `CC-SK-011` Unreachable skill
- `CC-SK-010` Invalid hooks in frontmatter
- `CC-SK-012/016` argument-hint ↔ `$ARGUMENTS` consistency
- `CC-SK-014/015/018/019/020` type checks for disable-model-invocation,
  user-invocable, effort, paths, shell
- `CC-SK-021` **Hardcoded user-home paths in bundled `scripts/`** — agnix *reads
  into the skill's `scripts/` directory and scans script bodies*
- Cross-cutting: **autofix** (2 of our fixtures reported fixable), SARIF + GitHub
  annotations, GitHub Action, 12 locales.

---

## Where agnix is genuinely better (no flattery)

1. **Breadth of the SKILL.md checkset.** ~36 skill rules vs our 24, and the extra
   ~20 are not padding — they are real Claude-semantic footguns (fork-without-agent,
   dangerous auto-invocation, unrestricted Bash, unreachable skill, hooks-in-
   frontmatter) that skillport is blind to.
2. **Autofix.** `--fix/--fix-safe/--fix-unsafe` with certainty tiers. skillport
   has none.
3. **Bundled-content awareness.** `CC-SK-021` scans `scripts/` files — agnix
   already looks *past* SKILL.md into the skill's payload, the exact territory we
   were hoping was empty.
4. **Same "deterministic Rust / offline / SARIF / exit codes" story we lead with.**
   This erases our positioning. Verified: byte-identical output, offline, SARIF,
   exit 1 on error.
5. **Multi-tool + i18n + maintenance velocity.** 5 agents, 12 locales, a 200KB
   changelog with weekly tool-baseline bumps. We cannot out-run this on breadth.

---

## Where skillport is genuinely better (honestly, not much)

1. **Token-budget `body.size`.** A real `cl100k_base` tokenizer flags bodies
   >~5000 tokens. agnix only counts *lines* (`AS-012`) and *bytes* (`AS-015`).
   For a context-window-cost lens this is a legitimate, defensible edge — but a
   narrow one.
2. **`body.empty`** and the **`<40-char` `description.detail`** nudge — soft
   quality signals agnix omits. Minor.
3. **Granular typed diagnostics.** Where agnix hard-fails a whole file with one
   `AS-016` parse error (`name: 42`, non-map `metadata`), skillport emits precise
   `name.type` / `metadata.type` / `metadata.values` findings. Arguably better
   DX, but agnix's coarse behavior is defensible and it still *catches* the same
   files.
4. **`name.dir-match` as a warning, not an error.** Debatable which is correct;
   agnix's error-level is stricter.

None of these justify skillport existing as a standalone linter next to agnix.

---

## *** Is the skills supply-chain lane still open? *** — YES

Searched agnix's rule logic and schemas for `sha256`/`blake3`/`checksum`/
`content-hash`/`provenance`/`lockfile`/`baseline`/`drift`/`signature`/`sigstore`.
Findings:

- **On the skills it lints: NONE of it.** No content hashing, no provenance/source
  tracking, no lockfile, no pin, no drift/baseline of a skill artifact. The
  `drift` hits in source are about internal config allow-list logic; the
  `baseline` hits mean the *agentskills.io spec baseline* (length caps), not a
  security baseline of skill contents.
- **The only overlap with "what a skill can DO"** is quality-heuristic, not
  provenance: `CC-SK-007` (unrestricted Bash), `CC-SK-006` (dangerous
  auto-invocation), `CC-SK-008` (unknown tool), and the `CC-SK-021` `scripts/`
  path scan. These flag *risky config*, not *identity/integrity/drift*.
- **agnix's supply-chain features protect agnix itself, not your skills:** the
  changelog shows "Release provenance attestations" (OIDC), "npm installer
  checksum verification (SHA-256)", and "Supply-chain hygiene (cargo audit)" —
  all for shipping the agnix binary, none exposed as a feature for auditing
  third-party skills. **Do not mistake these for skill provenance.**

**Caveat for honesty — the concept is already being built next door.** The
secondary target **agentlint** (`github.com/akz4ol/agentlint`, security scanner)
ships exactly the capability/provenance model we're eyeing — its 20 rules include
`OBS-002 No Permission Manifest`, `OBS-001 Missing Capability Declaration`,
`SCOPE-001 Capability Expansion Between Versions`, `SCOPE-002 Write Scope
Widening`, `NET-002 Remote Script Fetch`, plus a `diff <base> <target>` command
for behavioral-change/drift detection. **But it does not parse SKILL.md** —
`agentlint scan` on our fixtures returned `Parsed: 0 documents`. So the
*permissions-manifest + version-drift* idea is proven and unoccupied **for
SKILL.md specifically**; skillport would be first to apply it to skills, but
should expect agentlint (or agnix's `scripts/` scanning) to extend into it.

---

## Secondary targets (Step 7)

- **agentlinter@0.3.3** — Claude-centric, no public repo. **Does not handle
  SKILL.md**: on My-Skill and the whole bench tree it reported "No agent
  configuration files found (CLAUDE.md, AGENTS.md, etc)." It only reads
  CLAUDE.md/AGENTS.md-style files. **Privacy flag:** its default mode *uploads a
  report* ("Lint & share report (default)"); `--local` is opt-in. Not offline by
  default, not a SKILL.md tool, does not address the supply-chain question.
- **agentlint** — security/capability scanner, offline, `--format text|json|sarif`,
  `--fail-on` thresholds, and a `diff` command. **Does not parse SKILL.md** either
  (`Parsed: 0 documents` on our fixtures) — it targets hooks/MCP/agent configs.
  But its rule taxonomy (EXEC/FS/NET/SEC/HOOK/INST/SCOPE/OBS + `diff`) is the
  clearest existing blueprint for the capability/provenance lane, just aimed at
  the wrong artifact. Most strategically relevant of the three competitors *for
  where PROJ-002 wants to go*, precisely because it validates the thesis without
  occupying the skills slice of it.

---

## Recommended action

**Primary: (i) pivot the strategic weight to skills supply-chain / provenance,
blended with (iv) interoperate for lint.**

Rationale: options (ii) "keep lint as a deterministic Rust/CI niche" is weak —
agnix already *is* the deterministic Rust/CI SKILL.md linter, with autofix and
5-tool breadth we can't match, so a lint-first identity is a losing race. Option
(iii) "stop and rethink" is unwarranted: agnix covers the *lint* thesis but
explicitly **not** the supply-chain thesis for skills, which is the actual
PROJ-002 bet and remains open.

Concretely:
1. **Make provenance the product**, not lint. Build the pieces neither agnix nor
   agentlint offers *for skills*: a **content hash / lockfile** per installed
   skill (SKILL.md + `scripts/` + assets), **source/provenance tracking** (where
   did this skill come from, is it pinned), and **drift/baseline detection**
   (did an installed skill's contents or its declared capabilities change since
   you vetted it — the `agentlint diff` idea, applied to skills). This is
   defensible, deterministic, and genuinely unoccupied.
2. **Add a permissions manifest** for skills: what a skill can *do* (allowed-tools
   surface, bundled `scripts/` reach, network) as a stable, diffable artifact —
   agentlint's `OBS-002`/`SCOPE-*` model, ported to SKILL.md.
3. **Do not re-implement 400 lint rules.** For the linting people still want,
   **interoperate**: either shell out to / consume agnix's JSON/SARIF, or keep
   skillport's thin lint layer only as the parse step that *feeds* hashing
   (you're already parsing frontmatter + body; reuse it for the manifest). Keep
   only the token-budget `body.size` as a small unique signal.
4. **Move fast** — agnix already scans `scripts/` (`CC-SK-021`) and agentlint
   already ships version-diff; the open window is "provenance/integrity for
   skills," and both neighbors are one feature away from it.
