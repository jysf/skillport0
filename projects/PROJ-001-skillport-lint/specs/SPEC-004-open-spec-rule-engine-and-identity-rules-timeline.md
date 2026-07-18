# SPEC-004 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-004-<cycle>.md`.

## Instructions

- [x] **design** — completed 2026-07-18 (architect: claude-opus-4-8)
- [ ] **build** — prompt: `prompts/SPEC-004-build.md` (runs as a **Sonnet subagent** on branch `feat/spec-004-rules`)
- [ ] **verify** — prompt: `prompts/SPEC-004-verify.md` (runs as an **Opus subagent**; waiting on build)
- [ ] **ship** — prompt: pending (waiting on verify)
