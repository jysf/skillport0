# SPEC-001 — VERIFY prompt

> Fresh, independent reviewer session. Review PR #1 (branch `feat/spec-001-parser`)
> against the spec: run the gates yourself, check every Acceptance Criterion
> against the code, hunt for a panic/loss-of-data/ordering bug, and independently
> judge the builder's three flagged deviations (empty fenced block → Present-empty;
> Unclosed → empty body; consumed fence-terminator newline). Return
> ✅ APPROVED / ⚠ PUNCH LIST / ❌ REJECTED with concrete input→observed/expected.
> On approval: review PR #1, flip verify `[x]`, append the verify cost session
> (numerics null), and `just advance-cycle SPEC-001 ship`. Do not merge.

*(Full prompt as handed to the verify session; retained for the spec record.
Verdict: ✅ APPROVED — 14 tests pass, clippy/fmt clean, all ACs met, three
deviations judged reasonable. See the timeline and PR #1.)*
