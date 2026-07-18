# SPEC-001 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-001-<cycle>.md`.

## Instructions

- [x] **design** — completed 2026-07-17 (architect: claude-opus-4-8)
- [ ] **build** — prompt: `prompts/SPEC-001-build.md` (start a fresh session on branch `feat/spec-001-parser`)
- [ ] **verify** — prompt: pending (waiting on build)
- [ ] **ship** — prompt: pending (waiting on verify)
