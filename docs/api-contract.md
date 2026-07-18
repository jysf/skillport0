# CLI & Output Contract

skillport has no network API. Its public contract is the **command-line surface**
and the **machine-readable output** (`--json`, later `--sarif`) plus **exit
codes** — the things CI and downstream tooling depend on. All of it is
semver-governed (DEC-005): additive change is MINOR; a breaking change to a flag,
rule id, output field, or exit-code meaning is MAJOR.

## Overview

- **Binary:** `skillport` (single static binary).
- **PROJ-001 command:** `lint`. (`audit` arrives in PROJ-002.)
- **Versioning:** `skillport --version`; `--json` output carries a tool/schema
  marker so consumers can pin it.
- **Streams:** results → **stdout**; diagnostics → **stderr**. Machine consumers
  read stdout only.

## Command: `lint`

```
skillport lint <PATH> [--target <platform>] [--json | --sarif] [--strict]
```

| Argument / flag | Type | Description | Stage |
|---|---|---|---|
| `<PATH>` | path | A single `SKILL.md`, a skill folder, or a tree. A tree is walked into a collection (skips `.git`, `node_modules`, `target`). | STAGE-002 |
| `--target <platform>` | enum | Widen recognized frontmatter fields to a platform's documented set. **`claude`** is verified (STAGE-003); other values are advisory-only until verified (DEC-002). Omitted ⇒ open spec only. | STAGE-003 |
| `--json` | flag | Emit the report as JSON (stable schema). Mutually exclusive with `--sarif`. | STAGE-002 |
| `--sarif` | flag | Emit SARIF 2.1.0 for code-scanning ingestion. | STAGE-003 |
| `--strict` | flag | Treat warnings as failures (affects exit code only, not output). | STAGE-002 |

Default (no `--json`/`--sarif`) is human-readable, path-grouped output with a
severity summary.

## Exit codes

| Code | Meaning |
|---|---|
| `0` | No errors (and, under `--strict`, no warnings). |
| `1` | At least one **error** finding — or any **warning** when `--strict` is set. |
| `2` | Usage error (bad args, unreadable path). |

Info-level findings never affect the exit code. A malformed skill inside a bulk
run produces an error-class finding for that file and does **not** abort the run
(DEC-005).

## `--json` output shape (indicative; finalized in the STAGE-002 emitter spec)

```json
{
  "tool": "skillport",
  "version": "0.1.0",
  "schema": 1,
  "target": null,
  "summary": { "skills": 2, "errors": 1, "warnings": 1, "infos": 0 },
  "sections": [
    {
      "path": "skills/data-analysis/SKILL.md",
      "findings": []
    },
    {
      "path": "skills/My-Skill/SKILL.md",
      "findings": [
        {
          "rule": "name.charset",
          "severity": "error",
          "message": "'name' may only contain lowercase letters, digits, and hyphens (invalid: MS)",
          "field": "name",
          "line": 2
        }
      ]
    }
  ]
}
```

- `sections` is **sorted by path**; findings within a section are deterministically
  ordered (DEC-005).
- `rule` values are the stable ids from the STAGE-002 catalog.
- `line`/`field` are best-effort and may be absent where not cheaply available.

## `--sarif` output (STAGE-003)

SARIF 2.1.0: each rule id becomes a `reportingDescriptor`; each finding a
`result` with `level` mapped from severity (`error`→`error`, `warning`→`warning`,
`info`→`note`) and a `physicalLocation` pointing at the `SKILL.md`. Enables GitHub
code-scanning annotations via the shipped Action.

## Stability / "auth"

No authentication (local CLI). The stability guarantee replaces an auth section:
the CLI flags, rule ids, severity taxonomy, JSON/SARIF schema, and exit codes are
the contract consumers pin. See DEC-005 and `.repo-context.yaml` `version`.

## References

- Rule ids + severities: `projects/PROJ-001-skillport-lint/stages/STAGE-002-*.md`
- Types behind the output: [`./data-model.md`](./data-model.md)
- Pipeline: [`./architecture.md`](./architecture.md)
- Decisions: DEC-002 (verified-only), DEC-003 (severity→exit), DEC-005 (stable schema)
