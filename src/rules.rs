//! The open-spec rule engine (STAGE-002, spec 1 of it).
//!
//! `lint_skill` is the `rule_fn` seam `Report::from_collection` expects
//! (SPEC-003): given one parsed [`Skill`], run every implemented open-spec
//! rule and return its findings, unordered — the report layer sorts them.
//!
//! SPEC-004 covered frontmatter presence and the `name.*` / `description.*` /
//! `compatibility.length` rules (the crisp, high-value identity/description
//! batch). SPEC-006 completes the open-spec layer: `metadata.*`,
//! `allowed-tools.*`, `body.empty`/`body.lines`, `frontmatter.unknown`, and
//! `compatibility.type`; it also tightens `name.charset` to strict ASCII.
//! SPEC-010 completes the open-spec catalog with `body.size` (a real BPE
//! tokenizer, DEC-010). Only `--target` widening remains, for STAGE-003.
//!
//! Two locked design decisions (see the spec's "design decisions" section):
//!
//! 1. `FrontmatterStatus::Missing`/`Unclosed`/`Invalid` each surface as their
//!    own stable error id (`frontmatter.missing`/`frontmatter.unclosed`/
//!    `frontmatter.invalid`) and then we RETURN — field rules never run
//!    against an empty/absent map.
//! 2. `FrontmatterStatus::Present` but an empty map does NOT emit
//!    `frontmatter.missing` — a block *is* present, so `name.required` and
//!    `description.required` fire instead (clearer, more actionable).
//!
//! Reference: `initial_stuff/lint.rs` (ported, not copied — adapted to
//! `Finding`/`FrontmatterStatus` and the exact catalog severities here).

use crate::report::{Finding, Severity};
use crate::skill::{FrontmatterStatus, Skill};
use std::sync::OnceLock;
use tiktoken_rs::CoreBPE;

/// The `description.detail` terseness threshold (soft, tunable; info-only so
/// a false positive is harmless). Ported from the prototype's `< 40` chars.
const DESCRIPTION_DETAIL_THRESHOLD: usize = 40;

/// `body.lines` recommended ceiling (open spec: move detail into references/).
const BODY_LINES_THRESHOLD: usize = 500;

/// `body.size` recommended ceiling (open spec: ~5000 tokens; move detail into
/// references/). Tunable; `>` comparison; info-only (DEC-003) since the count
/// is a proxy (DEC-010), not an exact count for any specific model.
const BODY_TOKENS_THRESHOLD: usize = 5000;

/// The process-wide BPE, built once (not per skill) — `deterministic-stable-
/// output` + DEC-010. `cl100k_base` is a proxy tokenizer (no public Anthropic
/// tokenizer exists); see DEC-010 for the rationale.
static BPE: OnceLock<CoreBPE> = OnceLock::new();

fn bpe() -> &'static CoreBPE {
    BPE.get_or_init(|| {
        tiktoken_rs::cl100k_base().expect("cl100k_base ranks are embedded at compile time")
    })
}

/// Count `text`'s tokens with the real (proxy) BPE tokenizer — NOT a
/// chars/words heuristic. Counts ordinary/content tokens only
/// (`encode_ordinary`; no special-token handling needed for a Markdown
/// body). Deterministic: same input -> same count, every run (DEC-005).
fn body_token_count(text: &str) -> usize {
    bpe().encode_ordinary(text).len()
}

/// Frontmatter fields defined by the open spec (`SPEC_KEYS` in the prototype).
/// `--target` widening of this set is STAGE-003.
const SPEC_KEYS: &[&str] = &[
    "name",
    "description",
    "license",
    "compatibility",
    "metadata",
    "allowed-tools",
];

/// A lint target: a specific agent platform whose recognized-field/behavior
/// facts have been verified from that platform's primary docs (DEC-002). Only
/// `Claude` is verified so far — no Cursor/Codex/Vercel variant exists yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Claude,
}

// source: code.claude.com/docs/en/skills
//
// Claude Code's recognized SKILL.md frontmatter fields, beyond the open-spec
// `SPEC_KEYS` ("Extend Claude with skills", Frontmatter reference; verified
// 2026-07-18). Only fields NOT already in `SPEC_KEYS` are listed here.
const CLAUDE_KEYS: &[&str] = &[
    "disable-model-invocation",
    "user-invocable",
    "disallowed-tools",
    "model",
    "effort",
    "context",
    "hooks",
    "arguments",
    // SPEC-012: the 5 fields below close the enumeration gap SPEC-011's
    // verify flagged (each independently doc-verified 2026-07-18 against the
    // same Frontmatter reference table as the 8 above).
    // source: code.claude.com/docs/en/skills
    "when_to_use",
    // source: code.claude.com/docs/en/skills
    "argument-hint",
    // source: code.claude.com/docs/en/skills
    "agent",
    // source: code.claude.com/docs/en/skills
    "paths",
    // source: code.claude.com/docs/en/skills
    "shell",
];

/// One documented entry in the rule catalog: a stable id, its **default**
/// emitted severity, and a one-line human summary. `structural` marks the 2
/// ids `report.rs` emits for unreadable files/dirs rather than a field rule
/// here — the fixture-coverage test excuses them (they need a non-UTF-8 file
/// / a permission-denied directory, already covered by `report.rs` unit
/// tests, not a `SKILL.md` fixture).
///
/// `allowed-tools.format`'s `severity` here is its **default** (`Warning`,
/// i.e. no `--target`); the `--target claude` downgrade to `Info` is real
/// runtime behavior (see `check_allowed_tools`) but is documented as a note,
/// not a second catalog entry — SPEC-012, "Notes for the Implementer".
#[derive(Debug, Clone, Copy)]
pub struct RuleDoc {
    pub id: &'static str,
    pub severity: Severity,
    pub summary: &'static str,
    pub structural: bool,
}

