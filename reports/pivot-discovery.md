# Pivot Discovery Spike — memo

**Date:** 2026-07-19 · **Type:** discovery spike (no code, no implementation) ·
**Author:** orchestrator (Claude Opus) · **Status:** observations for a decision, not a build plan.

> **Update:** agnix *was* run (via `npx agnix`, v0.40.0) after an initial "not installed"
> pass. This memo reflects the real head-to-head. Where the earlier draft substituted
> skillport as a proxy, real agnix data supersedes it — and sharpens the conclusions.

## TL;DR

- **Lint-first is a losing race — now confirmed from the competitor's actual capability,
  not just our own output.** agnix is not "a linter that's a bit ahead." It's a broad,
  **cited, normative** validator (v0.40.0) spanning **Skills · MCP · Hooks · Memory ·
  Plugins · AGENTS.md across Claude/Cursor/Codex/Kiro**, with autofix and a rule-eval
  harness. On the same 1,449 real skills it found **202 errors / 5,599 warnings** vs
  skillport's **1 error / 245 warnings**. Competing on validation breadth is hopeless and
  pointless.
- **agnix already occupies most of what we considered pivoting *to*:** security/permissions
  (dangerous-skill, unrestricted-Bash, author-leaking paths), reference integrity, AGENTS.md
  governance + context-budget, prompt-quality. Our planned STAGE-006 (`--security`
  permissions manifest) and the parked AGENTS.md-audit idea are **largely already built** by
  agnix.
- **Both suspected skillport "bugs" are refuted** — from primary docs, from the real corpus,
  and from agnix's own rules. Do not pivot on a false-error premise; we don't false-error in
  practice.
