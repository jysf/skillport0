# Kickoff: `skillport` — an Agent Skills validator/linter

You are helping me build a small, focused CLI tool. Before writing code, read this
whole brief, restate the plan in your own words, and flag any of the "Open
questions" that block you. Do not invent platform constraints — verify them from
primary docs (see References) or mark them advisory.

## What we're building

A command-line **validator/linter for agent Skills** (`SKILL.md` files). It checks
a skill against the open Agent Skills specification and, optionally, against a
specific platform's extra constraints, and it can run over an entire repository of
skills in CI.

## Why this shape (don't re-scope past this)

Skills that comply with the open standard are portable across Claude Code, Codex,
Cursor, Gemini CLI, etc. by design — so a generic "convert a skill from format A to
format B" tool is close to a no-op, and distribution/migration is already covered
by Vercel's `npx skills` and Cursor's built-in `/migrate-to-skills`. The durable,
less-crowded value is **validation + normalization with per-platform awareness and
bulk/CI ergonomics**. Keep the tool focused there.

## In scope (v1)

- Parse a `SKILL.md` (YAML frontmatter + Markdown body) into a canonical form,
  losing nothing (preserve unknown keys and order).
- Validate against the open-spec rules below.
- Optional `--target <platform>` layer that additionally recognizes that
  platform's extension fields (so a Claude-only field isn't flagged as unknown
  when linting for Claude).
- Lint a single `SKILL.md`, a skill folder, or a whole directory tree of skills.
- Human-readable output and `--json` output.
- CI-friendly exit codes; `--strict` to also fail on warnings.

## Out of scope for v1 (note as possible later, don't build now)

- Generic skill→skill reformatting as a headline feature.
- Semantic migration between *rules* (`.cursor/rules/*.mdc`, `AGENTS.md`,
  `CLAUDE.md`) and *skills*. This is a real, lossy transformation (`globs` /
  `alwaysApply` have no skill equivalent; a skill's routing `description` has no
  rule equivalent) and needs its own design pass with explicit lossiness
  reporting — not a v1 afterthought.
- Autofix (`--fix`) — see Open questions.

## References (use primary sources; don't assert unverified constraints)

- **Open spec (authoritative):** https://agentskills.io/specification
- An official open-spec validator already exists: `skills-ref validate`
  (github.com/agentskills/agentskills). This means the open-spec checks are table
  stakes — our differentiation is the **per-platform layer + whole-tree/CI
  ergonomics in one static binary**. Position the tool accordingly.
- Verify platform fields/paths from primary docs before encoding them: Claude
  (docs.claude.com / platform.claude.com skills docs), Cursor (cursor.com/docs
  skills), OpenAI Codex (Codex docs / `AGENTS.md`), Vercel (skills.sh / Vercel
  docs).
- **Only the open-spec layer is currently verified.** Treat every per-platform
  constraint as unverified until confirmed from that platform's own docs. Where
  you can't verify, make the rule advisory (info) and cite the source in a
  comment, rather than emitting an error.

## Open-spec rule catalog (implement exactly these; source: agentskills.io)

Severity meaning: **error** = spec violation, **warning** = recommended-practice or
likely-wrong, **info** = advisory.

| Rule id                    | Severity | Check |
|----------------------------|----------|-------|
| `frontmatter.missing`      | error    | YAML frontmatter block present |
| `name.required`            | error    | `name` present |
| `name.type`                | error    | `name` is a string |
| `name.length`              | error    | 1–64 characters |
| `name.charset`             | error    | lowercase letters, digits, hyphens only |
| `name.hyphen-edges`        | error    | no leading/trailing hyphen |
| `name.hyphen-consecutive`  | error    | no `--` |
| `name.dir-match`           | warning  | `name` equals parent directory name |
| `description.required`     | error    | `description` present |
| `description.type`         | error    | `description` is a string |
| `description.length`       | error    | 1–1024 characters, non-empty |
| `description.detail`       | info     | too terse to convey *when* to use (soft; tune to avoid noise) |
| `compatibility.length`     | error    | ≤500 chars if present |
| `metadata.type`            | warning  | is a key→value map |
| `metadata.values`          | info     | values are strings (spec is string→string) |
| `allowed-tools.format`     | warning* | space-separated string, not a YAML list (*info for platforms known to accept a list) |
| `body.empty`               | warning  | body is non-empty |
| `body.lines`               | warning  | ≤500 lines recommended |
| `body.size`                | warning  | under ~5000 tokens recommended (estimate method = open question) |
| `frontmatter.unknown`      | info     | key is recognized (widen the recognized set per `--target`) |

