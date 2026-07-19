---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-010
  type: decision
  confidence: 0.85
  audience:
    - developer
    - agent

agent:
  id: claude-sonnet-5
  session_id: null

project:
  id: PROJ-001
repo:
  id: skillport

created_at: 2026-07-18
supersedes: null
superseded_by: null

affected_scope:
  - "Cargo.toml"
  - "src/rules.rs"

tags:
  - dependencies
  - tokenizer
  - license
  - rules
---

# DEC-010: `tiktoken-rs` (`cl100k_base`) as the `body.size` token counter

## Decision

`body.size` (SPEC-010) computes its token count with **`tiktoken-rs`**
(`cl100k_base()`), an embedded, offline BPE tokenizer, added as a direct
runtime dependency (`tiktoken-rs = "0.6"`). The BPE is built **once**, in a
`std::sync::OnceLock<CoreBPE>` (no `once_cell` needed — current stable
suffices), and the body is counted with `encode_ordinary` (content tokens
only; the body is plain Markdown with no special tokens to preserve or
strip). The count is used as a **proxy**, and the rule stays **info**
(DEC-003) — it is explicitly not presented as an exact count for any
specific model.

## Context

`body.size` was deferred from SPEC-006 because the prototype's `chars/4`
approach is a heuristic, and `no-heuristic-error` (plus DEC-003) means a
heuristic can never gate CI at error/warning — but SPEC-010's Frame answer
was "real tokenizer, info-level": readers still want an honest, real token
count, not a chars-based guess, even at info severity. That requires a real
BPE tokenizer dependency, which per `no-new-top-level-deps-without-decision`
needs a DEC authored in the same build pass.

There is no public, standalone Anthropic tokenizer crate to count tokens the
way Claude (or another target model) would. Any real tokenizer used here is
therefore inherently a **proxy** for "how many tokens will this skill
consume" — not a Claude-exact count.

## Alternatives Considered

- **Option A: Keep the `chars/4` heuristic, emit `body.size` as info anyway**
  - What it is: what the prototype did; no new dependency.
  - Why rejected: SPEC-010's whole premise (the Frame-cycle answer) is to
    replace the heuristic with a real tokenizer. `chars/4` is wrong by a
    large and inconsistent margin for real text (see Validation below), and
    the spec's Acceptance Criteria require a test proving the count differs
    from `chars/4` — i.e. this option is explicitly out of scope.

