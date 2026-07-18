# SPEC-007 — VERIFY prompt (Opus subagent)

You are an **independent verifier** for `SPEC-007: surface unreadable directories
as findings`, run as a metered subagent. A separate Sonnet build session
implemented it and committed to branch `feat/spec-007-unreadable-dir`. You did NOT
build it. Disprove "done." **Do not modify code, merge, or advance the cycle** —
return a verdict.

## Review the diff

```bash
git diff main...HEAD -- src/ tests/
```

## Read (in order)

1. `projects/PROJ-001-skillport-lint/specs/SPEC-007-surface-unreadable-directories-as-findings.md`
   — Goal, Design decisions (warning; section-per-item), ACs, Failing Tests, Out of
   scope, Build Completion/deviations.
2. `src/walk.rs`, `src/report.rs`; `decisions/DEC-003`/`DEC-004`/`DEC-005`.

## Verify — run it, don't trust names

```bash
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
cargo build
```

Then reproduce the behavior yourself (Unix):
```bash
mkdir -p /tmp/sp7/good && printf -- '---\nname: good\ndescription: a description long enough to pass the length and detail checks here.\n---\n# b\n' > /tmp/sp7/good/SKILL.md
mkdir -p /tmp/sp7/locked && chmod 000 /tmp/sp7/locked
./target/debug/skillport lint /tmp/sp7 ; echo "exit=$?"            # dir.unreadable warning + the good skill; exit 0
./target/debug/skillport lint /tmp/sp7 --strict ; echo "exit=$?"   # exit 1
chmod 755 /tmp/sp7/locked; rm -rf /tmp/sp7
```

Adversarially probe:
- **Walk never aborts (DEC-005):** the unreadable dir does NOT drop the sibling
  `good` skill; both appear. Try the unreadable dir as the `root` too.
- **`dir.unreadable`:** exactly one Warning finding per unreadable dir, correct
  path, literal id `"dir.unreadable"`; `summary.skills` NOT incremented by it;
  `summary.warnings` is.
- **Exit code:** `dir.unreadable` alone → exit 0 non-strict, 1 under `--strict`
  (via `Report::exit_code`). Info/nothing-else unaffected.
- **Ignored dirs:** a `.git`/`node_modules`/`target` dir that is also unreadable is
  still silently skipped — NOT reported as `dir.unreadable`.
- **Determinism:** `UnreadableDir` sections path-sorted with the rest; run twice,
  diff byte-identical.
- **Scope:** no new lint rule, no `key.duplicate` added, no `body.size`/`--target`/
  `--sarif`, no CLI/emitter change (except an optional test), no new dependency.
  The good fixture stays clean (0/0/0).
- Judge the **warning** severity choice (vs error) — sound, or should an unreadable
  dir be an error like `file.unreadable`?

## Return a verdict (final message = data for the orchestrator)

**✅ APPROVED** / **⚠ PUNCH LIST** (numbered, file:line + failing case) /
**❌ REJECTED** (which criterion, concrete command → observed vs expected). Include
gate results, your actual chmod-000 run + exit codes, a per-AC pass/fail summary,
and judge the severity decision + any deviations. Every flag needs a concrete
input→observed/expected. Don't touch code.
