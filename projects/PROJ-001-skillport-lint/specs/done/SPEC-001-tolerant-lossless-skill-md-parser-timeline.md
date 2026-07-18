# SPEC-001 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-001-<cycle>.md`.

## Instructions

- [x] **design** — completed 2026-07-17 (architect: claude-opus-4-8)
- [x] **build** — completed 2026-07-17 · PR #1 (branch `feat/spec-001-parser`) · prompt: `prompts/SPEC-001-build.md`
- [x] **verify** — completed 2026-07-17 · ✅ APPROVED (independent review of PR #1; 14 tests, clippy/fmt clean, all ACs met) · prompt: `prompts/SPEC-001-verify.md`
- [x] **ship** — completed 2026-07-18 · PR #1 squash-merged to `main` (`692917f`) · cost grandfathered (manual sessions, unmetered — see signal `cost-metering-manual-sessions`) · archived to `specs/done/`
