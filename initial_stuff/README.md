# skillport

A small Rust CLI that takes an agent **`SKILL.md`** in one platform's format and
converts / pushes it into another.

Claude, Cursor, Codex, and Vercel's `skills.sh` all share the open
[Agent Skills](https://agentskills.io) standard: YAML frontmatter (`name` +
`description` required) followed by a Markdown body. The real differences
between "formats" are only:

1. **which frontmatter fields** a platform honors (e.g. Claude Code adds
   `allowed-tools`, `context: fork`, `effort`, `hooks`; the portable core keeps
   only `name` / `description` / `metadata`), and
2. **where the file is installed** (`.claude/skills/`, `.cursor/skills/`, …).

`skillport` parses a skill once into a canonical form (losing nothing), then
re-emits it for a target profile — adjusting the frontmatter and writing it into
that platform's directory layout.

## Build

```bash
cargo build --release
# binary at target/release/skillport
```

## Usage

```bash
# See how a skill parses and which platform it looks like it came from
skillport inspect ./my-skill/            # file or folder both work

# List target platforms
skillport profiles

# Convert into an output dir (default ./out)
skillport convert ./my-skill --to vercel
skillport convert ./my-skill --to open --stdout      # print, don't write
skillport convert ./my-skill --to claude --keep-all  # keep every field

# Sync straight into a project's install folder (.cursor/skills/, …)
skillport push ./my-skill --to cursor --dest ./my-project
```

### Lint / validate

Check a skill against the open [Agent Skills spec](https://agentskills.io/specification)
(name/description format and length, `compatibility`/`metadata`/`allowed-tools`
shape, body size) — and optionally against a platform's recognized fields.

```bash
skillport lint ./my-skill                  # one skill, open spec
skillport lint ./my-skill --target claude  # also accept Claude's extension fields
skillport lint ./skills-repo               # lint every SKILL.md in a tree
skillport lint ./skills-repo --json        # machine-readable (CI)
skillport lint ./skills-repo --strict      # warnings also fail the build
```

Findings have three severities: **error** (spec violation), **warning**
(recommended-practice / likely-wrong), **info** (advisory, e.g. an unrecognized
frontmatter key). Exit code is non-zero if any error is found (or any warning
under `--strict`), so it drops straight into CI.

The open-spec checks overlap with the official `skills-ref validate`; what this
adds is the **per-platform field layer** and linting a **whole tree** in one
pass with a CI exit code. Rules live in `src/lint.rs` and are easy to extend.

`--from` is not needed: the source format is auto-detected from the frontmatter
and path.

### Targets

| id       | install path                     |
|----------|----------------------------------|
| `open`   | `skills/<name>/SKILL.md`         |
| `claude` | `.claude/skills/<name>/SKILL.md` |
| `cursor` | `.cursor/skills/<name>/SKILL.md` |
| `codex`  | `.codex/skills/<name>/SKILL.md`  |
| `vercel` | `skills/<name>/SKILL.md`         |

## What it does / doesn't do

- **Lossless in, honest out.** All frontmatter is preserved on parse. On
  convert, fields the target doesn't honor are dropped **with a warning** — or
  kept via `--keep-all`.
- **Field normalization.** Known cross-standard aliases are reconciled (e.g.
  `compatible_agents` → `agents`).
- **Resource folders** (`scripts/`, `references/`, `assets/`) are copied
  alongside the converted `SKILL.md`.
- **Profiles are just data.** The honored-field lists and install paths live in
  `src/profiles.rs` and reflect the open spec plus each platform's documented
  extensions — edit them as the specs evolve.

## Layout

```
src/
  main.rs      CLI (clap): inspect / convert / push / lint / profiles
  parse.rs     read SKILL.md, split frontmatter from body
  skill.rs     canonical Skill representation + slug
  profiles.rs  per-platform field allow-lists, renames, install paths
  emit.rs      render frontmatter for a target + write files/resources
  lint.rs      validation rules (open spec + per-platform layer)
fixture/         an example Claude skill (for convert)
lint-fixtures/   good/ and bad/ example skills (for lint)
```