- **Option B: `tokenizers` (Hugging Face)**
  - What it is: a general-purpose, more heavyweight tokenizer library
    supporting arbitrary vocab files (BPE, WordPiece, Unigram, etc.), used by
    most HF model tokenizers.
  - Why rejected: no embedded default vocab for a GPT-style encoding — using
    it would mean vendoring a vocab file ourselves (the same offline-BPE
    problem `tiktoken-rs` already solves) plus a much larger dependency
    surface (targets many tokenizer families this tool doesn't need) for no
    accuracy gain over `tiktoken-rs` as a *proxy*.

- **Option C (chosen): `tiktoken-rs`, `cl100k_base`**
  - What it is: a Rust port of OpenAI's `tiktoken`, with the `cl100k_base`
    (GPT-4/GPT-3.5-era) and `o200k_base` (GPT-4o-era) BPE rank tables
    embedded via `include_str!` at compile time — fully offline, no runtime
    download, no network dependency in CI or at lint time.
  - Why selected: modern BPE token counts are within roughly 10-20% of each
    other across encoders for English prose (the gap comes from vocabulary
    size and merge-rule differences, not fundamentally different
    tokenization strategies), so any modern BPE is a reasonable proxy for
    "roughly how many tokens will this consume" — and the rule is info/
    advisory (DEC-003, `only-verified-constraints-are-firm`), so proxy
    precision is acceptable. `cl100k_base` (rather than the newer
    `o200k_base`) was picked as the more conservative, longest-track-record
    encoding; either satisfies the spec, and swapping is a one-line change
    inside `bpe()` if a maintained Anthropic tokenizer appears later.

## Consequences

- **Positive:** `body.size` now reports a real, deterministic BPE token
  count instead of a heuristic; the crate is offline (embedded ranks, no
  network calls — matches this tool's CI-tool posture); the BPE is built
  once process-wide via `OnceLock`, so linting many skills doesn't re-parse
  the ~1.7 MB rank table per file.
- **Negative:** the embedded `cl100k_base.tiktoken` asset is ~1.7 MB
  (`include_str!`'d into the binary), growing the compiled binary by
  roughly 1-2 MB — acceptable for a CI lint tool, noted here per the spec.
  One more direct dependency (plus its small transitive tree: `anyhow`,
  `base64`, `bstr`, `fancy-regex`, `lazy_static`, `parking_lot`, `regex`,
  `rustc-hash`, and their sub-deps) to track for licensing/security.
- **Neutral:** the count is explicitly a *proxy*, not an exact count for any
  specific model; the rule message uses "~" and stays info-only so a
  divergence from any real model's tokenizer is harmless (never a false CI
  failure).

## Validation

Right if `body.size` never gates CI (stays info) and its count is
demonstrably a real tokenizer's output, not a chars-based guess — verified
by a pinned test (`body_token_count("tokenization")` == 2, cl100k_base's
actual output, vs. `chars/4` == 3 for the same string; `src/rules.rs` mod
tests). Revisit if a maintained Anthropic tokenizer crate is published
(swap the encoder inside `bpe()`), or if binary size becomes a real
distribution constraint (unlikely for a CI tool; see DEC-009).

## License compliance (`license-policy`)

Verified via `cargo metadata` (default feature set only — `tiktoken-rs`'s
optional `async-openai`/`dhat` features are not enabled) and confirmed with
`cargo deny check licenses` (`licenses ok`, no violations, no exceptions
needed):

- `tiktoken-rs` 0.6.0 — MIT
- `anyhow` 1.0.104 — MIT OR Apache-2.0
- `base64` 0.21.7 — MIT OR Apache-2.0
- `bstr` 1.13.0 — MIT OR Apache-2.0
- `fancy-regex` 0.13.0 — MIT
- `lazy_static` 1.5.0 — MIT OR Apache-2.0
- `parking_lot` 0.12.5, `parking_lot_core` 0.9.12, `lock_api` 0.4.14,
  `scopeguard` 1.2.0 — MIT OR Apache-2.0
- `regex` 1.13.1, `regex-automata` 0.4.16, `regex-syntax` 0.8.11,
  `aho-corasick` 1.1.4 (Unlicense OR MIT), `memchr` 2.8.3
  (Unlicense OR MIT) — MIT OR Apache-2.0 / Unlicense-or-MIT
- `rustc-hash` 1.1.0 — Apache-2.0/MIT
- `bit-set` 0.5.3, `bit-vec` 0.6.3 — MIT/Apache-2.0
- `cfg-if` 1.0.4, `libc` 0.2.186, `smallvec` 1.15.2 — MIT OR Apache-2.0

No copyleft dependency introduced. All licenses fall inside `deny.toml`'s
existing `allow` list (MIT, Apache-2.0, and the Unlicense-dual-licensed
crates resolve as MIT); no new `deny.toml` entries were needed.

## References

- Related specs: SPEC-006 (`check_body`: `body.empty`, `body.lines`), SPEC-010
  (`body.size`, this decision)
- Related decisions: DEC-002 (open-spec-authoritative — the ~5000-token
  guidance), DEC-003 (severity discipline — info, never error/warning),
  DEC-005 (deterministic output — BPE built once), DEC-007/DEC-008 (prior
  permissive-dependency-with-DEC precedent)
- Constraints: `no-new-top-level-deps-without-decision`, `license-policy`,
  `no-heuristic-error`, `deterministic-stable-output`
- External: <https://crates.io/crates/tiktoken-rs>,
  <https://github.com/openai/tiktoken>
