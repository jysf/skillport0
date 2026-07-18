# SPEC-008 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-008-<cycle>.md`.

## Instructions

- [x] **design** — completed 2026-07-18 (architect: claude-opus-4-8)
- [ ] **build** — prompt: `prompts/SPEC-008-build.md` (runs as a **Sonnet subagent** on branch `feat/spec-008-sarif`)
- [ ] **verify** — prompt: `prompts/SPEC-008-verify.md` (runs as an **Opus subagent**; waiting on build)
- [ ] **ship** — prompt: pending (waiting on verify)
