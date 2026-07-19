//! Integration tests for the real `skillport lint` binary (SPEC-005).
//!
//! Runs the built binary via `env!("CARGO_BIN_EXE_skillport")` (no extra
//! dev-dependency) and asserts on its exit code and stdout/stderr streams —
//! the actual CI contract (`docs/api-contract.md`), not just the library
//! internals `src/emit.rs`'s unit tests cover.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// Absolute path to a fixture under `lint-fixtures/`, resolved from the
/// package root so these tests work regardless of the invoking cwd.
fn fixture(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

/// Run `skillport lint <args>` and capture the outcome.
fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_skillport"))
        .arg("lint")
        .args(args)
        .output()
        .expect("failed to run skillport binary")
}

#[test]
fn lint_good_fixture_exits_0() {
    let out = run(&[fixture("lint-fixtures/good").to_str().unwrap()]);

    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr: {:?}",
        string(&out.stderr)
    );
}

#[test]
fn lint_bad_fixture_exits_1_and_mentions_name_charset() {
    let out = run(&[fixture("lint-fixtures/bad").to_str().unwrap()]);

    assert_eq!(out.status.code(), Some(1));
    let stdout = string(&out.stdout);
    assert!(
        stdout.contains("name.charset"),
        "expected stdout to mention name.charset, got: {stdout}"
    );
}

#[test]
fn lint_bad_fixture_mentions_spec_006_rules() {
    // SPEC-006: allowed-tools.format and frontmatter.unknown are new rules
    // that must flow through the unchanged CLI (main.rs/emit.rs untouched).
    let out = run(&[fixture("lint-fixtures/bad").to_str().unwrap()]);

    assert_eq!(out.status.code(), Some(1));
    let stdout = string(&out.stdout);
    assert!(
        stdout.contains("allowed-tools.format"),
        "expected stdout to mention allowed-tools.format, got: {stdout}"
    );
    assert!(
        stdout.contains("frontmatter.unknown"),
        "expected stdout to mention frontmatter.unknown, got: {stdout}"
    );
}

#[test]
fn good_fixture_strict_still_exits_0() {
    let out = run(&[fixture("lint-fixtures/good").to_str().unwrap(), "--strict"]);

    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr: {:?}",
        string(&out.stderr)
    );
}

#[test]
fn lint_json_is_valid_json_and_exit_code_reflects_findings() {
    let out = run(&[fixture("lint-fixtures").to_str().unwrap(), "--json"]);

    assert_eq!(out.status.code(), Some(1));
    let stdout = string(&out.stdout);
    let value: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("stdout was not valid JSON ({e}): {stdout}"));
    let errors = value["summary"]["errors"]
        .as_u64()
        .expect("summary.errors present");
    assert!(errors > 0, "expected summary.errors > 0, got {errors}");
}

#[test]
fn missing_path_exits_2_stderr_message_empty_stdout() {
    let out = run(&[fixture("lint-fixtures/does-not-exist").to_str().unwrap()]);

    assert_eq!(out.status.code(), Some(2));
    assert!(
        out.stdout.is_empty(),
        "expected empty stdout, got: {:?}",
        string(&out.stdout)
    );
    assert!(
        !out.stderr.is_empty(),
        "expected a stderr usage-error message"
    );
}

#[test]
fn strict_flips_warning_only_fixture_to_exit_1() {
    let path = fixture("lint-fixtures/warn-only");

    let without_strict = run(&[path.to_str().unwrap()]);
    assert_eq!(
        without_strict.status.code(),
        Some(0),
        "warning-only fixture should pass without --strict; stdout: {}",
        string(&without_strict.stdout)
    );

    let with_strict = run(&[path.to_str().unwrap(), "--strict"]);
    assert_eq!(
        with_strict.status.code(),
        Some(1),
        "warning-only fixture should fail under --strict; stdout: {}",
        string(&with_strict.stdout)
    );
}

