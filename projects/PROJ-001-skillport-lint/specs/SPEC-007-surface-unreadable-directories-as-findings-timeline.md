# SPEC-007 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-007-<cycle>.md`.

## Instructions

- [x] **design** — completed 2026-07-18 (architect: claude-opus-4-8)
- [ ] **build** — prompt: `prompts/SPEC-007-build.md` (runs as a **Sonnet subagent** on branch `feat/spec-007-unreadable-dir`)
- [ ] **verify** — prompt: `prompts/SPEC-007-verify.md` (runs as an **Opus subagent**; waiting on build)
- [ ] **ship** — prompt: pending (waiting on verify) — **last STAGE-002 spec**