/// SPEC-012: the single, ordered, duplicate-free source of truth for every
/// rule id the engine (this module) or the structural layer (`report.rs`)
/// can emit — 24 engine ids + 2 structural ids = 26. The README's
/// `## Rule reference` table and the `catalog_is_locked` /
/// `no_orphan_rule_ids` / `every_engine_rule_has_a_fixture` /
/// `readme_rule_table_matches_catalog` tests all derive from this list
/// (DEC-005: rule ids are a public contract — adding, removing, or renaming
/// one must touch this const and its lock test).
pub const RULES: &[RuleDoc] = &[
    RuleDoc {
        id: "frontmatter.missing",
        severity: Severity::Error,
        summary: "no YAML frontmatter block",
        structural: false,
    },
    RuleDoc {
        id: "frontmatter.unclosed",
        severity: Severity::Error,
        summary: "opening '---' but no closing '---'",
        structural: false,
    },
    RuleDoc {
        id: "frontmatter.invalid",
        severity: Severity::Error,
        summary: "frontmatter is not a valid YAML mapping",
        structural: false,
    },
    RuleDoc {
        id: "frontmatter.unknown",
        severity: Severity::Info,
        summary: "a key isn't recognized (open set; --target claude also allows Claude's fields)",
        structural: false,
    },
    RuleDoc {
        id: "name.required",
        severity: Severity::Error,
        summary: "'name' is missing",
        structural: false,
    },
    RuleDoc {
        id: "name.type",
        severity: Severity::Error,
        summary: "'name' is not a string",
        structural: false,
    },
    RuleDoc {
        id: "name.length",
        severity: Severity::Error,
        summary: "'name' is not 1-64 characters",
        structural: false,
    },
    RuleDoc {
        id: "name.charset",
        severity: Severity::Error,
        summary: "'name' has characters outside [a-z0-9-] (strict ASCII)",
        structural: false,
    },
    RuleDoc {
        id: "name.hyphen-edges",
        severity: Severity::Error,
        summary: "'name' starts or ends with '-'",
        structural: false,
    },
    RuleDoc {
        id: "name.hyphen-consecutive",
        severity: Severity::Error,
        summary: "'name' contains '--'",
        structural: false,
    },
    RuleDoc {
        id: "name.dir-match",
        severity: Severity::Warning,
        summary: "'name' does not match the skill's directory name",
        structural: false,
    },
    RuleDoc {
        id: "description.required",
        severity: Severity::Error,
        summary: "'description' is missing",
        structural: false,
    },
    RuleDoc {
        id: "description.type",
        severity: Severity::Error,
        summary: "'description' is not a string",
        structural: false,
    },
    RuleDoc {
        id: "description.length",
        severity: Severity::Error,
        summary: "'description' is empty or over 1024 characters",
        structural: false,
    },
    RuleDoc {
        id: "description.detail",
        severity: Severity::Info,
        summary: "'description' is under 40 characters (state what + when)",
        structural: false,
    },
    RuleDoc {
        id: "compatibility.length",
        severity: Severity::Error,
        summary: "'compatibility' is over 500 characters",
        structural: false,
    },
    RuleDoc {
        id: "compatibility.type",
        severity: Severity::Warning,
        summary: "'compatibility' is not a string",
        structural: false,
    },
    RuleDoc {
        id: "metadata.type",
        severity: Severity::Warning,
        summary: "'metadata' is not a key-value map",
        structural: false,
    },
    RuleDoc {
        id: "metadata.values",
        severity: Severity::Info,
        summary: "a 'metadata' value is not a string",
        structural: false,
    },
    RuleDoc {
        id: "allowed-tools.format",
        severity: Severity::Warning,
        summary: "'allowed-tools' is given as a YAML list (default severity; downgrades to Info under --target claude)",
        structural: false,
    },
    RuleDoc {
        id: "allowed-tools.type",
        severity: Severity::Warning,
        summary: "'allowed-tools' is neither a string nor a list",
        structural: false,
    },
    RuleDoc {
        id: "body.empty",
        severity: Severity::Warning,
        summary: "the SKILL.md body is blank",
        structural: false,
    },
    RuleDoc {
        id: "body.lines",
        severity: Severity::Warning,
        summary: "body is over 500 lines",
        structural: false,
    },
    RuleDoc {
        id: "body.size",
        severity: Severity::Info,
        summary: "body is over ~5000 tokens (real cl100k_base tokenizer)",
        structural: false,
    },
    RuleDoc {
        id: crate::report::FILE_UNREADABLE,
        severity: Severity::Error,
        summary: "a SKILL.md couldn't be read (e.g. non-UTF-8) — structural",
        structural: true,
    },
    RuleDoc {
        id: crate::report::DIR_UNREADABLE,
        severity: Severity::Warning,
        summary: "a directory in the tree couldn't be read — structural",
        structural: true,
    },
];

/// The 26 rule ids in catalog order, for callers that just want the id set
/// (e.g. the drift/coverage tests).
pub fn all_rule_ids() -> Vec<&'static str> {
    RULES.iter().map(|r| r.id).collect()
}

/// Run every implemented open-spec rule over `skill` and return its findings.
/// Unordered — `Report::from_collection` sorts deterministically. Equivalent
/// to `lint_skill_with_target(skill, None)`: behavior is the open spec only,
/// unchanged by `--target` widening.
pub fn lint_skill(skill: &Skill) -> Vec<Finding> {
    lint_skill_with_target(skill, None)
}

/// Run every implemented rule over `skill` and return its findings, widened
/// by `target` (STAGE-003, DEC-002): with `Some(Target::Claude)`,
/// `frontmatter.unknown` also recognizes `CLAUDE_KEYS` and `allowed-tools.format`
/// (list case) downgrades to `Info`. Every other rule is unchanged by `target`.
pub fn lint_skill_with_target(skill: &Skill, target: Option<Target>) -> Vec<Finding> {
    let mut findings = Vec::new();

    match &skill.frontmatter_status {
        FrontmatterStatus::Missing => {
            push(
                &mut findings,
                "frontmatter.missing",
                Severity::Error,
                "no YAML frontmatter found; 'name' and 'description' are required",
                skill,
                None,
            );
            return findings;
        }
        FrontmatterStatus::Unclosed => {
            push(
                &mut findings,
                "frontmatter.unclosed",
                Severity::Error,
                "frontmatter has an opening fence ('---') but no closing fence",
                skill,
                None,
            );
            return findings;
        }
        FrontmatterStatus::Invalid(reason) => {
            push(
                &mut findings,
                "frontmatter.invalid",
                Severity::Error,
                format!("frontmatter is not a valid YAML mapping: {reason}"),
                skill,
                None,
            );
            return findings;
        }
        FrontmatterStatus::Present => {}
    }

    check_name(skill, &mut findings);
    check_description(skill, &mut findings);
    check_compatibility(skill, &mut findings);
    check_metadata(skill, &mut findings);
    check_allowed_tools(skill, &mut findings, target);
    check_body(skill, &mut findings);
    check_unknown_fields(skill, &mut findings, target);

    findings
}

/// Push one finding onto `findings`, filling `path` from `skill` uniformly.
fn push(
    findings: &mut Vec<Finding>,
    rule: &'static str,
    severity: Severity,
    message: impl Into<String>,
    skill: &Skill,
    field: Option<&str>,
) {
    findings.push(Finding {
        rule,
        severity,
        message: message.into(),
        path: skill.path.clone(),
        field: field.map(str::to_string),
        line: None,
    });
}

