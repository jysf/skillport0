# SPEC-018 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-018-<cycle>.md`.

## Instructions

- [x] **design** — completed 2026-07-19 (architect: claude-opus-4-8) · probed the CLI (audit pre-anticipated as a subcommand), the Skill/report/emit surface, and the private body_token_count; framing decisions applied (tokens as headline metric; inventory-only)
- [ ] **build** — prompt: `prompts/SPEC-018-build.md` (runs as a **Sonnet subagent** on branch `feat/spec-018-audit-inventory`)
- [ ] **verify** — prompt: pending (waiting on build) — **Opus subagent**
- [ ] **ship** — prompt: pending (waiting on verify) — first PROJ-002 / `audit` spec
