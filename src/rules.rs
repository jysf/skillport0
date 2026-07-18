//! The open-spec rule engine (STAGE-002, spec 1 of it).
//!
//! `lint_skill` is the `rule_fn` seam `Report::from_collection` expects
//! (SPEC-003): given one parsed [`Skill`], run every implemented open-spec
//! rule and return its findings, unordered — the report layer sorts them.
//!
//! This spec covers only frontmatter presence and the `name.*` /
//! `description.*` / `compatibility.length` rules (the crisp, high-value
//! identity/description batch). The rest of the catalog (`metadata.*`,
//! `allowed-tools.*`, `body.*`, `frontmatter.unknown`) is SPEC-005.
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

/// The `description.detail` terseness threshold (soft, tunable; info-only so
/// a false positive is harmless). Ported from the prototype's `< 40` chars.
const DESCRIPTION_DETAIL_THRESHOLD: usize = 40;

/// Run every implemented open-spec rule over `skill` and return its findings.
/// Unordered — `Report::from_collection` sorts deterministically.
pub fn lint_skill(skill: &Skill) -> Vec<Finding> {
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

    // Lowercase letters, digits, and hyphens only. Reject uppercase even
    // though `char::is_alphanumeric` accepts it (mirrors the prototype).
    let invalid: String = name
        .chars()
        .filter(|c| !(*c == '-' || (c.is_alphanumeric() && !c.is_uppercase())))
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
        if let Some(s) = value.as_str() {
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
        // A non-string `compatibility` is out of scope for this spec (only
        // `compatibility.length` is in the table); SPEC-005 may add a type
        // check alongside the rest of the catalog.
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
}