#[test]
fn lint_sarif_on_bad_fixture_is_valid_json_exit_1_mentions_name_charset() {
    let out = run(&[fixture("lint-fixtures/bad").to_str().unwrap(), "--sarif"]);

    assert_eq!(out.status.code(), Some(1));
    let stdout = string(&out.stdout);
    let value: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("stdout was not valid SARIF JSON ({e}): {stdout}"));
    assert_eq!(value["version"], "2.1.0");
    assert!(
        stdout.contains("name.charset"),
        "expected stdout to mention name.charset, got: {stdout}"
    );
}

#[test]
fn lint_sarif_on_good_fixture_exits_0_with_empty_results() {
    let out = run(&[fixture("lint-fixtures/good").to_str().unwrap(), "--sarif"]);

    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr: {:?}",
        string(&out.stderr)
    );
    let stdout = string(&out.stdout);
    let value: serde_json::Value =
        serde_json::from_str(&stdout).unwrap_or_else(|e| panic!("not valid JSON ({e}): {stdout}"));
    assert_eq!(
        value["runs"][0]["results"].as_array().unwrap().len(),
        0,
        "expected empty results for the good fixture"
    );
}

#[test]
fn lint_sarif_and_json_together_exits_2_with_empty_stdout() {
    let out = run(&[
        fixture("lint-fixtures/good").to_str().unwrap(),
        "--sarif",
        "--json",
    ]);

    assert_eq!(out.status.code(), Some(2));
    assert!(
        out.stdout.is_empty(),
        "expected empty stdout, got: {:?}",
        string(&out.stdout)
    );
    assert!(
        !out.stderr.is_empty(),
        "expected a clap usage-error message on stderr"
    );
}

fn string(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

// --- SPEC-011: --target claude ------------------------------------------

#[test]
fn lint_target_claude_on_claude_fields_fixture_is_clean() {
    // "Clean" = 0 errors, no frontmatter.unknown for the Claude fields
    // (`context`). `allowed-tools.format` still fires as info (Behavior
    // table: list downgrades warning -> info, it does not disappear).
    let out = run(&[
        fixture("lint-fixtures/good-claude").to_str().unwrap(),
        "--target",
        "claude",
    ]);

    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr: {:?}",
        string(&out.stderr)
    );
    let stdout = string(&out.stdout);
    assert!(
        !stdout.contains("frontmatter.unknown"),
        "expected no frontmatter.unknown for Claude fields, got: {stdout}"
    );
    assert!(
        stdout.contains("info") && stdout.contains("allowed-tools.format"),
        "expected allowed-tools.format at info under --target claude, got: {stdout}"
    );
}

#[test]
fn lint_good_claude_fixture_without_target_has_findings() {
    let out = run(&[fixture("lint-fixtures/good-claude").to_str().unwrap()]);

    let stdout = string(&out.stdout);
    assert!(
        stdout.contains("frontmatter.unknown"),
        "expected frontmatter.unknown without --target claude, got: {stdout}"
    );
    assert!(
        stdout.contains("allowed-tools.format"),
        "expected allowed-tools.format without --target claude, got: {stdout}"
    );
}

#[test]
fn lint_target_bogus_exits_2() {
    let out = run(&[
        fixture("lint-fixtures/good").to_str().unwrap(),
        "--target",
        "bogus",
    ]);

    assert_eq!(out.status.code(), Some(2));
    assert!(
        out.stdout.is_empty(),
        "expected empty stdout, got: {:?}",
        string(&out.stdout)
    );
    assert!(
        !out.stderr.is_empty(),
        "expected a clap usage-error message on stderr"
    );
}

#[test]
fn lint_target_claude_json_has_target_claude_label() {
    let out = run(&[
        fixture("lint-fixtures/good-claude").to_str().unwrap(),
        "--target",
        "claude",
        "--json",
    ]);

    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr: {:?}",
        string(&out.stderr)
    );
    let stdout = string(&out.stdout);
    let value: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("stdout was not valid JSON ({e}): {stdout}"));
    assert_eq!(value["target"], "claude");
}