fn check_name(skill: &Skill, findings: &mut Vec<Finding>) {
    let value = match skill.get("name") {
        None => {
            push(
                findings,
                "name.required",
                Severity::Error,
                "'name' is required",
                skill,
                Some("name"),
            );
            return;
        }
        Some(v) => v,
    };

    let name = match value.as_str() {
        Some(s) => s,
        None => {
            push(
                findings,
                "name.type",
                Severity::Error,
                "'name' must be a string",
                skill,
                Some("name"),
            );
            return;
        }
    };

    let len = name.chars().count();
    if len == 0 || len > 64 {
        push(
            findings,
            "name.length",
            Severity::Error,
            format!("'name' must be 1-64 characters (got {len})"),
            skill,
            Some("name"),
        );
    }

    // Strict ASCII: lowercase letters, digits, and hyphens only (SPEC-006,
    // signal `name-charset-ascii`). `name` is a kebab-case identifier that
    // must map to a directory name and be portable, so non-ASCII letters/
    // digits (e.g. `café`, Arabic-Indic digits) are rejected too, not just
    // uppercase ASCII.
    let invalid: String = name
        .chars()
        .filter(|c| !(c.is_ascii_lowercase() || c.is_ascii_digit() || *c == '-'))
        .collect();
    if !invalid.is_empty() {
        push(
            findings,
            "name.charset",
            Severity::Error,
            format!(
                "'name' may only contain lowercase letters, digits, and hyphens (invalid: {invalid})"
            ),
            skill,
            Some("name"),
        );
    }

    if name.starts_with('-') || name.ends_with('-') {
        push(
            findings,
            "name.hyphen-edges",
            Severity::Error,
            "'name' must not start or end with a hyphen",
            skill,
            Some("name"),
        );
    }

    if name.contains("--") {
        push(
            findings,
            "name.hyphen-consecutive",
            Severity::Error,
            "'name' must not contain consecutive hyphens",
            skill,
            Some("name"),
        );
    }

    if let Some(dir) = &skill.dir_name {
        if name != dir {
            push(
                findings,
                "name.dir-match",
                Severity::Warning,
                format!("'name' ({name}) should match the skill directory name ({dir})"),
                skill,
                Some("name"),
            );
        }
    }
}

fn check_description(skill: &Skill, findings: &mut Vec<Finding>) {
    let value = match skill.get("description") {
        None => {
            push(
                findings,
                "description.required",
                Severity::Error,
                "'description' is required",
                skill,
                Some("description"),
            );
            return;
        }
        Some(v) => v,
    };

    let description = match value.as_str() {
        Some(s) => s,
        None => {
            push(
                findings,
                "description.type",
                Severity::Error,
                "'description' must be a string",
                skill,
                Some("description"),
            );
            return;
        }
    };

    let len = description.chars().count();
    if len == 0 {
        push(
            findings,
            "description.length",
            Severity::Error,
            "'description' must not be empty",
            skill,
            Some("description"),
        );
    } else if len > 1024 {
        push(
            findings,
            "description.length",
            Severity::Error,
            format!("'description' must be at most 1024 characters (got {len})"),
            skill,
            Some("description"),
        );
    } else if len < DESCRIPTION_DETAIL_THRESHOLD {
        push(
            findings,
            "description.detail",
            Severity::Info,
            "'description' is short; the spec recommends stating both what the skill does and when to use it",
            skill,
            Some("description"),
        );
    }
}

fn check_compatibility(skill: &Skill, findings: &mut Vec<Finding>) {
    if let Some(value) = skill.get("compatibility") {
        match value.as_str() {
            Some(s) => {
                let len = s.chars().count();
                if len > 500 {
                    push(
                        findings,
                        "compatibility.length",
                        Severity::Error,
                        format!("'compatibility' must be at most 500 characters (got {len})"),
                        skill,
                        Some("compatibility"),
                    );
                }
            }
            None => {
                push(
                    findings,
                    "compatibility.type",
                    Severity::Warning,
                    "'compatibility' should be a string",
                    skill,
                    Some("compatibility"),
                );
            }
        }
    }
}

fn check_metadata(skill: &Skill, findings: &mut Vec<Finding>) {
    if let Some(value) = skill.get("metadata") {
        match value.as_mapping() {
            None => {
                push(
                    findings,
                    "metadata.type",
                    Severity::Warning,
                    "'metadata' should be a key-value map",
                    skill,
                    Some("metadata"),
                );
            }
            Some(map) => {
                for (k, v) in map {
                    if !v.is_string() {
                        let key = k.as_str().unwrap_or("?");
                        push(
                            findings,
                            "metadata.values",
                            Severity::Info,
                            format!(
                                "metadata.{key} is not a string; the spec defines metadata as string-to-string (quote values like \"1.0\")"
                            ),
                            skill,
                            Some(&format!("metadata.{key}")),
                        );
                    }
                }
            }
        }
    }
}

fn check_allowed_tools(skill: &Skill, findings: &mut Vec<Finding>, target: Option<Target>) {
    if let Some(value) = skill.get("allowed-tools") {
        if value.is_sequence() {
            // Open spec defines `allowed-tools` as a space-separated string,
            // not a list. Under `--target claude`, this downgrades to Info:
            // source: code.claude.com/docs/en/skills — "Accepts a space- or
            // comma-separated string, or a YAML list."
            if target == Some(Target::Claude) {
                push(
                    findings,
                    "allowed-tools.format",
                    Severity::Info,
                    "'allowed-tools' is a list; the open spec expects a space-separated string, but Claude Code accepts a list (source: code.claude.com/docs/en/skills)",
                    skill,
                    Some("allowed-tools"),
                );
            } else {
                push(
                    findings,
                    "allowed-tools.format",
                    Severity::Warning,
                    "the open spec defines 'allowed-tools' as a space-separated string, not a list",
                    skill,
                    Some("allowed-tools"),
                );
            }
        } else if !value.is_string() {
            push(
                findings,
                "allowed-tools.type",
                Severity::Warning,
                "'allowed-tools' should be a space-separated string",
                skill,
                Some("allowed-tools"),
            );
        }
    }
}

fn check_body(skill: &Skill, findings: &mut Vec<Finding>) {
    if skill.body.trim().is_empty() {
        push(
            findings,
            "body.empty",
            Severity::Warning,
            "the SKILL.md body is empty; add instructions for the agent",
            skill,
            None,
        );
        return;
    }

    let lines = skill.body.lines().count();
    if lines > BODY_LINES_THRESHOLD {
        push(
            findings,
            "body.lines",
            Severity::Warning,
            format!(
                "body is {lines} lines; the spec recommends keeping SKILL.md under {BODY_LINES_THRESHOLD} (move detail into references/)"
            ),
            skill,
            None,
        );
    }

    let tokens = body_token_count(&skill.body);
    if tokens > BODY_TOKENS_THRESHOLD {
        push(
            findings,
            "body.size",
            Severity::Info,
            format!(
                "body is ~{tokens} tokens; the spec recommends under {BODY_TOKENS_THRESHOLD} — use progressive disclosure (move detail into references/)"
            ),
            skill,
            None,
        );
    }
}

