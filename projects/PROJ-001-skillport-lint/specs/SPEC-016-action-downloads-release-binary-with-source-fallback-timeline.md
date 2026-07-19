# SPEC-016 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-016-<cycle>.md`.

## Instructions

- [x] **design** — completed 2026-07-19 (architect: claude-opus-4-8) · probed action.yml + SPEC-014 archive naming/layout; designed a testable scripts/install-release.sh with a --print-plan dry mode + a fallback-only toolchain step
- [ ] **build** — prompt: `prompts/SPEC-016-build.md` (runs as a **Sonnet subagent** on branch `feat/spec-016-action-download`)
- [ ] **verify** — prompt: pending (waiting on build) — **Opus subagent**
- [ ] **ship** — prompt: pending (waiting on verify) — STAGE-004 step 4 (Action speedup)