#[test]
fn lint_json_without_target_has_null_target_label() {
    let out = run(&[fixture("lint-fixtures/good").to_str().unwrap(), "--json"]);

    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr: {:?}",
        string(&out.stderr)
    );
    let stdout = string(&out.stdout);
    let value: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("stdout was not valid JSON ({e}): {stdout}"));
    assert_eq!(value["target"], serde_json::Value::Null);
}

#[cfg(unix)]
#[test]
fn unreadable_subdir_surfaces_dir_unreadable_warning() {
    use std::os::unix::fs::PermissionsExt;

    let tree = tempfile::tempdir().expect("create temp dir");
    let good = tree.path().join("good");
    std::fs::create_dir_all(&good).expect("create good dir");
    std::fs::write(
        good.join("SKILL.md"),
        "---\nname: foo\ndescription: d\n---\nbody\n",
    )
    .expect("write skill");

    let locked = tree.path().join("locked");
    std::fs::create_dir_all(&locked).expect("create locked dir");
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o000))
        .expect("chmod 000 locked dir");

    // Guard restores permissions on drop (including panic) so tempdir cleanup works.
    struct RestorePerms(std::path::PathBuf);
    impl Drop for RestorePerms {
        fn drop(&mut self) {
            let _ = std::fs::set_permissions(&self.0, std::fs::Permissions::from_mode(0o755));
        }
    }
    let _restore = RestorePerms(locked.clone());

    let without_strict = run(&[tree.path().to_str().unwrap()]);
    let stdout = string(&without_strict.stdout);
    assert!(
        stdout.contains("dir.unreadable"),
        "expected stdout to mention dir.unreadable, got: {stdout}"
    );
    assert_eq!(
        without_strict.status.code(),
        Some(0),
        "a lone dir.unreadable warning should not fail non-strict lint; stdout: {stdout}"
    );

    let with_strict = run(&[tree.path().to_str().unwrap(), "--strict"]);
    assert_eq!(
        with_strict.status.code(),
        Some(1),
        "dir.unreadable warning should fail under --strict"
    );
}

// --- SPEC-012: rule catalog drift/coverage + spec-perfect fixture --------

/// Every rule id found in `lint-fixtures/`'s `--json` output, running once
/// without `--target` and once with `--target claude` (so the
/// `allowed-tools.format` Info variant and the Claude-recognized-field
/// behavior are both exercised).
fn all_emitted_rule_ids() -> std::collections::BTreeSet<String> {
    let root = fixture("lint-fixtures");
    let mut ids = std::collections::BTreeSet::new();
    for args in [
        vec![root.to_str().unwrap(), "--json"],
        vec![root.to_str().unwrap(), "--json", "--target", "claude"],
    ] {
        let out = run(&args);
        let stdout = string(&out.stdout);
        let value: serde_json::Value = serde_json::from_str(&stdout)
            .unwrap_or_else(|e| panic!("stdout was not valid JSON ({e}): {stdout}"));
        let sections = value["sections"].as_array().expect("sections array");
        for section in sections {
            let findings = section["findings"].as_array().expect("findings array");
            for finding in findings {
                let rule = finding["rule"].as_str().expect("rule string");
                ids.insert(rule.to_string());
            }
        }
    }
    ids
}

#[test]
fn no_orphan_rule_ids() {
    // Every rule any fixture emits must be in the catalog (guards against an
    // id the catalog forgot).
    let catalog: std::collections::BTreeSet<&str> = skillport::RULES.iter().map(|r| r.id).collect();

    for rule in all_emitted_rule_ids() {
        assert!(
            catalog.contains(rule.as_str()),
            "emitted rule '{rule}' is not in the RULES catalog"
        );
    }
}

