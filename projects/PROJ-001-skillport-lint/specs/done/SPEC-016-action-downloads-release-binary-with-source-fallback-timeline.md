# SPEC-016 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-016-<cycle>.md`.

## Instructions

- [x] **design** — completed 2026-07-19 (architect: claude-opus-4-8) · probed action.yml + SPEC-014 archive naming/layout; designed a testable scripts/install-release.sh with a --print-plan dry mode + a fallback-only toolchain step
- [x] **build** — completed 2026-07-19 (implementer: claude-sonnet-5) · prompt: `prompts/SPEC-016-build.md` (ran as a **Sonnet subagent** on branch `feat/spec-016-action-download`) · scripts/install-release.sh + action.yml prebuilt-install step + README note; shellcheck/actionlint clean, print-plan + real fallback run verified
- [x] **verify** — completed 2026-07-19 (Opus subagent, 70,399 tok/~$0.46/~9 min) — ✅ APPROVED, 0 punch-list; shellcheck + gates + --print-plan (all pairs) + real fallback; asset naming matches SPEC-014; happy path runs no Rust step
- [x] **ship** — completed 2026-07-19 (PR #16 squash-merged a85a6d7) — STAGE-004 step 4 shipped; download-success path first exercised at v0.1.0 (SPEC-017/human)
