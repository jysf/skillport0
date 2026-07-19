# SPEC-012 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-012-<cycle>.md`.

## Instructions

- [x] **design** — completed 2026-07-18 (architect: claude-opus-4-8) · incl. design-time doc re-verification of the 5 new CLAUDE_KEYS (WebFetch code.claude.com) per DEC-002 + full severity probe of src/rules.rs + src/report.rs (the 26-id catalog table)
- [ ] **build** — prompt: `prompts/SPEC-012-build.md` (runs as a **Sonnet subagent** on branch `feat/spec-012-rule-reference-readme`)
- [ ] **verify** — prompt: pending (waiting on build) — will be an **Opus subagent**
- [ ] **ship** — prompt: pending (waiting on verify) — **last STAGE-003 spec; then close the stage**