- **Exactly two ideas survive contact with the mature competitor:** **provenance/drift**
  (agnix has *zero* provenance/hash/lockfile rules — it's a stateless snapshot validator) and
  **triggering/routing measurement** (agnix's `eval` is linter-QA, not "does the skill
  actually get invoked"). These are the only uncontested, skills-native gaps.

---

## What agnix actually is (the reframe)

`npx agnix` → v0.40.0. *"Validate agent specifications across Claude Code, Cursor, Codex,
and beyond. Validates: Skills • MCP • Hooks • Memory • Plugins."* Subcommands: `validate`,
`init`, **`eval`** (rule efficacy vs labeled cases), **`explain`** (rule by ID), `schema`,
`tools`, `--fix`/`--fix-safe`/`--fix-unsafe`, `--target {generic,claude-code,cursor,codex,
kiro}`, locales, SARIF/GitHub output.

Its rules are **sourced and normative**. `agnix explain AS-004` →
*"Invalid Name Format · Severity: HIGH · Normative level: MUST · Source type: spec ·
Verified on: 2026-02-04 · Sources: agentskills.io/specification · Autofix: true."* This is a
serious, maintained product, not a fixture toy.

**Rule namespaces observed firing:** `CC-SK` (Claude skills), `AS` (agent-skills spec), `AGM`
(AGENTS.md), `REF` (references), `PE` (prompt-engineering), `CC-MEM` (Claude memory),
`CDX-AG` (Codex agents), `XP` (cross-platform), `XML`, `VER` (version-awareness). Categories
by diagnostic volume: claude-skills 5032, agent-skills 298, cross-platform 153, references
132, xml 104, prompt-engineering 31, claude-memory 21, codex 20, agents-md 11.

---

## TASK 1 — Head-to-head on 1,449 real skills (`~/.claude/skills`)

| | skillport | agnix v0.40.0 |
|---|---|---|
| errors | **1** | **202** |
| warnings | 245 | 5,599 |
| infos | 4,866 | 2 |
| files with ≥1 error | 1 | 68 |
| scope | open skill spec only | skills + MCP + hooks + memory + AGENTS.md, 5 platforms |

**agnix's real errors/warnings are substantive and actionable** (skillport catches none of
these):

| count | rule | what it catches |
|---|---|---|
| 4,696 (warn) | `CC-SK-017` | Unknown frontmatter field — **the same firehose as skillport, but at *warning* not *info*** |
| 239 | `CC-SK-021` | Hardcoded user dir `C:\Users\renat` — **leaks author identity** (privacy) |
| 195 | `AS-012` | Skill body > 500 lines |
| 151 | `XP-003` | Hard-coded Windows absolute path (portability) |
| **99 (err)** | `XML-001` | Unclosed XML tag in the body (structural) |
| 98 | `AS-013` | File reference nested deeper than one level |
| **64 (err)** | `REF-001` | Absolute import path not allowed (broken ref) |
| 52 | `CC-SK-007` | **Unrestricted Bash access** — security |
| **22 (err)** | `CC-SK-006` | **Dangerous skill must set `disable-model-invocation`** — security |
| 21 | `PE-001` | Critical keyword `NEVER` at 47% of doc (prompt-quality heuristic) |
| 20 | `CDX-AG-005` | AGENTS.md references a missing file |
| 18 | `CC-MEM-006` | Negative instruction without a positive alternative |
| 15 | `CC-SK-012` | `argument-hint` set but body never references `$ARGUMENTS` |
| — | `AGM-003 / AGM-006` | AGENTS.md over char-budget / multiple-AGENTS.md detected |

**The shared non-signal:** both tools' #1 rule is "unknown frontmatter field" (~4.7k), driven
by real-world `risk`/`source`/`date_added`/`tags`/`version` metadata the open spec doesn't
list. **agnix emits these as *warnings*; skillport as *infos* — so on real skills agnix is
the *noisier* one on the dominant non-signal.** But agnix earns its noise: underneath it, it
surfaces 200+ genuinely actionable findings skillport is blind to.

**Felt experience verdict:** skillport = near-silent-and-mildly-noisy; agnix =
loud-but-substantive. The validation race isn't close, and it's the wrong race to enter.

---

## TASK 2 — Two suspected bugs, verified (docs + corpus + agnix itself)

**Verdict: both refuted. Not a real-world false-error. Do not pivot on this premise.**

**(a) Description 1024 vs 1536.** agentskills.io (authoritative, DEC-002): *"Max 1024
characters … Must be 1-1024."* Claude docs: the 1,536 is a **configurable listing-display
truncation** (`skillListingMaxDescChars`) of *description + when_to_use combined*, not a
validity cap. Corpus: **max description = 619 chars; 0 skills exceed 1024.** skillport's 1024
is spec-correct; agnix's "1536 for claude-code" conflates display-budget with validity.

**(b) Name "per segment"/namespacing (agnix AS-004).** `agnix explain AS-004` shows it
enforces the **same** charset skillport does — bad `Run_Tests!` → good `run-tests`, sourced
to agentskills.io, MUST. The "1-64 per segment" wording is an agnix **extension** allowing
`/`-namespaced segments; the authoritative spec's plain text is flat `[a-z0-9-]`, no slashes.
Corpus: **0 names contain `/`.** So skillport and agnix *agree in practice*; the extension is
untested by any real skill. skillport is spec-correct.

> The "agnix implies we emit false errors" motivation does not survive the primary sources
> **or agnix's own rule definitions.** A negative result — and a load-bearing one.

---

## TASK 3 — SPIKE: is skill *triggering* measurable? (and does agnix already do it? **No.**)

agnix's `eval` = *"Evaluate rule efficacy against labeled test cases"* (filter by rule prefix
like `AS-`) — it's **QA for agnix's own lint rules**, not a measure of whether a skill's
description causes the agent to invoke it. Its `PE-*` rules judge prompt *phrasing*, not
routing. **Triggering is unowned even by this mature tool.**

**In-session probe** (no API): four real corpus descriptions, three deliberately confusable
(`seo-keyword-strategist` under test, vs `seo-content-writer`, `seo-content-auditor`,
`sql-injection-testing` as distractors). 10 should-trigger + 10 near-miss queries, routed by
me, then the description mutated to vague *"Helps improve your content for search engines."*

| description | hit-rate (10 should) | false-trigger (10 near-miss) |
|---|---|---|
| baseline (specific) | ~9/10 | ~1/10 |
| mutated (vague) | ~4/10 | ~4/10 |

The vague description **lost the keyword-specific tokens that were the discriminator** and
**gained overlap with its SEO siblings** — hit-rate fell, false-triggers rose, directionally
and mechanistically.

**Skeptic's read:** I authored *and* judged the queries → self-consistency, not an
independent signal; n tiny; mutation direction expected. **But** the effect is large,
directional, and mechanistically explained, so the underlying phenomenon (description quality
measurably changes routing) is plausibly real and **not obviously noise**. The corpus makes
it concrete — dozens of confusable clusters (`seo-*`, `azure-*`, `odoo-*`, multiple
`code-review`/commit variants) where mis-routing is a live failure. **This is the freshest,
most differentiated, *unowned* signal in the whole spike.**

---

## TASK 4 — Agent config at ORG scale

Inventory (`~/PSeven` + `~/.claude`): `SKILL.md` 1,550 (**1,449 in one managed dir**);
`CLAUDE.md` 63 — **~50 byte-identical copies of ~5 originals**, drifting across repos
(mtimes 2026-04 → 07); `AGENTS.md` 51; `.mcp.json` 20; hooked `settings.json` 5; Cursor 0.

