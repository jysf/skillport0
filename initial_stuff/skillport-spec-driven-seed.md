# skillport — spec-driven seed

Content to drive `jysf/spec-driven-template`. The **repo is the app** (skillport);
**proj-001** and **proj-002** are two waves of work. This file has three parts:

| Part | Goes into |
|------|-----------|
| 1. Repo context & decisions | `.repo-context.yaml`, `AGENTS.md`, `decisions/` (persists across all projects) |
| 2. PROJ-001 kickoff | your first project — the Frame phase consumes this brief |
| 3. PROJ-002 kickoff | the next wave, started after proj-001 ships |

Suggested variant: **`claude-only`** to start (per the template's own advice).

The stage breakdowns below are suggestions sized to the template's "2–5 stages per
project" rule — let the Frame/Design cycle turn each stage into specs; don't treat
them as pre-written specs.

---

# Part 1 — Repo context & decisions (persist these)

## What the app is

**skillport** — a command-line tool for **validating and auditing agent Skills**
(`SKILL.md` files), shipped as a single fast Rust binary. It answers two questions:
"does this skill conform?" (lint, per-file, CI gate) and "how healthy and how risky
is this *collection* of skills?" (audit, per-library, human report).

## The strategic bet (don't re-scope past this)

Spec-compliant skills are already portable across Claude Code, Codex, Cursor, etc.
by design, and distribution/migration is already covered by Vercel's `npx skills`
and Cursor's native `/migrate-to-skills`. So a **converter** is a near-no-op in a
crowded lane. skillport's value is **validation + normalization + library/security
audit with per-platform awareness and bulk/CI ergonomics** — the unoccupied lane.

## Load-bearing decisions (seed `decisions/`)

- **D1 — Not a converter.** Focus is validate / normalize / audit. Skill↔rule
  semantic migration (`.cursor/rules/*.mdc`, `AGENTS.md`, `CLAUDE.md`) is
  explicitly out; if ever revisited it's a separate wave with lossiness reporting.
- **D2 — Only the open spec is authoritative.** Rules come from
  https://agentskills.io/specification. Every *per-platform* constraint
  (Claude/Cursor/Codex/Vercel) must be confirmed from that platform's own primary
  docs before it's encoded. Unverified constraints are advisory (info) with a
  source comment — never errors. Do not assert what you haven't verified.
- **D3 — Severity discipline.** Crisp spec violations are **errors** (they gate
  CI). Heuristic/analytical findings (overlap, coherence, "looks dead") are
  **advisory** and live in the audit report, never as error-level CI gates. No
  heuristic rule is ever an error.
- **D4 — Build proj-001 for proj-002 reuse.** The substrate is collection-first
  from day one: the tree-walker returns a set of skills, the report layer takes N
  skills with sections (not a single pass/fail), and every rule/finding has a
  stable id. This makes the audit an additive layer, not a refactor.
- **D5 — Rust static binary.** Deterministic, stable output (sort by path); stable
  `--json` schema for CI. One malformed skill must never abort a bulk run — report
  it as a per-file finding and continue.
- **D6 — Provenance is hash-anchored, not honor-system.** `metadata.author` /
  `version` are self-asserted and unverifiable; trustworthy provenance is a
  content hash + observed source that the *tool* records and checks for drift.

## Positioning caveat to keep in view

The open-spec checks overlap with the official `skills-ref validate`
(github.com/agentskills/agentskills). So the open-spec layer is table stakes;
skillport's differentiation is the **per-platform layer, bulk/CI ergonomics, and
the audit (library health + security + provenance)**. Weight effort accordingly.

## References

- Open spec (authoritative): https://agentskills.io/specification
- Existing open-spec validator: `skills-ref validate`
- Verify per-platform from primary docs: Claude (docs.claude.com), Cursor
  (cursor.com/docs), Codex (Codex docs / `AGENTS.md`), Vercel (skills.sh).

## Optional starting point

A working prototype exists (parse + the open-spec rule catalog in Part 2 + human/
JSON output + exit codes + bulk mode + good/bad fixtures). You may build on it or
start fresh. If you build on it: the `open`-layer rules are spec-backed and
trustworthy; the `claude/cursor/codex/vercel` profiles are **unverified guesses**
to confirm before shipping. Any dependency version pins in it were an old-toolchain
artifact — use current versions.

---

# Part 2 — PROJ-001 kickoff: foundation + lean lint

⤵ **Paste this to Frame proj-001.**

**Goal of this wave:** a genuinely useful `lint` command, built on a substrate
deliberately shaped for the proj-002 audit to reuse. Ship the foundation and the
crisp validator — not the polish.

**In scope**
- Parse `SKILL.md` (YAML frontmatter + Markdown body) into a canonical,
  order-preserving, lossless model.
- Open-spec rule engine with three severities (error / warning / info).
- `lint <path>`: a single skill, a skill folder, or a whole tree of skills.
- Optional `--target <platform>` layer widening which frontmatter fields are
  "recognized" (verify each platform from primary docs first; otherwise advisory).
- Human-readable and `--json` output; CI exit codes; `--strict` fails on warnings.

**Out of scope for this wave (defer, don't gate on)**
- `--fix` autofix, SARIF output, a GitHub Action — pull forward only if cheap.
- Anything audit/collection-level (that's proj-002).

**Suggested stages**
1. **Core substrate (built for reuse).** Tolerant parser (BOM, leading blank
   lines, missing/unclosed frontmatter handled gracefully); canonical `Skill`
   model; tree-walker returning a *collection* (skip `.git`, `node_modules`,
   `target`); finding + report model that already takes N skills with sections and
   stable ids. This is the shared base for proj-002 — design it there.
2. **Open-spec rule engine + `lint` command.** Implement the rule catalog below;
   single-skill and tree modes; human + `--json`; exit codes + `--strict`;
   per-file parse errors don't abort a bulk run.
3. **Per-platform layer + DX.** `--target` recognized-field sets (each verified
   from primary docs, else advisory); README with rule ids/severities; per-rule
   unit tests + good/bad fixtures; a test that a spec-perfect skill yields zero
   findings; a CI snippet.

**Open-spec rule catalog (implement exactly; source: agentskills.io).** Severity:
error = spec violation, warning = recommended/likely-wrong, info = advisory.

| Rule id | Sev | Check |
|---|---|---|
| `frontmatter.missing` | error | frontmatter block present |
| `name.required` / `name.type` | error | present; is a string |
| `name.length` | error | 1–64 chars |
| `name.charset` | error | lowercase letters, digits, hyphens only |
| `name.hyphen-edges` | error | no leading/trailing hyphen |
| `name.hyphen-consecutive` | error | no `--` |
| `name.dir-match` | warning | equals parent directory name |
| `description.required` / `description.type` | error | present; is a string |
| `description.length` | error | 1–1024 chars, non-empty |
| `description.detail` | info | too terse to convey *when* to use (soft; tune) |
| `compatibility.length` | error | ≤500 chars if present |
| `metadata.type` | warning | is a key→value map |
| `metadata.values` | info | values are strings (spec is string→string) |
| `allowed-tools.format` | warning* | space-separated string, not a list (*info where a platform is confirmed to accept a list) |
| `body.empty` | warning | body non-empty |
| `body.lines` | warning | ≤500 lines recommended |
| `body.size` | warning | ~<5000 tokens recommended (estimate method = open question) |
| `frontmatter.unknown` | info | key recognized (widen per `--target`) |

**Open questions to surface in Frame (ask before building)**
1. Which single platform to verify/support first for `--target`? (That's where the
   real primary-doc work goes.)
2. Token-size estimate: rough chars/words heuristic (advisory) or a real
   tokenizer? Default: heuristic, info-level.
3. Confirm `--fix` / SARIF / Action stay out of this wave unless trivially cheap.

**Done when:** `lint` runs over a single skill and a repo tree, emits human + JSON,
returns correct CI exit codes, has per-rule tests and fixtures, and the substrate
(collection walker, sectioned report, stable ids) is in place for proj-002.

---

# Part 3 — PROJ-002 kickoff: the audit (library health + security + provenance)

⤵ **Paste this to Frame proj-002, after proj-001 ships.**

**Goal of this wave:** an `audit` command that analyzes a *collection* of skills —
the differentiated core. Think "SBOM + health report for a skill library." Reuses
proj-001's parser, model, tree-walker, and sectioned report.

**Framing:** lint is per-file and gates CI; audit is per-collection and produces a
report a human reads periodically or before enabling third-party skills. The fuzzy
checks that would be noisy as CI gates are appropriate here (per D3).

**Suggested stages**
1. **Inventory + library health.** `audit <path>` producing: an inventory of every
   skill (name, size, location); **description overlap/collision** detection (near-
   duplicate descriptions confuse agent routing — high-value); oversized or
   likely-dead skills; description-vs-body coherence. Sectioned report reusing
   proj-001's report layer.
2. **Permissions manifest.** Per skill, surface *what it can do*: declared
   `allowed-tools`, presence/type of `scripts/`, network hints — flag anything
   execute- or network-capable. A `--security` focus mode. Motivated by real
   guidance to audit third-party skills before enabling exec/network.
3. **Provenance & integrity (SBOM).** A lockfile (e.g. `.skillport.lock`) recording
   a **content hash + observed source** per skill. On later audits, flag **drift**
   ("modified since recorded"), **new/unknown** skills, and unrecognized sources.
   Per D6, this is hash-anchored — self-asserted `metadata` is reported but never
   trusted as provenance.

**Where the value concentrates:** stages 2–3 fuse into one signal nothing else
emits today — *"this skill, from source X, can run Bash and reach the network, and
it has changed since you recorded it."* That's the trust gap in the emerging skill
marketplaces.

**Open questions to surface in Frame**
1. One `audit` command with sections + a `--security` flag, or split `audit`
   (health) from a separate `provenance`/`sbom` command? (Lean: one command,
   sections, focus flags — provenance and permissions reinforce each other.)
2. Lockfile format and location — `.skillport.lock` (TOML/JSON?) at the audited
   root; committed to the repo or not?
3. Overlap detection method — exact/normalized string match to start, or embed
   descriptions for semantic similarity later? (Lean: start lexical, no ML dep.)
4. What counts as a recognized "source" for provenance (git remote, registry,
   local)? Define the minimal viable set.

**Done when:** `audit` walks a library, reports inventory + health + a permissions
manifest, maintains a hash-anchored lockfile with drift/unknown-source detection,
and surfaces the fused capability-plus-drift risk signal.

---

*Note: I've given the content, not the exact `.repo-context.yaml` schema — map
Part 1 into whatever fields that file and `decisions/` actually use once `just
init` scaffolds them.*