#[test]
fn every_engine_rule_has_a_fixture() {
    // Every *engine* (non-structural) rule id in the catalog must be emitted
    // by at least one committed fixture. The 2 structural ids
    // (file.unreadable/dir.unreadable) are explicitly excused here — they
    // require a non-UTF-8 file / an unreadable directory, already covered by
    // `report.rs`'s unit tests, not a `SKILL.md` fixture.
    let emitted = all_emitted_rule_ids();
    let engine_ids: Vec<&str> = skillport::RULES
        .iter()
        .filter(|r| !r.structural)
        .map(|r| r.id)
        .collect();

    let missing: Vec<&&str> = engine_ids
        .iter()
        .filter(|id| !emitted.contains(**id))
        .collect();

    assert!(
        missing.is_empty(),
        "engine rule id(s) with no covering fixture in lint-fixtures/: {missing:?}"
    );
}

#[test]
fn spec_perfect_skill_is_clean() {
    // lint-fixtures/good/data-analysis is the designated spec-perfect
    // fixture: 0/0/0 both with and without --target claude.
    let path = fixture("lint-fixtures/good/data-analysis");

    let without_target = run(&[path.to_str().unwrap()]);
    assert_eq!(without_target.status.code(), Some(0));
    let stdout = string(&without_target.stdout);
    assert!(
        stdout.contains("0 error(s), 0 warning(s), 0 info(s)"),
        "expected 0/0/0 without --target claude, got: {stdout}"
    );

    let with_target = run(&[path.to_str().unwrap(), "--target", "claude"]);
    assert_eq!(with_target.status.code(), Some(0));
    let stdout = string(&with_target.stdout);
    assert!(
        stdout.contains("0 error(s), 0 warning(s), 0 info(s)"),
        "expected 0/0/0 under --target claude, got: {stdout}"
    );
}

#[test]
fn readme_rule_table_matches_catalog() {
    // Parse only the `## Rule reference` table region: rows with a
    // backtick-wrapped rule id in the first column and a severity word
    // (error/warning/info) in the severity column. Defensive on purpose —
    // it must not assert on prose (SPEC-012, "Notes for the Implementer").
    let readme = std::fs::read_to_string(fixture("README.md")).expect("read README.md");

    let start = readme
        .find("## Rule reference")
        .expect("README.md must have a '## Rule reference' section");
    let rest = &readme[start..];
    let end = rest[3..].find("\n## ").map(|i| i + 3).unwrap_or(rest.len());
    let section = &rest[..end];

    let mut documented: std::collections::BTreeMap<String, String> =
        std::collections::BTreeMap::new();
    for line in section.lines() {
        if !line.trim_start().starts_with('|') {
            continue;
        }
        // Split a Markdown table row into cells.
        let cells: Vec<&str> = line.split('|').map(str::trim).collect();
        if cells.len() < 3 {
            continue;
        }
        let id_cell = cells[1];
        let severity_cell = cells[2];
        let Some(id) = extract_backtick_id(id_cell) else {
            continue;
        };
        let severity = ["error", "warning", "info"]
            .into_iter()
            .find(|s| severity_cell.to_lowercase().contains(s));
        let Some(severity) = severity else { continue };
        documented.insert(id, severity.to_string());
    }

    let catalog_ids: std::collections::BTreeSet<String> =
        skillport::RULES.iter().map(|r| r.id.to_string()).collect();
    let documented_ids: std::collections::BTreeSet<String> = documented.keys().cloned().collect();

    assert_eq!(
        documented_ids, catalog_ids,
        "README '## Rule reference' table ids must exactly match the RULES catalog"
    );

    for rule in skillport::RULES {
        let expected = rule.severity.label();
        let got = documented
            .get(rule.id)
            .unwrap_or_else(|| panic!("README missing severity for '{}'", rule.id));
        assert_eq!(
            got, expected,
            "README severity for '{}' is '{got}', catalog default is '{expected}'",
            rule.id
        );
    }
}

/// Pull a `rule.id`-shaped token out of a backtick-wrapped Markdown cell,
/// e.g. "`` `name.charset` `` -> Some("name.charset")". Returns `None` if the
/// cell has no backtick-wrapped token.
fn extract_backtick_id(cell: &str) -> Option<String> {
    let start = cell.find('`')? + 1;
    let end = cell[start..].find('`')? + start;
    let token = &cell[start..end];
    if token.is_empty() {
        None
    } else {
        Some(token.to_string())
    }
}
