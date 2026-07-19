# SPEC-012 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-012-<cycle>.md`.

## Instructions

- [x] **design** — completed 2026-07-18 (architect: claude-opus-4-8) · incl. design-time doc re-verification of the 5 new CLAUDE_KEYS (WebFetch code.claude.com) per DEC-002 + full severity probe of src/rules.rs + src/report.rs (the 26-id catalog table)
- [x] **build** — completed 2026-07-18 (Sonnet subagent, 159,737 tok/~$1.05/~33 min) on branch `feat/spec-012-rule-reference-readme`; 131 tests, no new dep; 26-id catalog + README drift test + 10 fixtures + CLAUDE_KEYS→13
- [x] **verify** — completed 2026-07-18 (Opus subagent, 114,607 tok/~$0.76/~8 min) — ✅ APPROVED, 0 punch-list; independently confirmed the drift test is real + catalog is complete
- [x] **ship** — completed 2026-07-18 (PR #12 squash-merged 0722713) — **last STAGE-003 spec; stage ready to close**
