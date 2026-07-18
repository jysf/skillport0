//! Validation rules for a parsed skill.
//!
//! The firm rules come straight from the open Agent Skills specification
//! (agentskills.io/specification): name/description format and length,
//! compatibility/metadata/allowed-tools shape, and body-size guidance. On top
//! of that, an optional target profile widens the set of "recognized"
//! frontmatter fields so a Claude-Code-only extension (e.g. `effort`) isn't
//! flagged as unknown when you're linting for Claude.
//!
//! Note: the open-spec portion overlaps with the official `skills-ref validate`
//! tool. The value here is the per-platform layer plus linting a whole tree of
//! skills in one pass with a CI-friendly exit code.

use crate::profiles::{Keep, Profile};
use crate::skill::Skill;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

impl Severity {
    pub fn label(self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warn",
            Severity::Info => "info",
        }
    }
}

pub struct Finding {
    pub severity: Severity,
    pub rule: &'static str,
    pub message: String,
}

/// Frontmatter fields defined by the open spec.
const SPEC_KEYS: &[&str] = &[
    "name",
    "description",
    "license",
    "compatibility",
    "metadata",
    "allowed-tools",
];

/// Run every rule and collect findings. `dir_name` is the skill's parent
/// directory name (for the name/dir match rule); `target` optionally adds a
/// platform's recognized fields.
pub fn lint(skill: &Skill, dir_name: Option<&str>, target: Option<&Profile>) -> Vec<Finding> {
    let mut f = Vec::new();

    if skill.frontmatter.is_empty() {
        push(
            &mut f,
            Severity::Error,
            "frontmatter.missing",
            "no YAML frontmatter found; 'name' and 'description' are required",
        );
    } else {
        check_name(skill, dir_name, &mut f);
        check_description(skill, &mut f);
        check_compatibility(skill, &mut f);
        check_metadata(skill, &mut f);
        check_allowed_tools(skill, target, &mut f);
        check_unknown_fields(skill, target, &mut f);
    }

    check_body(skill, &mut f);
    f
}

fn push(f: &mut Vec<Finding>, severity: Severity, rule: &'static str, message: impl Into<String>) {
    f.push(Finding {
        severity,
        rule,
        message: message.into(),
    });
}

fn check_name(skill: &Skill, dir_name: Option<&str>, f: &mut Vec<Finding>) {
    let value = match skill.frontmatter.get("name") {
        None => {
            push(f, Severity::Error, "name.required", "'name' is required");
            return;
        }
        Some(v) => v,
    };
    let name = match value.as_str() {
        Some(s) => s,
        None => {
            push(f, Severity::Error, "name.type", "'name' must be a string");
            return;
        }
    };

    let len = name.chars().count();
    if len == 0 || len > 64 {
        push(
            f,
            Severity::Error,
            "name.length",
            format!("'name' must be 1-64 characters (got {len})"),
        );
    }
    let invalid: String = name
        .chars()
        .filter(|c| !(*c == '-' || (c.is_alphanumeric() && !c.is_uppercase())))
        .collect();
    if !invalid.is_empty() {
        push(
            f,
            Severity::Error,
            "name.charset",
            format!("'name' may only contain lowercase letters, digits, and hyphens (invalid: {invalid})"),
        );
    }
    if name.starts_with('-') || name.ends_with('-') {
        push(
            f,
            Severity::Error,
            "name.hyphen-edges",
            "'name' must not start or end with a hyphen",
        );
    }
    if name.contains("--") {
        push(
            f,
            Severity::Error,
            "name.hyphen-consecutive",
            "'name' must not contain consecutive hyphens",
        );
    }
    if let Some(dir) = dir_name {
        if name != dir {
            push(
                f,
                Severity::Warning,
                "name.dir-match",
                format!("'name' ({name}) should match the skill directory name ({dir})"),
            );
        }
    }
}

fn check_description(skill: &Skill, f: &mut Vec<Finding>) {
    let value = match skill.frontmatter.get("description") {
        None => {
            push(
                f,
                Severity::Error,
                "description.required",
                "'description' is required",
            );
            return;
        }
        Some(v) => v,
    };
    let desc = match value.as_str() {
        Some(s) => s,
        None => {
            push(
                f,
                Severity::Error,
                "description.type",
                "'description' must be a string",
            );
            return;
        }
    };

    let len = desc.chars().count();
    if len == 0 {
        push(
            f,
            Severity::Error,
            "description.length",
            "'description' must not be empty",
        );
    } else if len > 1024 {
        push(
            f,
            Severity::Error,
            "description.length",
            format!("'description' must be at most 1024 characters (got {len})"),
        );
    } else if len < 40 {
        push(
            f,
            Severity::Info,
            "description.detail",
            "'description' is short; the spec recommends stating both what the skill does and when to use it",
        );
    }
}

