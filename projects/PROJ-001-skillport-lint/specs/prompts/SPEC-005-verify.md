# SPEC-005 — VERIFY prompt (Opus subagent)

You are an **independent verifier** for `SPEC-005: lint command with human and
json output`, run as a metered subagent. A separate Sonnet build session
implemented it and committed to branch `feat/spec-005-cli`. You did NOT build it.
Disprove "done." **Do not modify code, merge, or advance the cycle** — return a
verdict.

## Review the diff

```bash
git diff main...HEAD
```

## Read (in order)

1. `projects/PROJ-001-skillport-lint/specs/SPEC-005-lint-command-with-human-and-json-output.md`
   — CLI surface, exit-code table, exact `--json` schema, ACs, Failing Tests, Out
   of scope, Build Completion/deviations.
2. `docs/api-contract.md` (the contract), `src/main.rs`, `src/emit.rs`,
   `tests/cli.rs`, `decisions/DEC-008` (the new deps), `DEC-001`/`DEC-003`/`DEC-005`.

## Verify — run it, don't trust names

```bash
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
cargo build
```

Then **actually run the binary** and check behavior against the contract:
```bash
BIN=$(cargo build --message-format=json 2>/dev/null | tail -1 >/dev/null; echo target/debug/skillport)
./target/debug/skillport lint lint-fixtures/good ; echo "exit=$?"      # expect 0
./target/debug/skillport lint lint-fixtures/bad  ; echo "exit=$?"      # expect 1
./target/debug/skillport lint lint-fixtures --json ; echo "exit=$?"    # valid JSON, exit 1
./target/debug/skillport lint /no/such/path ; echo "exit=$?"           # exit 2, msg on stderr
./target/debug/skillport lint lint-fixtures --json > /tmp/o.json; python3 -m json.tool /tmp/o.json >/dev/null && echo JSON_OK
```

Adversarially probe:
- **Exit codes exact (DEC-003):** 0 clean; 1 on any error; warning-only → 0 but 1
  under `--strict`; missing path → 2. Confirm the `--strict` warning flip with a
  warning-only input. Confirm info never changes the code.
- **`--json` schema exact (DEC-005):** `tool`/`version`/`schema`/`target`/`summary`/
  `sections` present; `version==CARGO_PKG_VERSION`; `schema==1`; severities are
  lowercase strings; sections path-sorted; findings match the human output.
- **stdout vs stderr:** report on stdout; the missing-path usage error on stderr
  with **empty stdout**. Machine consumers must be able to read stdout alone.
- **Determinism:** same input → byte-identical stdout (run twice; diff). No
  `HashMap`-order leakage.
- **Scope (DEC-001):** only `lint` — no `convert`/`push`/`profiles`; no `--sarif`/
  `--target`; no new rules; no color. Deps in DEC-008 are permissive; nothing
  beyond clap/serde_json/serde added.

## Return a verdict (final message = data for the orchestrator)

**✅ APPROVED** / **⚠ PUNCH LIST** (numbered, file:line + failing case) /
**❌ REJECTED** (which criterion/decision, concrete command → observed vs
expected). Include gate results, the actual `skillport lint` runs + exit codes you
observed, a per-AC pass/fail summary, and judge DEC-008 (dep choices) + any
deviations. Every flag needs a concrete input→observed/expected. Don't touch code.
