//! The finding + severity + sectioned report model.
//!
//! This is the assembly layer that turns a walked [`crate::walk::Collection`]
//! into a deterministic, sectioned [`Report`]: one [`Section`] per collection
//! item, path-sorted, each containing [`Finding`]s in a stable order. A
//! `CollectionItem::Unreadable` becomes exactly one structural `file.unreadable`
//! error finding here (DEC-003 — a crisp mechanical fact, not a heuristic); a
//! `CollectionItem::Skill` runs the caller-supplied `rule_fn` (the STAGE-002
//! rule-engine seam). This module owns no open-spec rule and no heuristic —
//! see DEC-003/DEC-004/DEC-005 and `guidance/constraints.yaml`.

use crate::skill::Skill;
use crate::walk::{Collection, CollectionItem};
use std::cmp::Reverse;
use std::path::PathBuf;

/// The stable id for the one structural finding this module emits for an
/// unreadable file.
const FILE_UNREADABLE: &str = "file.unreadable";
/// The stable id for the structural finding emitted for a directory the walk
/// could not descend into (coverage gap, not a skill violation).
const DIR_UNREADABLE: &str = "dir.unreadable";

/// A finding's severity. Total order: `Error > Warning > Info` (DEC-003).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// Least severe, ordered first so the derived `Ord` still needs a
    /// deliberate reversal wherever "most severe first" is required.
    Info,
    Warning,
    Error,
}

impl Severity {
    /// The lowercase label used in human/CI output.
    pub fn label(self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "info",
        }
    }
}

/// One reported fact about a skill or file. `rule` is a stable, compile-time
/// id (the public contract, DEC-005) — never a heap-allocated/formatted
/// string, so ids can't drift between call sites.
#[derive(Debug, Clone)]
pub struct Finding {
    pub rule: &'static str,
    pub severity: Severity,
    pub message: String,
    pub path: PathBuf,
    pub field: Option<String>,
    pub line: Option<usize>,
}

/// All findings for one collection item (one `SKILL.md` or unreadable file).
#[derive(Debug, Clone)]
pub struct Section {
    pub path: PathBuf,
    pub findings: Vec<Finding>,
}

/// Aggregate counts across the whole report.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Summary {
    pub skills: usize,
    pub errors: usize,
    pub warnings: usize,
    pub infos: usize,
}

/// The full, sectioned report assembled from a [`Collection`].
#[derive(Debug, Clone)]
pub struct Report {
    pub sections: Vec<Section>,
    pub summary: Summary,
}

impl Report {
    /// Assemble a [`Report`] from a walked `collection`. `rule_fn` is the
    /// STAGE-002 rule-engine seam; STAGE-001 callers pass a no-op
    /// (`|_| Vec::new()`). A `CollectionItem::Unreadable` becomes one
    /// structural `file.unreadable` error finding here; a
    /// `CollectionItem::Skill` runs `rule_fn` untouched.
    ///
    /// Deterministic regardless of input order (DEC-005): `sections` are
    /// sorted by path ascending, and findings within a section are sorted
    /// severity-descending, then by `rule` id, then by `field`/`message`.
    pub fn from_collection(
        collection: &Collection,
        rule_fn: impl Fn(&Skill) -> Vec<Finding>,
    ) -> Report {
        let mut summary = Summary::default();
        let mut sections: Vec<Section> = collection
            .items
            .iter()
            .map(|item| match item {
                CollectionItem::Skill(skill) => {
                    summary.skills += 1;
                    let mut findings = rule_fn(skill);
                    sort_findings(&mut findings);
                    tally(&mut summary, &findings);
                    Section {
                        path: skill.path.clone(),
                        findings,
                    }
                }
                CollectionItem::Unreadable { path, error } => {
                    let finding = Finding {
                        rule: FILE_UNREADABLE,
                        severity: Severity::Error,
                        message: format!("could not read file: {error}"),
                        path: path.clone(),
                        field: None,
                        line: None,
                    };
                    let findings = vec![finding];
                    tally(&mut summary, &findings);
                    Section {
                        path: path.clone(),
                        findings,
                    }
                }
                CollectionItem::UnreadableDir { path, error } => {
                    let finding = Finding {
                        rule: DIR_UNREADABLE,
                        severity: Severity::Warning,
                        message: format!(
                            "could not read directory (skills inside were not checked): {error}"
                        ),
                        path: path.clone(),
                        field: None,
                        line: None,
                    };
                    let findings = vec![finding];
                    tally(&mut summary, &findings);
                    Section {
                        path: path.clone(),
                        findings,
                    }
                }
            })
            .collect();

        sections.sort_by(|a, b| a.path.cmp(&b.path));

        Report { sections, summary }
    }

    /// The CI exit-code contract (DEC-003/005): 1 if any `Error`; 1 if
    /// `strict` and any `Warning`; 0 otherwise. `Info` never affects it.
    /// Usage errors (exit 2) are a CLI-level concern, not this function's.
    pub fn exit_code(&self, strict: bool) -> i32 {
        let fails = self.summary.errors > 0 || (strict && self.summary.warnings > 0);
        i32::from(fails)
    }
}

