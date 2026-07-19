# SPEC-017 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-017-<cycle>.md`.

## Instructions

- [x] **design** — completed 2026-07-19 (architect: claude-opus-4-8) · probed `just next-version` (v0.1.0, no bump), confirmed root CHANGELOG.md is template-owned (app notes → GitHub Release), read SPEC-014 asset names + the release.yml notes line
- [ ] **build** — prompt: `prompts/SPEC-017-build.md` (runs as a **Sonnet subagent** on branch `feat/spec-017-cut-v0-1-0`)
- [ ] **verify** — prompt: pending (waiting on build) — **Opus subagent**
- [ ] **ship** — prompt: pending (waiting on verify) — **last STAGE-004 + PROJ-001 spec; then the human pushes v0.1.0**