Rules that don't apply on a spec-perfect skill must produce zero findings. No
heuristic/soft rule may be error-level.

## CLI surface (v1)

- `skillport lint <path> [--target <platform>] [--json] [--strict]`
  - `<path>` = a `SKILL.md`, a skill folder, or a tree containing many skills.
  - Exit non-zero if any error (or any warning under `--strict`).
- Optional if cheap: `skillport inspect <path>` — dump the parsed canonical view
  for debugging.
- Keep the architecture open to later `convert` / `push` subcommands, but do not
  implement them in v1.

## Non-functional requirements

- Single self-contained binary; lints hundreds of skills in a repo with negligible
  overhead.
- Deterministic, stable output (sort skills by path); stable `--json` schema
  suitable for CI parsing.
- Invalid YAML in one skill must **not** abort a bulk run — report it as a per-file
  error finding and keep going.
- Messages are actionable: name the field and the fix.
- Every rule addressable by a stable id (enables future per-repo config).

## Suggested architecture

- `parse`: read file (tolerate a BOM, leading blank lines, and a missing
  frontmatter block); split `---` frontmatter from body; parse YAML preserving
  unknown keys and order.
- canonical `Skill`: full order-preserving frontmatter + body + source dir.
- `lint`: pure `(skill, dir_name, target) -> Vec<Finding>`; each rule small and
  independently unit-testable.
- platform profiles as **data** (recognized-field sets, and any install-path
  conventions), easy to extend as specs evolve.
- `report`: human + JSON renderers and the exit-code logic.
- tree walker for bulk mode (skip `.git`, `node_modules`, `target`).

## Language / stack

Rust, producing a static binary — best fit for the CI-tool goal. Swap only with a
strong reason.

## Optional starting point

A working prototype (`skillport`) already implements parse + this exact rule
catalog + human/JSON output + exit codes + bulk mode, with good/bad fixtures. You
may build on it or start fresh. If you build on it: the `open`-layer rules are
spec-backed and trustworthy; the `claude` / `cursor` / `codex` / `vercel` profiles
are **unverified guesses** and must be checked against primary docs before
shipping. Any dependency version pins in the prototype were an old-toolchain
artifact — use current versions.

## Open questions — surface these before/while building (ask me)

1. **Platform coverage for v1** — which of claude / cursor / codex / vercel to
   verify and support first?
2. **Token-size estimate** — a rough chars-or-words heuristic (advisory only), or a
   real tokenizer? Default: heuristic, info-level.
3. **`--fix` in v1?** Candidate safe fixes: quote non-string metadata values,
   convert an `allowed-tools` list to a string, normalize
   `compatible_agents`→`agents`. Default: out of v1 unless cheap.
4. **Per-repo config** (e.g. `.skillport.toml` to enable/disable/re-severity
   rules)? Default: not in v1, but keep rule ids stable so it's easy to add.

## Deliverables

- Working `lint` CLI with the rule catalog and both output modes.
- Unit tests per rule; good/bad fixture skills; a test asserting a spec-perfect
  skill yields zero findings.
- README with usage, the rule list (ids + severities), and a CI snippet.
- A short note on how to add a rule and a platform profile.

## How to work

Restate the plan first. Don't encode any platform-specific constraint you haven't
confirmed from that platform's primary docs; when unsure, make it advisory and cite
the source in a comment. Prefer small, independently testable rules with stable ids.
