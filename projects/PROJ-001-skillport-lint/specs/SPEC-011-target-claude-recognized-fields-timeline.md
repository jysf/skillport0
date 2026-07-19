# SPEC-011 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-011-<cycle>.md`.

## Instructions

- [x] **design** — completed 2026-07-18 (architect: claude-opus-4-8) · incl. primary-doc research (WebFetch code.claude.com) per DEC-002
- [ ] **build** — prompt: `prompts/SPEC-011-build.md` (runs as a **Sonnet subagent** on branch `feat/spec-011-target-claude`)
- [ ] **verify** — prompt: `prompts/SPEC-011-verify.md` (runs as an **Opus subagent**; waiting on build)
- [ ] **ship** — prompt: pending (waiting on verify) — **first verified per-platform target**
