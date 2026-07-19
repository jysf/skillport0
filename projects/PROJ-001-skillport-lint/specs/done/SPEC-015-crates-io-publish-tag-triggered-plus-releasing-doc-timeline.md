# SPEC-015 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-015-<cycle>.md`.

## Instructions

- [x] **design** — completed 2026-07-18 (architect: claude-opus-4-8) · re-confirmed crates.io name `skillport` free (404); designed a tag-gated publish job on release.yml + a RELEASING runbook (publish/token/tag are human-only per DEC-009)
- [x] **build** — completed 2026-07-19 (Sonnet subagent, 69,398 tok/~$0.46/~5 min) on branch `feat/spec-015-crates-publish`; tag-gated publish job + RELEASING.md; actionlint clean, crate still 404
- [x] **verify** — completed 2026-07-19 (Opus subagent, 59,911 tok/~$0.40/~2 min) — ✅ APPROVED, 0 punch-list; dispatch-can't-publish + version-guard + secret-not-literal traced; crate 404
- [x] **ship** — completed 2026-07-19 (PR #15 squash-merged 71f4d92) — STAGE-004 step 3 shipped
