---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-007
  type: decision
  confidence: 0.85
  audience:
    - developer
    - agent

agent:
  id: claude-opus-4-8
  session_id: null

project:
  id: PROJ-001
repo:
  id: skillport

created_at: 2026-07-17
supersedes: null
superseded_by: null

affected_scope:
  - "Cargo.toml"
  - "Cargo.lock"
  - "src/parse.rs"
  - "src/skill.rs"

tags:
  - dependencies
  - yaml
  - substrate
  - license
---

# DEC-007: Parse frontmatter with `serde_yaml_ng` + `indexmap` (not `serde_yaml`)

## Decision

The tolerant, lossless frontmatter parser (SPEC-001) uses **`serde_yaml_ng`** as
its YAML crate and **`indexmap`** as the order-preserving frontmatter map:

- `Frontmatter` is `indexmap::IndexMap<String, serde_yaml_ng::Value>` — an
  index-map keyed by frontmatter key in **source insertion order** (DEC-004),
  never a `HashMap`.
- `serde_yaml_ng::Value` is kept as the **typed** value (its `Null / Bool /
  Number / String / Sequence / Mapping / Tagged` variants), so rules can
  distinguish a string from a sequence from a mapping (DEC-002) — nothing is
  stringified.
- Deps use **caret ranges** on the current stable toolchain
  (`serde_yaml_ng = "0.10"`, `indexmap = "2"`), not the prototype's
  Rust-1.75-era `=`-exact pins (DEC-005).

## Context

SPEC-001 needs a YAML crate that (a) is maintained, (b) exposes a typed `Value`
with string/sequence/mapping discrimination, and (c) preserves mapping insertion
order — and it must satisfy `no-new-top-level-deps-without-decision`
(runtime dep ⇒ DEC in the same pass) and `license-policy` (permissive only).

`serde_yaml` (dtolnay) is **archived/deprecated** and must not be used
(`toolchain-brief.md`). The maintained options considered were a drop-in
`serde_yaml` fork (`serde_yaml_ng`), the `serde_yml` fork, and lower-level pure
parsers (`saphyr` / `yaml-rust2`).

## Alternatives Considered

- **Option A: `serde_yaml` (dtolnay)**
  - What it is: the de-facto crate the prototype used.
  - Why rejected: archived/unmaintained; explicitly forbidden by the toolchain
    brief and the dependency guidance in the spec.

- **Option B: `serde_yml`**
  - What it is: a hard fork of `serde_yaml`.
  - Why rejected: contested maintenance/provenance and heavy vendored-`unsafe`
    churn; a less predictable long-term footing than a conservative fork.

- **Option C: `saphyr` / `yaml-rust2` (low-level parsers)**
  - What it is: pure-Rust YAML parsers exposing their own event/`Yaml` model.
  - Why rejected: no `serde` `Value` out of the box; we'd hand-roll the typed
    value + de/serialization the substrate wants. More surface for a first spec
    than the payoff; revisit only if `serde_yaml_ng` stalls.

- **Option D (chosen): `serde_yaml_ng` + `indexmap`**
  - What it is: `serde_yaml_ng` is a maintained, minimal-drift continuation of
    `serde_yaml` with the same `Value` API; `indexmap` gives the explicit
    insertion-order map.
  - Why selected: maintained, typed `Value` with full shape discrimination, and
    pairing it with `indexmap` makes order-preservation explicit and independent
    of the YAML crate's internal map — exactly the DEC-004/DEC-002 contract. We
    deserialize the block to `Value`, require a `Mapping` root (else `Invalid`),
    and copy entries into an `IndexMap` in encounter order.

## Consequences

- **Positive:** maintained crate, typed values for STAGE-002 rules, explicit
  order preservation, permissive-licensed, minimal migration cost from the
  prototype's `serde_yaml` shape.
- **Negative:** two deps instead of one; `serde_yaml_ng` is a `0.x` crate, so a
  future minor may need a small port. If it is ever abandoned, Option C is the
  fallback (it would touch only `parse.rs`).
- **Neutral:** `serde` arrives transitively via `serde_yaml_ng` (not a direct
  top-level dep). `cargo-deny` is not wired yet (a later spec) — the chosen deps
  are already in the permissive policy (see below).

## Validation

Right if the parser exposes typed, order-preserved frontmatter that STAGE-002's
`name.type` / `metadata.type` / `allowed-tools.format` rules can inspect without
re-parsing, and `cargo-deny` (once wired) passes with no license exceptions.
Revisit if `serde_yaml_ng` is archived or a rule needs a value shape the crate
cannot represent.

## License compliance (`license-policy`)

All resolved crates are permissive (MIT / Apache-2.0 / BSL-1.0), inside the
allowed set (verified via `cargo metadata`):

- `serde_yaml_ng` — MIT
- `indexmap` — Apache-2.0 OR MIT
- `serde` (transitive) — MIT OR Apache-2.0
- `unsafe-libyaml` (transitive, via `serde_yaml_ng`) — MIT
- `hashbrown`, `equivalent`, `itoa` (transitive) — MIT OR Apache-2.0
- `ryu` (transitive) — Apache-2.0 OR BSL-1.0 (taken under Apache-2.0)

## References

- Related specs: SPEC-001 (tolerant lossless SKILL.md parser)
- Related decisions: DEC-004 (order-preserving/lossless model), DEC-002 (typed
  frontmatter), DEC-005 (current stable toolchain, caret ranges; no `=`-pins)
- Constraints: `no-new-top-level-deps-without-decision`, `license-policy`
- Toolchain: `guidance/toolchain-brief.md` (no `serde_yaml`)
- External: <https://crates.io/crates/serde_yaml_ng>,
  <https://crates.io/crates/indexmap>