fn check_unknown_fields(skill: &Skill, findings: &mut Vec<Finding>, target: Option<Target>) {
    // Order-preserving iteration (the frontmatter is an `IndexMap`, not a
    // `HashMap`) for deterministic output, per constraint
    // `deterministic-stable-output`.
    for key in skill.keys() {
        let recognized = SPEC_KEYS.iter().any(|k| k == key)
            || (target == Some(Target::Claude) && CLAUDE_KEYS.iter().any(|k| k == key));
        if !recognized {
            push(
                findings,
                "frontmatter.unknown",
                Severity::Info,
                format!("'{key}' is not a recognized field; compliant agents ignore unknown keys"),
                skill,
                Some(key),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skill::{Frontmatter, YamlValue};
    use crate::walk::walk;
    use crate::Report;
    use std::path::{Path, PathBuf};

    /// Build a minimal in-memory `Skill` for rule tests, without touching
    /// the filesystem or the real parser.
    fn make_skill(
        frontmatter: Frontmatter,
        status: FrontmatterStatus,
        dir_name: Option<&str>,
    ) -> Skill {
        Skill {
            path: PathBuf::from("test/SKILL.md"),
            dir_name: dir_name.map(str::to_string),
            frontmatter,
            body: String::from("body"),
            raw: String::new(),
            frontmatter_status: status,
        }
    }

    /// A frontmatter map with just `name`/`description` set to valid values.
    fn valid_frontmatter() -> Frontmatter {
        let mut fm = Frontmatter::new();
        fm.insert(
            "name".to_string(),
            YamlValue::String("valid-skill".to_string()),
        );
        fm.insert(
            "description".to_string(),
            YamlValue::String(
                "Do the thing well. Use this when you need the thing done properly.".to_string(),
            ),
        );
        fm
    }

    fn str_val(s: &str) -> YamlValue {
        YamlValue::String(s.to_string())
    }

    /// The set of (rule, severity) pairs a `Vec<Finding>` produced, for
    /// order-independent assertions.
    fn rule_severities(findings: &[Finding]) -> Vec<(&'static str, Severity)> {
        let mut pairs: Vec<(&'static str, Severity)> =
            findings.iter().map(|f| (f.rule, f.severity)).collect();
        pairs.sort_by_key(|(rule, _)| *rule);
        pairs
    }

    fn has_rule(findings: &[Finding], rule: &str) -> bool {
        findings.iter().any(|f| f.rule == rule)
    }

    #[test]
    fn frontmatter_missing_yields_frontmatter_missing_error_only() {
        let skill = make_skill(Frontmatter::new(), FrontmatterStatus::Missing, None);

        let findings = lint_skill(&skill);

        assert_eq!(
            rule_severities(&findings),
            vec![("frontmatter.missing", Severity::Error)]
        );
    }

    #[test]
    fn frontmatter_unclosed_yields_frontmatter_unclosed() {
        let skill = make_skill(Frontmatter::new(), FrontmatterStatus::Unclosed, None);

        let findings = lint_skill(&skill);

        assert_eq!(
            rule_severities(&findings),
            vec![("frontmatter.unclosed", Severity::Error)]
        );
    }

    #[test]
    fn frontmatter_invalid_yields_frontmatter_invalid() {
        let skill = make_skill(
            Frontmatter::new(),
            FrontmatterStatus::Invalid("root is a sequence, not a mapping".to_string()),
            None,
        );

        let findings = lint_skill(&skill);

        assert_eq!(
            rule_severities(&findings),
            vec![("frontmatter.invalid", Severity::Error)]
        );
    }

    #[test]
    fn empty_present_frontmatter_requires_name_and_description_not_missing() {
        let skill = make_skill(Frontmatter::new(), FrontmatterStatus::Present, None);

        let findings = lint_skill(&skill);

        assert!(!has_rule(&findings, "frontmatter.missing"));
        assert!(has_rule(&findings, "name.required"));
        assert!(has_rule(&findings, "description.required"));
    }

    #[test]
    fn name_required_when_absent() {
        let mut fm = valid_frontmatter();
        fm.shift_remove("name");
        let skill = make_skill(fm, FrontmatterStatus::Present, None);

        let findings = lint_skill(&skill);

        assert!(has_rule(&findings, "name.required"));
        assert!(!has_rule(&findings, "name.type"));
    }

    #[test]
    fn name_type_when_non_string() {
        let mut fm = valid_frontmatter();
        fm.insert("name".to_string(), YamlValue::Number(1.into()));
        let skill = make_skill(fm, FrontmatterStatus::Present, None);

        let findings = lint_skill(&skill);

        assert!(has_rule(&findings, "name.type"));
        // Type check returns early; no length/charset piled on.
        assert!(!has_rule(&findings, "name.length"));
    }

    #[test]
    fn name_length_zero_is_error() {
        let mut fm = valid_frontmatter();
        fm.insert("name".to_string(), str_val(""));
        let skill = make_skill(fm, FrontmatterStatus::Present, None);

        let findings = lint_skill(&skill);

        assert!(has_rule(&findings, "name.length"));
    }

    #[test]
    fn name_length_65_is_error() {
        let mut fm = valid_frontmatter();
        fm.insert("name".to_string(), str_val(&"a".repeat(65)));
        let skill = make_skill(fm, FrontmatterStatus::Present, None);

        let findings = lint_skill(&skill);

        assert!(has_rule(&findings, "name.length"));
    }

    #[test]
    fn name_length_64_is_ok() {
        let mut fm = valid_frontmatter();
        fm.insert("name".to_string(), str_val(&"a".repeat(64)));
        let skill = make_skill(fm, FrontmatterStatus::Present, None);

        let findings = lint_skill(&skill);

        assert!(!has_rule(&findings, "name.length"));
    }

    #[test]
    fn name_charset_uppercase_is_error() {
        let mut fm = valid_frontmatter();
        fm.insert("name".to_string(), str_val("Valid-Skill"));
        let skill = make_skill(fm, FrontmatterStatus::Present, None);

        let findings = lint_skill(&skill);

        assert!(has_rule(&findings, "name.charset"));
    }

    #[test]
    fn name_charset_space_is_error() {
        let mut fm = valid_frontmatter();
        fm.insert("name".to_string(), str_val("valid skill"));
        let skill = make_skill(fm, FrontmatterStatus::Present, None);

        let findings = lint_skill(&skill);

        assert!(has_rule(&findings, "name.charset"));
    }

    #[test]
    fn name_charset_lowercase_digits_hyphen_is_ok() {
        let mut fm = valid_frontmatter();
        fm.insert("name".to_string(), str_val("valid-skill-123"));
        let skill = make_skill(fm, FrontmatterStatus::Present, None);

        let findings = lint_skill(&skill);

        assert!(!has_rule(&findings, "name.charset"));
    }

    #[test]
    fn name_charset_rejects_non_ascii_letters() {
        let mut fm = valid_frontmatter();
        fm.insert("name".to_string(), str_val("café-skill"));
        let skill = make_skill(fm, FrontmatterStatus::Present, None);

        let findings = lint_skill(&skill);

        let finding = findings.iter().find(|f| f.rule == "name.charset");
        assert_eq!(finding.map(|f| f.severity), Some(Severity::Error));
    }

    #[test]
    fn name_charset_rejects_non_ascii_digit() {
        let mut fm = valid_frontmatter();
        // Arabic-Indic digit ٣ ("3"), not accepted even though
        // `char::is_numeric()` would accept it.
        fm.insert("name".to_string(), str_val("skill-٣"));
        let skill = make_skill(fm, FrontmatterStatus::Present, None);

        let findings = lint_skill(&skill);

        let finding = findings.iter().find(|f| f.rule == "name.charset");
        assert_eq!(finding.map(|f| f.severity), Some(Severity::Error));
    }

    #[test]
    fn name_hyphen_edges_leading_and_trailing_is_error() {
        let mut fm = valid_frontmatter();
        fm.insert("name".to_string(), str_val("-leading"));
        let leading = make_skill(fm, FrontmatterStatus::Present, None);
        assert!(has_rule(&lint_skill(&leading), "name.hyphen-edges"));

        let mut fm = valid_frontmatter();
        fm.insert("name".to_string(), str_val("trailing-"));
        let trailing = make_skill(fm, FrontmatterStatus::Present, None);
        assert!(has_rule(&lint_skill(&trailing), "name.hyphen-edges"));
    }

    #[test]
    fn name_hyphen_consecutive_is_error() {
        let mut fm = valid_frontmatter();
        fm.insert("name".to_string(), str_val("double--hyphen"));
        let skill = make_skill(fm, FrontmatterStatus::Present, None);

        let findings = lint_skill(&skill);

        assert!(has_rule(&findings, "name.hyphen-consecutive"));
    }

    #[test]
    fn name_dir_match_mismatch_is_warning() {
        let mut fm = valid_frontmatter();
        fm.insert("name".to_string(), str_val("valid-skill"));
        let skill = make_skill(fm, FrontmatterStatus::Present, Some("other-dir"));

        let findings = lint_skill(&skill);

        let dir_match = findings.iter().find(|f| f.rule == "name.dir-match");
        assert_eq!(dir_match.map(|f| f.severity), Some(Severity::Warning));
    }

    #[test]
    fn name_dir_match_equal_is_none() {
        let mut fm = valid_frontmatter();
        fm.insert("name".to_string(), str_val("valid-skill"));
        let skill = make_skill(fm, FrontmatterStatus::Present, Some("valid-skill"));

        let findings = lint_skill(&skill);

        assert!(!has_rule(&findings, "name.dir-match"));
    }

    #[test]
    fn name_dir_match_skipped_when_dir_name_none() {
        let fm = valid_frontmatter();
        let skill = make_skill(fm, FrontmatterStatus::Present, None);

        let findings = lint_skill(&skill);

        assert!(!has_rule(&findings, "name.dir-match"));
    }

    #[test]
    fn description_required_type_empty_too_long_are_errors() {
        // required
        let mut fm = valid_frontmatter();
        fm.shift_remove("description");
        assert!(has_rule(
            &lint_skill(&make_skill(fm, FrontmatterStatus::Present, None)),
            "description.required"
        ));

        // type
        let mut fm = valid_frontmatter();
        fm.insert("description".to_string(), YamlValue::Number(1.into()));
        assert!(has_rule(
            &lint_skill(&make_skill(fm, FrontmatterStatus::Present, None)),
            "description.type"
        ));

        // empty
        let mut fm = valid_frontmatter();
        fm.insert("description".to_string(), str_val(""));
        assert!(has_rule(
            &lint_skill(&make_skill(fm, FrontmatterStatus::Present, None)),
            "description.length"
        ));

        // too long
        let mut fm = valid_frontmatter();
        fm.insert("description".to_string(), str_val(&"a".repeat(1025)));
        assert!(has_rule(
            &lint_skill(&make_skill(fm, FrontmatterStatus::Present, None)),
            "description.length"
        ));
    }

    #[test]
    fn description_detail_short_is_info_good_description_has_none() {
        let mut fm = valid_frontmatter();
        fm.insert("description".to_string(), str_val("short desc"));
        let skill = make_skill(fm, FrontmatterStatus::Present, None);
        let findings = lint_skill(&skill);
        let detail = findings.iter().find(|f| f.rule == "description.detail");
        assert_eq!(detail.map(|f| f.severity), Some(Severity::Info));

        let fm = valid_frontmatter(); // long enough description
        let skill = make_skill(fm, FrontmatterStatus::Present, None);
        let findings = lint_skill(&skill);
        assert!(!has_rule(&findings, "description.detail"));
    }

    #[test]
    fn compatibility_length_over_500_is_error_under_is_none_absent_is_none() {
        let mut fm = valid_frontmatter();
        fm.insert("compatibility".to_string(), str_val(&"a".repeat(501)));
        let skill = make_skill(fm, FrontmatterStatus::Present, None);
        assert!(has_rule(&lint_skill(&skill), "compatibility.length"));

        let mut fm = valid_frontmatter();
        fm.insert("compatibility".to_string(), str_val(&"a".repeat(500)));
        let skill = make_skill(fm, FrontmatterStatus::Present, None);
        assert!(!has_rule(&lint_skill(&skill), "compatibility.length"));

        let fm = valid_frontmatter(); // no compatibility key at all
        let skill = make_skill(fm, FrontmatterStatus::Present, None);
        assert!(!has_rule(&lint_skill(&skill), "compatibility.length"));
    }

    #[test]
    fn metadata_non_mapping_is_metadata_type_warning() {
        let mut fm = valid_frontmatter();
        fm.insert("metadata".to_string(), str_val("not-a-map"));
        let skill = make_skill(fm, FrontmatterStatus::Present, None);

        let findings = lint_skill(&skill);

        let finding = findings.iter().find(|f| f.rule == "metadata.type");
        assert_eq!(finding.map(|f| f.severity), Some(Severity::Warning));
    }

    #[test]
    fn metadata_string_value_is_ok() {
        let mut fm = valid_frontmatter();
        let mut map = serde_yaml_ng::Mapping::new();
        map.insert(str_val("version"), str_val("1.0"));
        fm.insert("metadata".to_string(), YamlValue::Mapping(map));
        let skill = make_skill(fm, FrontmatterStatus::Present, None);

        let findings = lint_skill(&skill);

        assert!(!has_rule(&findings, "metadata.type"));
        assert!(!has_rule(&findings, "metadata.values"));
    }

    #[test]
    fn metadata_non_string_value_is_metadata_values_info() {
        let mut fm = valid_frontmatter();
        let mut map = serde_yaml_ng::Mapping::new();
        map.insert(str_val("version"), YamlValue::Number(1.0.into()));
        fm.insert("metadata".to_string(), YamlValue::Mapping(map));
        let skill = make_skill(fm, FrontmatterStatus::Present, None);

        let findings = lint_skill(&skill);

        let finding = findings.iter().find(|f| f.rule == "metadata.values");
        assert_eq!(finding.map(|f| f.severity), Some(Severity::Info));
    }

    #[test]
    fn metadata_absent_is_none() {
        let fm = valid_frontmatter();
        let skill = make_skill(fm, FrontmatterStatus::Present, None);

        let findings = lint_skill(&skill);

        assert!(!has_rule(&findings, "metadata.type"));
        assert!(!has_rule(&findings, "metadata.values"));
    }

    #[test]
    fn allowed_tools_list_is_allowed_tools_format_warning() {
        let mut fm = valid_frontmatter();
        fm.insert(
            "allowed-tools".to_string(),
            YamlValue::Sequence(vec![str_val("Bash"), str_val("Read")]),
        );
        let skill = make_skill(fm, FrontmatterStatus::Present, None);

        let findings = lint_skill(&skill);

        let finding = findings.iter().find(|f| f.rule == "allowed-tools.format");
        assert_eq!(finding.map(|f| f.severity), Some(Severity::Warning));
    }

    #[test]
    fn allowed_tools_string_is_none() {
        let mut fm = valid_frontmatter();
        fm.insert("allowed-tools".to_string(), str_val("Bash Read"));
        let skill = make_skill(fm, FrontmatterStatus::Present, None);

        let findings = lint_skill(&skill);

        assert!(!has_rule(&findings, "allowed-tools.format"));
        assert!(!has_rule(&findings, "allowed-tools.type"));
    }

    #[test]
    fn allowed_tools_number_is_allowed_tools_type_warning() {
        let mut fm = valid_frontmatter();
        fm.insert("allowed-tools".to_string(), YamlValue::Number(1.into()));
        let skill = make_skill(fm, FrontmatterStatus::Present, None);

        let findings = lint_skill(&skill);

        let finding = findings.iter().find(|f| f.rule == "allowed-tools.type");
        assert_eq!(finding.map(|f| f.severity), Some(Severity::Warning));
    }

    #[test]
    fn allowed_tools_absent_is_none() {
        let fm = valid_frontmatter();
        let skill = make_skill(fm, FrontmatterStatus::Present, None);

        let findings = lint_skill(&skill);

        assert!(!has_rule(&findings, "allowed-tools.format"));
        assert!(!has_rule(&findings, "allowed-tools.type"));
    }

    #[test]
    fn body_empty_is_body_empty_warning() {
        let fm = valid_frontmatter();
        let mut skill = make_skill(fm, FrontmatterStatus::Present, None);
        skill.body = "   \n\n  ".to_string();

        let findings = lint_skill(&skill);

        let finding = findings.iter().find(|f| f.rule == "body.empty");
        assert_eq!(finding.map(|f| f.severity), Some(Severity::Warning));
    }

    #[test]
    fn body_over_500_lines_is_body_lines_warning() {
        let fm = valid_frontmatter();
        let mut skill = make_skill(fm, FrontmatterStatus::Present, None);
        skill.body = "line\n".repeat(501);

        let findings = lint_skill(&skill);

        let finding = findings.iter().find(|f| f.rule == "body.lines");
        assert_eq!(finding.map(|f| f.severity), Some(Severity::Warning));
    }

    #[test]
    fn body_normal_is_neither() {
        let fm = valid_frontmatter();
        let mut skill = make_skill(fm, FrontmatterStatus::Present, None);
        skill.body = "Some instructions for the agent.".to_string();

        let findings = lint_skill(&skill);

        assert!(!has_rule(&findings, "body.empty"));
        assert!(!has_rule(&findings, "body.lines"));
    }

    #[test]
    fn unknown_top_level_key_is_frontmatter_unknown_info() {
        let mut fm = valid_frontmatter();
        fm.insert("random_field".to_string(), str_val("hello"));
        let skill = make_skill(fm, FrontmatterStatus::Present, None);

        let findings = lint_skill(&skill);

        let finding = findings.iter().find(|f| f.rule == "frontmatter.unknown");
        assert_eq!(finding.map(|f| f.severity), Some(Severity::Info));
    }

    #[test]
    fn only_known_fields_yields_no_unknown_finding() {
        let fm = valid_frontmatter();
        let skill = make_skill(fm, FrontmatterStatus::Present, None);

        let findings = lint_skill(&skill);

        assert!(!has_rule(&findings, "frontmatter.unknown"));
    }

    #[test]
    fn compatibility_non_string_is_compatibility_type_warning() {
        let mut fm = valid_frontmatter();
        fm.insert("compatibility".to_string(), YamlValue::Number(1.into()));
        let skill = make_skill(fm, FrontmatterStatus::Present, None);

        let findings = lint_skill(&skill);

        let finding = findings.iter().find(|f| f.rule == "compatibility.type");
        assert_eq!(finding.map(|f| f.severity), Some(Severity::Warning));
    }

    #[test]
    fn compatibility_string_under_500_is_none() {
        let mut fm = valid_frontmatter();
        fm.insert("compatibility".to_string(), str_val("works with any agent"));
        let skill = make_skill(fm, FrontmatterStatus::Present, None);

        let findings = lint_skill(&skill);

        assert!(!has_rule(&findings, "compatibility.type"));
        assert!(!has_rule(&findings, "compatibility.length"));
    }

    #[test]
    fn valid_skill_yields_zero_findings() {
        let fm = valid_frontmatter();
        let skill = make_skill(fm, FrontmatterStatus::Present, Some("valid-skill"));

        let findings = lint_skill(&skill);

        assert!(
            findings.is_empty(),
            "expected no findings, got {findings:?}"
        );
    }

    #[test]
    fn no_error_level_heuristic_dir_match_warning_detail_info() {
        // name.dir-match: warning, never error.
        let mut fm = valid_frontmatter();
        fm.insert("name".to_string(), str_val("valid-skill"));
        let skill = make_skill(fm, FrontmatterStatus::Present, Some("different"));
        let findings = lint_skill(&skill);
        let dir_match = findings.iter().find(|f| f.rule == "name.dir-match");
        assert_eq!(dir_match.map(|f| f.severity), Some(Severity::Warning));

        // description.detail: info, never error.
        let mut fm = valid_frontmatter();
        fm.insert("description".to_string(), str_val("short"));
        let skill = make_skill(fm, FrontmatterStatus::Present, None);
        let findings = lint_skill(&skill);
        let detail = findings.iter().find(|f| f.rule == "description.detail");
        assert_eq!(detail.map(|f| f.severity), Some(Severity::Info));

        // Neither ever appears as Severity::Error anywhere in this whole suite's
        // scope (spot-checked): confirm by construction above.
    }

    #[test]
    fn body_token_count_uses_a_real_tokenizer_not_chars_4() {
        // Pinned to cl100k_base's actual output (run once locally to get the
        // number): "tokenization" -> ["token", "ization"], 2 tokens. A
        // chars/4 heuristic would give 12/4 = 3, a different number — proving
        // this is the real BPE tokenizer, not a heuristic.
        let sample = "tokenization";
        assert_eq!(body_token_count(sample), 2);
        assert_ne!(body_token_count(sample), sample.chars().count() / 4);
    }

    #[test]
    fn short_body_has_no_body_size_finding() {
        let fm = valid_frontmatter();
        let mut skill = make_skill(fm, FrontmatterStatus::Present, None);
        skill.body = "Some short instructions for the agent.".to_string();

        let findings = lint_skill(&skill);

        assert!(!has_rule(&findings, "body.size"));
    }

    #[test]
    fn oversized_body_yields_one_body_size_info_finding_with_count() {
        let fm = valid_frontmatter();
        let mut skill = make_skill(fm, FrontmatterStatus::Present, None);
        // 700 repeats of a 47-char sentence -> 7001 cl100k_base tokens
        // (pinned by running the encoder once locally), safely over
        // BODY_TOKENS_THRESHOLD (5000).
        skill.body = "The quick brown fox jumps over the lazy dog. ".repeat(700);

        let findings: Vec<Finding> = lint_skill(&skill)
            .into_iter()
            .filter(|f| f.rule == "body.size")
            .collect();

        assert_eq!(findings.len(), 1, "expected exactly one body.size finding");
        let finding = &findings[0];
        assert_eq!(finding.severity, Severity::Info);
        assert!(
            finding.message.contains("7001"),
            "message should contain the token count: {}",
            finding.message
        );
    }

    #[test]
    fn body_just_under_threshold_yields_no_body_size_finding() {
        let fm = valid_frontmatter();
        let mut skill = make_skill(fm, FrontmatterStatus::Present, None);
        // 499 repeats -> 4991 cl100k_base tokens (pinned), under the 5000
        // threshold.
        skill.body = "The quick brown fox jumps over the lazy dog. ".repeat(499);

        let findings = lint_skill(&skill);

        assert!(!has_rule(&findings, "body.size"));
    }

    #[test]
    fn body_size_severity_is_info() {
        let fm = valid_frontmatter();
        let mut skill = make_skill(fm, FrontmatterStatus::Present, None);
        skill.body = "The quick brown fox jumps over the lazy dog. ".repeat(700);

        let findings = lint_skill(&skill);

        let finding = findings.iter().find(|f| f.rule == "body.size");
        assert_eq!(finding.map(|f| f.severity), Some(Severity::Info));
    }

    #[test]
    fn body_size_is_the_exact_stable_id() {
        let fm = valid_frontmatter();
        let mut skill = make_skill(fm, FrontmatterStatus::Present, None);
        skill.body = "The quick brown fox jumps over the lazy dog. ".repeat(700);

        let findings = lint_skill(&skill);

        assert!(findings.iter().any(|f| f.rule == "body.size"));
    }

    #[test]
    fn from_collection_over_lint_fixtures_good_has_zero_errors() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("lint-fixtures/good");

        let collection = walk(&root);
        let report = Report::from_collection(&collection, lint_skill);

        assert_eq!(
            report.summary.errors, 0,
            "expected zero errors linting lint-fixtures/good, got: {:#?}",
            report.sections
        );
    }

    // --- SPEC-011: --target claude ---------------------------------------

    #[test]
    fn target_claude_a_claude_field_does_not_trigger_frontmatter_unknown() {
        let mut fm = valid_frontmatter();
        fm.insert("context".to_string(), str_val("fork"));
        let skill = make_skill(fm, FrontmatterStatus::Present, None);

        let findings = lint_skill_with_target(&skill, Some(Target::Claude));

        assert!(!has_rule(&findings, "frontmatter.unknown"));
    }

    #[test]
    fn no_target_context_does_trigger_frontmatter_unknown_info() {
        let mut fm = valid_frontmatter();
        fm.insert("context".to_string(), str_val("fork"));
        let skill = make_skill(fm, FrontmatterStatus::Present, None);

        let findings = lint_skill_with_target(&skill, None);

        let finding = findings.iter().find(|f| f.rule == "frontmatter.unknown");
        assert_eq!(finding.map(|f| f.severity), Some(Severity::Info));
    }

    #[test]
    fn target_claude_a_truly_unknown_key_still_triggers_frontmatter_unknown() {
        let mut fm = valid_frontmatter();
        fm.insert("random_field".to_string(), str_val("hello"));
        let skill = make_skill(fm, FrontmatterStatus::Present, None);

        let findings = lint_skill_with_target(&skill, Some(Target::Claude));

        let finding = findings.iter().find(|f| f.rule == "frontmatter.unknown");
        assert_eq!(finding.map(|f| f.severity), Some(Severity::Info));
    }

    #[test]
    fn target_claude_allowed_tools_list_is_allowed_tools_format_info() {
        let mut fm = valid_frontmatter();
        fm.insert(
            "allowed-tools".to_string(),
            YamlValue::Sequence(vec![str_val("Bash"), str_val("Read")]),
        );
        let skill = make_skill(fm, FrontmatterStatus::Present, None);

        let findings = lint_skill_with_target(&skill, Some(Target::Claude));

        let finding = findings.iter().find(|f| f.rule == "allowed-tools.format");
        assert_eq!(finding.map(|f| f.severity), Some(Severity::Info));
    }

    #[test]
    fn no_target_allowed_tools_list_is_allowed_tools_format_warning() {
        let mut fm = valid_frontmatter();
        fm.insert(
            "allowed-tools".to_string(),
            YamlValue::Sequence(vec![str_val("Bash"), str_val("Read")]),
        );
        let skill = make_skill(fm, FrontmatterStatus::Present, None);

        let findings = lint_skill_with_target(&skill, None);

        let finding = findings.iter().find(|f| f.rule == "allowed-tools.format");
        assert_eq!(finding.map(|f| f.severity), Some(Severity::Warning));
    }

    #[test]
    fn target_claude_allowed_tools_type_number_still_warning() {
        let mut fm = valid_frontmatter();
        fm.insert("allowed-tools".to_string(), YamlValue::Number(1.into()));
        let skill = make_skill(fm, FrontmatterStatus::Present, None);

        let findings = lint_skill_with_target(&skill, Some(Target::Claude));

        let finding = findings.iter().find(|f| f.rule == "allowed-tools.type");
        assert_eq!(finding.map(|f| f.severity), Some(Severity::Warning));
    }

    #[test]
    fn target_claude_does_not_relax_name_required_or_description_length() {
        // name.required still fires when name is absent.
        let mut fm = valid_frontmatter();
        fm.shift_remove("name");
        let skill = make_skill(fm, FrontmatterStatus::Present, None);
        let findings = lint_skill_with_target(&skill, Some(Target::Claude));
        assert!(has_rule(&findings, "name.required"));

        // description.length still fires when description is too long.
        let mut fm = valid_frontmatter();
        fm.insert("description".to_string(), str_val(&"a".repeat(1025)));
        let skill = make_skill(fm, FrontmatterStatus::Present, None);
        let findings = lint_skill_with_target(&skill, Some(Target::Claude));
        assert!(has_rule(&findings, "description.length"));
    }

    #[test]
    fn lint_skill_no_target_is_unchanged() {
        // A Claude field -> still frontmatter.unknown info.
        let mut fm = valid_frontmatter();
        fm.insert("context".to_string(), str_val("fork"));
        let skill = make_skill(fm, FrontmatterStatus::Present, None);
        let findings = lint_skill(&skill);
        let finding = findings.iter().find(|f| f.rule == "frontmatter.unknown");
        assert_eq!(finding.map(|f| f.severity), Some(Severity::Info));

        // A list -> still allowed-tools.format warning.
        let mut fm = valid_frontmatter();
        fm.insert(
            "allowed-tools".to_string(),
            YamlValue::Sequence(vec![str_val("Bash")]),
        );
        let skill = make_skill(fm, FrontmatterStatus::Present, None);
        let findings = lint_skill(&skill);
        let finding = findings.iter().find(|f| f.rule == "allowed-tools.format");
        assert_eq!(finding.map(|f| f.severity), Some(Severity::Warning));
    }

    #[test]
    fn from_collection_over_lint_fixtures_good_claude_is_clean_with_target() {
        // "Clean" = 0 errors, 0 warnings (exit code 0). The Behavior-table
        // (the spec's authoritative row) has `allowed-tools.format` downgrade
        // to Info under `--target claude`, not disappear — so a fixture using
        // `allowed-tools:` as a list still carries exactly that one Info
        // finding under the target. See Build Completion "Deviations" for the
        // literal "0/0/0" vs. this reconciliation.
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("lint-fixtures/good-claude");

        let collection = walk(&root);
        let report = Report::from_collection(&collection, |s| {
            lint_skill_with_target(s, Some(Target::Claude))
        });

        assert_eq!(report.summary.errors, 0);
        assert_eq!(report.summary.warnings, 0);
        assert_eq!(
            report.exit_code(false),
            0,
            "expected exit code 0 under --target claude, got: {:#?}",
            report.sections
        );
        assert!(
            !has_rule_anywhere(&report, "frontmatter.unknown"),
            "expected no frontmatter.unknown under --target claude, got: {:#?}",
            report.sections
        );
    }

    #[test]
    fn from_collection_over_lint_fixtures_good_claude_without_target_has_findings() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("lint-fixtures/good-claude");

        let collection = walk(&root);
        let report = Report::from_collection(&collection, lint_skill);

        assert!(
            has_rule_anywhere(&report, "frontmatter.unknown"),
            "expected frontmatter.unknown without --target claude, got: {:#?}",
            report.sections
        );
        assert!(
            has_rule_anywhere(&report, "allowed-tools.format"),
            "expected allowed-tools.format without --target claude, got: {:#?}",
            report.sections
        );
    }

    fn has_rule_anywhere(report: &Report, rule: &str) -> bool {
        report
            .sections
            .iter()
            .any(|s| s.findings.iter().any(|f| f.rule == rule))
    }

    // --- SPEC-012: CLAUDE_KEYS completion + the rule catalog --------------

    #[test]
    fn claude_keys_complete() {
        let mut fm = valid_frontmatter();
        fm.insert("when_to_use".to_string(), str_val("more context"));
        fm.insert("argument-hint".to_string(), str_val("[file]"));
        fm.insert("agent".to_string(), str_val("some-agent"));
        fm.insert("paths".to_string(), str_val("src/**"));
        fm.insert("shell".to_string(), str_val("bash"));
        fm.insert("not_a_field".to_string(), str_val("hello"));
        let skill = make_skill(fm, FrontmatterStatus::Present, None);

        let with_target = lint_skill_with_target(&skill, Some(Target::Claude));
        let unknown_with_target: Vec<&Finding> = with_target
            .iter()
            .filter(|f| f.rule == "frontmatter.unknown")
            .collect();
        assert_eq!(
            unknown_with_target.len(),
            1,
            "expected exactly one frontmatter.unknown (for not_a_field) under --target claude, got: {with_target:?}"
        );
        assert_eq!(unknown_with_target[0].field.as_deref(), Some("not_a_field"));

        let without_target = lint_skill_with_target(&skill, None);
        let unknown_without_target = without_target
            .iter()
            .filter(|f| f.rule == "frontmatter.unknown")
            .count();
        assert_eq!(
            unknown_without_target, 6,
            "expected all 6 fields (5 Claude + 1 genuinely unknown) to fire frontmatter.unknown without a target, got: {without_target:?}"
        );
    }

    #[test]
    fn catalog_is_locked() {
        // The DEC-005 tripwire: adding/removing/renaming a rule id anywhere
        // in the engine (this file) or the structural layer (report.rs)
        // without updating `RULES` fails this test.
        const EXPECTED: &[&str] = &[
            "frontmatter.missing",
            "frontmatter.unclosed",
            "frontmatter.invalid",
            "frontmatter.unknown",
            "name.required",
            "name.type",
            "name.length",
            "name.charset",
            "name.hyphen-edges",
            "name.hyphen-consecutive",
            "name.dir-match",
            "description.required",
            "description.type",
            "description.length",
            "description.detail",
            "compatibility.length",
            "compatibility.type",
            "metadata.type",
            "metadata.values",
            "allowed-tools.format",
            "allowed-tools.type",
            "body.empty",
            "body.lines",
            "body.size",
            "file.unreadable",
            "dir.unreadable",
        ];

        let ids = all_rule_ids();
        assert_eq!(ids.len(), 26, "expected exactly 26 rule ids, got: {ids:?}");
        assert_eq!(ids, EXPECTED);

        let mut sorted = ids.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), ids.len(), "duplicate rule id in RULES");

        let structural_count = RULES.iter().filter(|r| r.structural).count();
        assert_eq!(structural_count, 2, "expected exactly 2 structural ids");
        assert!(RULES
            .iter()
            .any(|r| r.id == "file.unreadable" && r.structural && r.severity == Severity::Error));
        assert!(RULES
            .iter()
            .any(|r| r.id == "dir.unreadable" && r.structural && r.severity == Severity::Warning));
    }
}