fn check_compatibility(skill: &Skill, f: &mut Vec<Finding>) {
    if let Some(value) = skill.frontmatter.get("compatibility") {
        match value.as_str() {
            Some(s) if s.chars().count() > 500 => push(
                f,
                Severity::Error,
                "compatibility.length",
                format!("'compatibility' must be at most 500 characters (got {})", s.chars().count()),
            ),
            Some(_) => {}
            None => push(
                f,
                Severity::Warning,
                "compatibility.type",
                "'compatibility' should be a string",
            ),
        }
    }
}

fn check_metadata(skill: &Skill, f: &mut Vec<Finding>) {
    if let Some(value) = skill.frontmatter.get("metadata") {
        match value.as_mapping() {
            None => push(
                f,
                Severity::Warning,
                "metadata.type",
                "'metadata' should be a key-value map",
            ),
            Some(map) => {
                for (k, v) in map {
                    if !v.is_string() {
                        let key = k.as_str().unwrap_or("?");
                        push(
                            f,
                            Severity::Info,
                            "metadata.values",
                            format!("metadata.{key} is not a string; the spec defines metadata as string-to-string (quote values like \"1.0\")"),
                        );
                    }
                }
            }
        }
    }
}

fn check_allowed_tools(skill: &Skill, target: Option<&Profile>, f: &mut Vec<Finding>) {
    if let Some(value) = skill.frontmatter.get("allowed-tools") {
        if value.is_sequence() {
            // Some agents (e.g. Claude Code) have accepted a list; the open
            // spec defines a space-separated string. Flag firmly only for the
            // portable target.
            let severity = if is_open(target) {
                Severity::Warning
            } else {
                Severity::Info
            };
            push(
                f,
                severity,
                "allowed-tools.format",
                "the open spec defines 'allowed-tools' as a space-separated string, not a list",
            );
        } else if !value.is_string() {
            push(
                f,
                Severity::Warning,
                "allowed-tools.type",
                "'allowed-tools' should be a space-separated string",
            );
        }
    }
}

fn check_body(skill: &Skill, f: &mut Vec<Finding>) {
    if skill.body.trim().is_empty() {
        push(
            f,
            Severity::Warning,
            "body.empty",
            "the SKILL.md body is empty; add instructions for the agent",
        );
        return;
    }
    let lines = skill.body.lines().count();
    if lines > 500 {
        push(
            f,
            Severity::Warning,
            "body.lines",
            format!("body is {lines} lines; the spec recommends keeping SKILL.md under 500 (move detail into references/)"),
        );
    }
    let approx_tokens = skill.body.chars().count() / 4;
    if approx_tokens > 5000 {
        push(
            f,
            Severity::Warning,
            "body.size",
            format!("body is ~{approx_tokens} tokens (rough estimate); the spec recommends under 5000 (use progressive disclosure)"),
        );
    }
}

fn check_unknown_fields(skill: &Skill, target: Option<&Profile>, f: &mut Vec<Finding>) {
    let recognized = recognized_keys(target);
    let suffix = match target {
        Some(p) => format!(" for target '{}'", p.id),
        None => String::new(),
    };
    for key in skill.keys() {
        if !recognized.iter().any(|k| *k == key) {
            push(
                f,
                Severity::Info,
                "frontmatter.unknown",
                format!("'{key}' is not a recognized field{suffix}; compliant agents ignore unknown keys"),
            );
        }
    }
}

fn recognized_keys(target: Option<&Profile>) -> Vec<&'static str> {
    let mut keys: Vec<&'static str> = SPEC_KEYS.to_vec();
    if let Some(p) = target {
        if let Keep::Only(list) = p.keep {
            for k in list {
                if !keys.contains(k) {
                    keys.push(k);
                }
            }
        }
        for (from, _) in p.renames {
            if !keys.contains(from) {
                keys.push(from);
            }
        }
    }
    keys
}

fn is_open(target: Option<&Profile>) -> bool {
    match target {
        None => true,
        Some(p) => p.id == "open",
    }
}