/// Sort a section's findings deterministically: severity descending, then
/// rule id, then field, then message — so identical input always yields
/// byte-identical output (DEC-005).
fn sort_findings(findings: &mut [Finding]) {
    findings.sort_by(|a, b| {
        Reverse(a.severity)
            .cmp(&Reverse(b.severity))
            .then_with(|| a.rule.cmp(b.rule))
            .then_with(|| a.field.cmp(&b.field))
            .then_with(|| a.message.cmp(&b.message))
    });
}

/// Fold a slice of findings' severities into the running summary counts.
fn tally(summary: &mut Summary, findings: &[Finding]) {
    for finding in findings {
        match finding.severity {
            Severity::Error => summary.errors += 1,
            Severity::Warning => summary.warnings += 1,
            Severity::Info => summary.infos += 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skill::{Frontmatter, FrontmatterStatus};
    use std::path::Path;

    /// A minimal, parsed-looking `Skill` for tests that don't care about
    /// frontmatter contents.
    fn skill_at(path: &str) -> Skill {
        Skill {
            path: PathBuf::from(path),
            dir_name: None,
            frontmatter: Frontmatter::new(),
            body: String::new(),
            raw: String::new(),
            frontmatter_status: FrontmatterStatus::Present,
        }
    }

    fn unreadable_at(path: &str, error: &str) -> CollectionItem {
        CollectionItem::Unreadable {
            path: PathBuf::from(path),
            error: error.to_string(),
        }
    }

    fn unreadable_dir_at(path: &str, error: &str) -> CollectionItem {
        CollectionItem::UnreadableDir {
            path: PathBuf::from(path),
            error: error.to_string(),
        }
    }

    fn collection(items: Vec<CollectionItem>) -> Collection {
        Collection {
            root: PathBuf::from("."),
            items,
        }
    }

    fn finding(rule: &'static str, severity: Severity, path: &str) -> Finding {
        Finding {
            rule,
            severity,
            message: format!("{rule} message"),
            path: PathBuf::from(path),
            field: None,
            line: None,
        }
    }

    #[test]
    fn exit_code_table() {
        let mut report = Report {
            sections: Vec::new(),
            summary: Summary::default(),
        };

        // Error -> 1 regardless of strict.
        report.summary = Summary {
            skills: 1,
            errors: 1,
            warnings: 0,
            infos: 0,
        };
        assert_eq!(report.exit_code(false), 1);
        assert_eq!(report.exit_code(true), 1);

        // Warning, not strict -> 0.
        report.summary = Summary {
            skills: 1,
            errors: 0,
            warnings: 1,
            infos: 0,
        };
        assert_eq!(report.exit_code(false), 0);

        // Warning, strict -> 1.
        assert_eq!(report.exit_code(true), 1);

        // Info only -> 0 either way.
        report.summary = Summary {
            skills: 1,
            errors: 0,
            warnings: 0,
            infos: 1,
        };
        assert_eq!(report.exit_code(false), 0);
        assert_eq!(report.exit_code(true), 0);

        // Empty -> 0.
        report.summary = Summary::default();
        assert_eq!(report.exit_code(false), 0);
        assert_eq!(report.exit_code(true), 0);
    }

    #[test]
    fn unreadable_item_becomes_file_unreadable_error() {
        let coll = collection(vec![unreadable_at("bad/SKILL.md", "invalid UTF-8: x")]);

        let report = Report::from_collection(&coll, |_| Vec::new());

        assert_eq!(report.sections.len(), 1);
        let section = &report.sections[0];
        assert_eq!(section.path, Path::new("bad/SKILL.md"));
        assert_eq!(section.findings.len(), 1);
        let f = &section.findings[0];
        assert_eq!(f.rule, "file.unreadable");
        assert_eq!(f.severity, Severity::Error);
        assert_eq!(f.path, Path::new("bad/SKILL.md"));
    }

    #[test]
    fn skill_item_runs_rule_fn() {
        let coll = collection(vec![CollectionItem::Skill(skill_at("a/SKILL.md"))]);

        let report = Report::from_collection(&coll, |skill| {
            vec![finding(
                "test.warn",
                Severity::Warning,
                skill.path.to_str().unwrap(),
            )]
        });

        assert_eq!(report.sections.len(), 1);
        let section = &report.sections[0];
        assert_eq!(section.findings.len(), 1);
        assert_eq!(section.findings[0].rule, "test.warn");
        assert_eq!(section.findings[0].severity, Severity::Warning);
        assert_eq!(report.summary.warnings, 1);
    }

    #[test]
    fn no_op_rule_fn_on_readable_collection_yields_zero_findings() {
        let coll = collection(vec![
            CollectionItem::Skill(skill_at("a/SKILL.md")),
            CollectionItem::Skill(skill_at("b/SKILL.md")),
        ]);

        let report = Report::from_collection(&coll, |_| Vec::new());

        assert_eq!(report.sections.len(), 2);
        assert!(report.sections.iter().all(|s| s.findings.is_empty()));
        assert_eq!(report.summary.skills, 2);
        assert_eq!(report.summary.errors, 0);
        assert_eq!(report.summary.warnings, 0);
        assert_eq!(report.summary.infos, 0);
    }

    #[test]
    fn sections_sorted_by_path_regardless_of_input_order() {
        let coll = collection(vec![
            CollectionItem::Skill(skill_at("zzz/SKILL.md")),
            CollectionItem::Skill(skill_at("aaa/SKILL.md")),
            unreadable_at("mmm/SKILL.md", "err"),
        ]);

        let report = Report::from_collection(&coll, |_| Vec::new());

        let paths: Vec<&Path> = report.sections.iter().map(|s| s.path.as_path()).collect();
        let mut sorted = paths.clone();
        sorted.sort();
        assert_eq!(paths, sorted);
    }

    #[test]
    fn findings_within_a_section_are_deterministically_ordered() {
        let coll = collection(vec![CollectionItem::Skill(skill_at("a/SKILL.md"))]);

        let report = Report::from_collection(&coll, |skill| {
            let p = skill.path.to_str().unwrap();
            vec![
                finding("zzz.info", Severity::Info, p),
                finding("aaa.error", Severity::Error, p),
                finding("mmm.warn", Severity::Warning, p),
            ]
        });

        let section = &report.sections[0];
        let severities: Vec<Severity> = section.findings.iter().map(|f| f.severity).collect();
        assert_eq!(
            severities,
            vec![Severity::Error, Severity::Warning, Severity::Info]
        );
    }

    #[test]
    fn summary_counts_skills_and_severities() {
        let coll = collection(vec![
            unreadable_at("bad/SKILL.md", "err"),
            CollectionItem::Skill(skill_at("a/SKILL.md")),
            CollectionItem::Skill(skill_at("b/SKILL.md")),
        ]);

        let report = Report::from_collection(&coll, |skill| {
            let p = skill.path.to_str().unwrap();
            if p.starts_with('a') {
                vec![
                    finding("test.error", Severity::Error, p),
                    finding("test.warn", Severity::Warning, p),
                ]
            } else {
                vec![finding("test.info", Severity::Info, p)]
            }
        });

        // 1 unreadable error + a's error/warning + b's info.
        assert_eq!(report.summary.skills, 2);
        assert_eq!(report.summary.errors, 2);
        assert_eq!(report.summary.warnings, 1);
        assert_eq!(report.summary.infos, 1);
    }

    #[test]
    fn file_unreadable_is_the_exact_stable_id() {
        assert_eq!(FILE_UNREADABLE, "file.unreadable");

        let coll = collection(vec![unreadable_at("x/SKILL.md", "err")]);
        let report = Report::from_collection(&coll, |_| Vec::new());
        assert_eq!(report.sections[0].findings[0].rule, "file.unreadable");
    }

    #[test]
    fn unreadable_dir_becomes_one_dir_unreadable_warning_skills_unchanged() {
        let coll = collection(vec![
            unreadable_dir_at("locked", "permission denied"),
            CollectionItem::Skill(skill_at("a/SKILL.md")),
        ]);

        let report = Report::from_collection(&coll, |_| Vec::new());

        assert_eq!(report.sections.len(), 2);
        let section = report
            .sections
            .iter()
            .find(|s| s.path == Path::new("locked"))
            .expect("locked section present");
        assert_eq!(section.findings.len(), 1);
        let f = &section.findings[0];
        assert_eq!(f.rule, "dir.unreadable");
        assert_eq!(f.severity, Severity::Warning);
        assert_eq!(f.path, Path::new("locked"));

        assert_eq!(report.summary.warnings, 1);
        assert_eq!(report.summary.errors, 0);
        // Only the real skill counts toward summary.skills.
        assert_eq!(report.summary.skills, 1);
    }

    #[test]
    fn dir_unreadable_is_the_exact_stable_id() {
        assert_eq!(DIR_UNREADABLE, "dir.unreadable");

        let coll = collection(vec![unreadable_dir_at("locked", "err")]);
        let report = Report::from_collection(&coll, |_| Vec::new());
        assert_eq!(report.sections[0].findings[0].rule, "dir.unreadable");
    }

    #[test]
    fn exit_code_dir_unreadable_warning_non_strict_0_strict_1() {
        let coll = collection(vec![unreadable_dir_at("locked", "err")]);
        let report = Report::from_collection(&coll, |_| Vec::new());

        assert_eq!(report.summary.warnings, 1);
        assert_eq!(report.summary.errors, 0);
        assert_eq!(report.exit_code(false), 0);
        assert_eq!(report.exit_code(true), 1);
    }
}