**Real config-sprawl/drift problem — but two inconvenient truths:**
1. It's concentrated in the **agent-config lane** (CLAUDE.md/AGENTS.md duplication), which we
   parked. Skills themselves are the *tidy* part (one managed dir, uniform metadata).
2. **agnix already covers it:** `AGM-006` (multiple-AGENTS.md detected), `AGM-003` (AGENTS.md
   over char-budget = the context-budget idea from the parked `agents-md-audit` signal),
   `CDX-AG-005` (AGENTS.md missing-file refs). The governance lane is not a green field.

**One signal that cuts toward provenance:** real skills already carry `source` (1,407),
`risk` (1,408), `date_added` (1,182) frontmatter — latent demand for source/provenance
tracking, currently met by an ad-hoc unstandardized convention (which both linters flag as
"unknown").

---

## Competitive map — who owns what

| capability | agnix v0.40.0 | skillport | reality |
|---|---|---|---|
| Open-spec + multi-tool validation | comprehensive, cited, normative, 5 platforms | 26 rules, skills-only | **agnix wins decisively** |
| Autofix | yes (safe/unsafe) | no (out of scope) | agnix |
| Security / permissions | `CC-SK-006/007/021` | *planned* STAGE-006 | **agnix already owns** |
| Config governance / context-budget / multi-file | `AGM-*`, cross-platform | parked | **agnix already owns** |
| Reference integrity | `REF-*` | none | agnix |
| Prompt-quality heuristics | `PE-*`, `CC-MEM-*` | none | agnix |
| **Provenance / drift / content-hash** | **none** (stateless validator) | **DEC-006 thesis** | **GAP — uncontested** |
| **Triggering / description-routing measurement** | **none** (`eval` = rule QA) | Task-3 idea | **GAP — uncontested + novel** |

---

## TASK 5 — Which is a real problem, which is a solution in search of one?

1. **Validation / security / governance — DO NOT PURSUE as the product.** Not because they're
   unreal, but because **agnix already owns them comprehensively.** Entering here is fighting
   a mature, cited, multi-platform incumbent on its home turf with 1/8th the coverage. This
   includes our own planned STAGE-006 (`--security`) and the parked AGENTS.md lane — both are
   largely agnix territory now. (Keep skillport's `lint` as a small free feature; stop
   treating validation as strategy.)

2. **Triggering / routing measurement — the strongest differentiated bet. Investigate
   first.** Uncontested even by agnix, skills-native, targets a real failure (confusable
   clusters), and the spike showed a signal that *moved with the input*. Biggest risk: the
   effect collapses under blinding/multi-model — which a cheap next experiment settles.

3. **Provenance / drift — the other uncontested gap; realer than validation, softer than it
   looks.** agnix structurally cannot do it (stateless snapshot validator; no lockfile/state,
   0 provenance rules). Demand exists (skills already carry `source`/`date_added`). But the
   local reality is one tidy managed dir with an ad-hoc convention already half-filling the
   need, so a heavyweight hash-anchored SBOM may over-serve. Interrogate *what `source:`
   can't do that a drift-tracking lockfile can* before committing.

4. **The suspected false-error bugs — DEAD END.** Refuted three ways. Don't let them motivate
   anything.

**Solution-in-search-of-a-problem flags:** (a) building *any* validation/security/governance
feature now — agnix ate that lunch; (b) a full provenance SBOM before confirming the ad-hoc
`source:` convention is actually insufficient; (c) more `--target` platform work (Task 1:
moves nothing on real data).

## Recommended next investigation (NOT a build)

1. **A properly-controlled triggering experiment** — confusable clusters mined from the
   corpus; should/should-not queries generated by a *different* process; routing via the real
   API, n≥5/query, ≥2 models; independent grader; measure whether description edits produce a
   *stable* hit/false-trigger delta. **Kill criterion:** no stable signal under blinding →
   drop it. Cheap, decisive, and it probes the one thing no competitor measures.
2. **Interrogate the provenance demand** before building it: what question does a
   drift-tracking lockfile answer that the in-the-wild `source:`/`date_added:` convention
   can't? If thin, deprioritize.
3. **One conscious strategic decision:** given that agnix already owns validation + security +
   governance, is skillport's edge **triggering-measurement**, **provenance/drift**, or a
   fusion (*"is this skill trustworthy AND does it actually fire?"*)? Pick deliberately; do
   not drift back toward validation, which is lost.

*No build recommended. Each item above is an investigation with a kill criterion.*
