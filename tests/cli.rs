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

fn string(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
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
