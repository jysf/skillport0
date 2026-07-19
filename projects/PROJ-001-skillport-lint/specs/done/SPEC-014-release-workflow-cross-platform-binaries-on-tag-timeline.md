# SPEC-014 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-014-<cycle>.md`.

## Instructions

- [x] **design** — completed 2026-07-18 (architect: claude-opus-4-8) · probed build-info.sh (provenance), ci.yml action patterns, the DEC-009 5-target matrix; designed workflow_dispatch dry path so it's CI-testable without a tag
- [x] **build** — completed 2026-07-18 (Sonnet subagent, 78,703 tok/~$0.52/~7 min) on branch `feat/spec-014-release-workflow`; release.yml 5-target matrix + gh release job + workflow_dispatch dry path; actionlint clean, local archive round-trip proven
- [x] **verify** — completed 2026-07-18 (Opus subagent, 67,588 tok/~$0.45/~6 min) — ✅ APPROVED, 0 punch-list; actionlint + gates + local round-trip, tag-only release guard + real musl cross confirmed, scope diff empty
- [x] **ship** — completed 2026-07-18 (PR #14 squash-merged d5ba368) — STAGE-004 step 2 shipped; full 5-leg matrix confirmed at first human workflow_dispatch/tag
