//! Turning a filesystem path into an ordered [`Collection`] of skills.
//!
//! [`walk`] discovers every `SKILL.md` beneath a root, parses each with
//! [`crate::parse`], and returns them **sorted by path** so the same tree
//! yields the same order on every run and OS (DEC-005). It is **total**: no
//! `Result`, no panic — a missing root is an empty collection, and a file
//! that isn't valid UTF-8 becomes a per-item [`CollectionItem::Unreadable`]
//! rather than aborting the run (DEC-004, DEC-005). This module depends only
//! on `parse`/`Skill` and std; rules, severities, and findings are later
//! specs' concern.

use crate::parse::parse;
use crate::skill::Skill;
use std::path::{Path, PathBuf};

/// Directory names never descended into, anywhere in the tree.
const IGNORED_DIRS: [&str; 3] = [".git", "node_modules", "target"];

/// The exact filename a directory walk looks for. Case-sensitive by design:
/// on case-sensitive filesystems (Linux CI) `skill.md`/`SKILL.MD` are simply
/// different files, and treating skillport's own matching as case-insensitive
/// would make behavior diverge between macOS/Windows and Linux. An explicitly
/// passed file path is always honored regardless of its name (see `walk`).
const SKILL_FILENAME: &str = "SKILL.md";

/// An ordered, deterministic set of what a walk discovered. Sorted by path.
#[derive(Debug, Clone)]
pub struct Collection {
    pub root: PathBuf,
    /// Sorted by item path, ascending, stable (DEC-005).
    pub items: Vec<CollectionItem>,
}

/// One discovered file's outcome.
#[derive(Debug, Clone)]
pub enum CollectionItem {
    /// A discovered `SKILL.md` that was read and parsed (parse is total, so
    /// this exists even when the skill's frontmatter is
    /// Missing/Unclosed/Invalid).
    Skill(Skill),
    /// The file was found but could not be read as UTF-8 text (I/O error,
    /// invalid UTF-8, ...). Captured, never fatal.
    Unreadable { path: PathBuf, error: String },
    /// A directory the walk tried to descend into but couldn't (e.g.
    /// permission denied) — the subtree under `path` was not checked.
    /// Never emitted for intentionally-ignored dirs (`.git`/`node_modules`/
    /// `target`); those stay silently skipped.
    UnreadableDir { path: PathBuf, error: String },
}

impl CollectionItem {
    /// The item's path, regardless of variant — the sort key.
    fn path(&self) -> &Path {
        match self {
            CollectionItem::Skill(skill) => &skill.path,
            CollectionItem::Unreadable { path, .. } => path,
            CollectionItem::UnreadableDir { path, .. } => path,
        }
    }
}

/// Discover every skill under `root` and return them as an ordered
/// [`Collection`].
///
/// - A path to a single file is a 1-item collection: the file is parsed
///   **regardless of its name** (the user pointed at it on purpose).
/// - A path to a directory is walked recursively; subtrees named `.git`,
///   `node_modules`, or `target` are skipped, and only files named exactly
///   `SKILL.md` are discovered. Directory symlinks are not followed, so a
///   self-referential symlink cannot cause an infinite walk.
/// - A missing root (neither a file nor a directory) yields an empty
///   `Collection` — a missing-path usage error is a later, CLI-layer concern.
///
/// Total: never returns `Err`, never panics.
pub fn walk(root: &Path) -> Collection {
    let mut items = Vec::new();
    let mut dir_errors: Vec<(PathBuf, String)> = Vec::new();

    // `symlink_metadata` never follows the final symlink component, so we can
    // tell a symlink from a real file/dir up front.
    if let Ok(meta) = std::fs::symlink_metadata(root) {
        if meta.is_file() {
            items.push(read_item(root.to_path_buf()));
        } else if meta.is_dir() {
            let mut paths = Vec::new();
            collect(root, &mut paths, &mut dir_errors);
            paths.sort();
            items.extend(paths.into_iter().map(read_item));
        }
        // A symlink directly at `root` (to a file or dir) - `is_file`/`is_dir`
        // on `symlink_metadata` are false for symlinks, so fall through to
        // resolved metadata for that one, explicit, user-provided root only.
        else if meta.file_type().is_symlink() {
            if let Ok(resolved) = std::fs::metadata(root) {
                if resolved.is_file() {
                    items.push(read_item(root.to_path_buf()));
                } else if resolved.is_dir() {
                    let mut paths = Vec::new();
                    collect(root, &mut paths, &mut dir_errors);
                    paths.sort();
                    items.extend(paths.into_iter().map(read_item));
                }
            }
        }
    }

    items.extend(
        dir_errors
            .into_iter()
            .map(|(path, error)| CollectionItem::UnreadableDir { path, error }),
    );

    items.sort_by(|a, b| a.path().cmp(b.path()));

    Collection {
        root: root.to_path_buf(),
        items,
    }
}

/// Recursively collect every `SKILL.md` path under `dir` into `out`,
/// skipping ignored directory names and not following directory symlinks. A
/// directory the walk actually tries to descend into but can't read is
/// recorded in `dir_errors` (not emitted for intentionally-ignored dirs,
/// which are never descended into in the first place).
fn collect(dir: &Path, out: &mut Vec<PathBuf>, dir_errors: &mut Vec<(PathBuf, String)>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) => {
            dir_errors.push((dir.to_path_buf(), format!("read error: {e}")));
            return;
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();

        // Use symlink_metadata so a symlinked directory is identified as a
        // symlink (and skipped) rather than followed as a directory - this
        // is what prevents symlink-loop infinite recursion.
        let Ok(meta) = std::fs::symlink_metadata(&path) else {
            continue;
        };

        if meta.file_type().is_symlink() {
            // Don't follow directory (or any) symlinks while recursing.
            continue;
        }

        if meta.is_dir() {
            let name = path.file_name().and_then(|n| n.to_str());
            if matches!(name, Some(n) if IGNORED_DIRS.contains(&n)) {
                continue;
            }
            collect(&path, out, dir_errors);
        } else if meta.is_file()
            && path.file_name().and_then(|n| n.to_str()) == Some(SKILL_FILENAME)
        {
            out.push(path);
        }
    }
}

/// Read and parse one file's `SKILL.md` bytes into a `CollectionItem`.
///
/// Uses `fs::read` + `String::from_utf8` (not `read_to_string`) so an I/O
/// error and an invalid-UTF-8 error both land on the same `Unreadable` path
/// with a distinguishing message.
fn read_item(path: PathBuf) -> CollectionItem {
    match std::fs::read(&path) {
        Ok(bytes) => match String::from_utf8(bytes) {
            Ok(raw) => CollectionItem::Skill(parse(path, &raw)),
            Err(e) => CollectionItem::Unreadable {
                path,
                error: format!("invalid UTF-8: {e}"),
            },
        },
        Err(e) => CollectionItem::Unreadable {
            path,
            error: format!("read error: {e}"),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skill::FrontmatterStatus;
    use std::fs;

    /// A hermetic, auto-cleaned temp directory for building test trees.
    struct TempTree {
        dir: tempfile::TempDir,
    }

    impl TempTree {
        fn new() -> Self {
            TempTree {
                dir: tempfile::tempdir().expect("create temp dir"),
            }
        }

        fn path(&self) -> &Path {
            self.dir.path()
        }

        /// Write a UTF-8 file at `rel` (creating parent dirs as needed).
        fn write(&self, rel: &str, contents: &str) -> PathBuf {
            let full = self.path().join(rel);
            if let Some(parent) = full.parent() {
                fs::create_dir_all(parent).expect("create parent dirs");
            }
            fs::write(&full, contents).expect("write file");
            full
        }

        /// Write raw (possibly non-UTF-8) bytes at `rel`.
        fn write_bytes(&self, rel: &str, bytes: &[u8]) -> PathBuf {
            let full = self.path().join(rel);
            if let Some(parent) = full.parent() {
                fs::create_dir_all(parent).expect("create parent dirs");
            }
            fs::write(&full, bytes).expect("write file");
            full
        }
    }

    const MINIMAL_SKILL: &str = "---\nname: foo\ndescription: d\n---\nbody\n";

    #[test]
    fn single_skill_md_file_is_one_skill_item() {
        let tree = TempTree::new();
        let path = tree.write("SKILL.md", MINIMAL_SKILL);

        let collection = walk(&path);

        assert_eq!(collection.items.len(), 1);
        match &collection.items[0] {
            CollectionItem::Skill(skill) => {
                assert_eq!(
                    skill.frontmatter.get("name").and_then(|v| v.as_str()),
                    Some("foo")
                );
            }
            other => panic!("expected Skill item, got {other:?}"),
        }
    }

    #[test]
    fn explicit_non_skill_filename_is_still_parsed() {
        let tree = TempTree::new();
        let path = tree.write("foo.md", MINIMAL_SKILL);

        let collection = walk(&path);

        assert_eq!(collection.items.len(), 1);
        assert!(matches!(collection.items[0], CollectionItem::Skill(_)));
    }

    #[test]
    fn recursive_discovery_finds_nested_skill_md() {
        let tree = TempTree::new();
        tree.write("a/SKILL.md", MINIMAL_SKILL);
        tree.write("b/c/SKILL.md", MINIMAL_SKILL);

        let collection = walk(tree.path());

        assert_eq!(collection.items.len(), 2);
        assert!(collection
            .items
            .iter()
            .all(|i| matches!(i, CollectionItem::Skill(_))));
    }

    #[test]
    fn items_are_sorted_by_path() {
        let tree = TempTree::new();
        // Names chosen so read_dir's natural (unsorted) order is unlikely to
        // already be ascending.
        tree.write("zzz/SKILL.md", MINIMAL_SKILL);
        tree.write("mmm/SKILL.md", MINIMAL_SKILL);
        tree.write("aaa/SKILL.md", MINIMAL_SKILL);
        tree.write("ccc/SKILL.md", MINIMAL_SKILL);

        let collection = walk(tree.path());

        let paths: Vec<&Path> = collection.items.iter().map(|i| i.path()).collect();
        let mut sorted = paths.clone();
        sorted.sort();
        assert_eq!(paths, sorted);
        assert_eq!(collection.items.len(), 4);
    }

    #[test]
    fn ignores_git_node_modules_target() {
        let tree = TempTree::new();
        tree.write(".git/SKILL.md", MINIMAL_SKILL);
        tree.write("node_modules/SKILL.md", MINIMAL_SKILL);
        tree.write("target/SKILL.md", MINIMAL_SKILL);
        tree.write("skill/SKILL.md", MINIMAL_SKILL);

        let collection = walk(tree.path());

        assert_eq!(collection.items.len(), 1);
        match &collection.items[0] {
            CollectionItem::Skill(skill) => {
                assert_eq!(skill.dir_name.as_deref(), Some("skill"));
            }
            other => panic!("expected Skill item, got {other:?}"),
        }
    }

    #[test]
    fn unreadable_non_utf8_file_becomes_unreadable_item_walk_continues() {
        let tree = TempTree::new();
        tree.write("good/SKILL.md", MINIMAL_SKILL);
        tree.write_bytes("bad/SKILL.md", &[0xFF, 0xFE, b'n', b'o', b'p', b'e']);

        let collection = walk(tree.path());

        assert_eq!(collection.items.len(), 2);
        let has_skill = collection
            .items
            .iter()
            .any(|i| matches!(i, CollectionItem::Skill(_)));
        let has_unreadable = collection
            .items
            .iter()
            .any(|i| matches!(i, CollectionItem::Unreadable { .. }));
        assert!(has_skill, "expected a Skill item among {collection:?}");
        assert!(
            has_unreadable,
            "expected an Unreadable item among {collection:?}"
        );
    }

    #[test]
    fn malformed_frontmatter_is_still_a_skill_item() {
        let tree = TempTree::new();
        tree.write("SKILL.md", "---\nname: x\n\n# body, no close");

        let collection = walk(tree.path());

        assert_eq!(collection.items.len(), 1);
        match &collection.items[0] {
            CollectionItem::Skill(skill) => {
                assert_eq!(skill.frontmatter_status, FrontmatterStatus::Unclosed);
            }
            other => panic!("expected Skill item, got {other:?}"),
        }
    }

    #[test]
    fn dir_name_is_the_parent_directory() {
        let tree = TempTree::new();
        tree.write("my-skill/SKILL.md", MINIMAL_SKILL);

        let collection = walk(tree.path());

        assert_eq!(collection.items.len(), 1);
        match &collection.items[0] {
            CollectionItem::Skill(skill) => {
                assert_eq!(skill.dir_name.as_deref(), Some("my-skill"));
            }
            other => panic!("expected Skill item, got {other:?}"),
        }
    }

    #[test]
    fn empty_dir_yields_empty_collection() {
        let tree = TempTree::new();

        let collection = walk(tree.path());

        assert!(collection.items.is_empty());
    }

    #[test]
    fn missing_path_yields_empty_collection() {
        let tree = TempTree::new();
        let missing = tree.path().join("does/not/exist");

        let collection = walk(&missing);

        assert!(collection.items.is_empty());
    }

    #[test]
    fn only_exact_skill_md_is_matched() {
        let tree = TempTree::new();
        tree.write("a/skill.md", MINIMAL_SKILL);
        tree.write("b/SKILL.MD", MINIMAL_SKILL);
        tree.write("c/SKILL.md~", MINIMAL_SKILL);

        let collection = walk(tree.path());

        // Case-sensitive, exact-name match only (documented on
        // `SKILL_FILENAME`): none of `skill.md`, `SKILL.MD`, `SKILL.md~`
        // qualify, even though `SKILL.MD` differs only by case.
        assert!(collection.items.is_empty());
    }

    #[test]
    fn no_symlink_loops() {
        let tree = TempTree::new();
        tree.write("real/SKILL.md", MINIMAL_SKILL);

        #[cfg(unix)]
        {
            let loop_path = tree.path().join("real/loop");
            std::os::unix::fs::symlink(tree.path(), &loop_path).expect("create symlink");

            let collection = walk(tree.path());

            // The walk must terminate and still find the one real skill; the
            // symlinked directory back to root is not followed.
            assert_eq!(collection.items.len(), 1);
        }

        #[cfg(not(unix))]
        {
            // No portable symlink-loop setup on this platform; just assert
            // the non-looped walk still works.
            let collection = walk(tree.path());
            assert_eq!(collection.items.len(), 1);
        }
    }

    #[test]
    fn walks_the_repo_lint_fixtures_good_tree() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("lint-fixtures/good");

        let collection = walk(&root);

        let found = collection.items.iter().any(|i| match i {
            CollectionItem::Skill(skill) => skill
                .path
                .to_string_lossy()
                .replace('\\', "/")
                .ends_with("data-analysis/SKILL.md"),
            CollectionItem::Unreadable { .. } | CollectionItem::UnreadableDir { .. } => false,
        });
        assert!(
            found,
            "expected data-analysis/SKILL.md among {collection:?}"
        );
    }

    /// Guard that `chmod`s a directory back to a readable mode on drop, so a
    /// locked-down temp dir can still be cleaned up even if an assertion
    /// panics mid-test.
    #[cfg(unix)]
    struct RestorePerms(PathBuf);

    #[cfg(unix)]
    impl Drop for RestorePerms {
        fn drop(&mut self) {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&self.0, std::fs::Permissions::from_mode(0o755));
        }
    }

    #[cfg(unix)]
    #[test]
    fn unreadable_subdir_becomes_unreadable_dir_siblings_still_found() {
        use std::os::unix::fs::PermissionsExt;

        let tree = TempTree::new();
        tree.write("good/SKILL.md", MINIMAL_SKILL);
        let locked = tree.path().join("locked");
        fs::create_dir_all(&locked).expect("create locked dir");
        tree.write("locked/SKILL.md", MINIMAL_SKILL);

        fs::set_permissions(&locked, fs::Permissions::from_mode(0o000))
            .expect("chmod 000 locked dir");
        let _restore = RestorePerms(locked.clone());

        let collection = walk(tree.path());

        let has_good_skill = collection.items.iter().any(|i| match i {
            CollectionItem::Skill(skill) => skill.dir_name.as_deref() == Some("good"),
            _ => false,
        });
        assert!(
            has_good_skill,
            "expected the good skill among {collection:?}"
        );

        let has_unreadable_dir = collection
            .items
            .iter()
            .any(|i| matches!(i, CollectionItem::UnreadableDir { path, .. } if path == &locked));
        assert!(
            has_unreadable_dir,
            "expected an UnreadableDir for locked/ among {collection:?}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn items_including_unreadable_dir_are_path_sorted() {
        use std::os::unix::fs::PermissionsExt;

        let tree = TempTree::new();
        tree.write("zzz/SKILL.md", MINIMAL_SKILL);
        tree.write("aaa/SKILL.md", MINIMAL_SKILL);
        let locked = tree.path().join("mmm_locked");
        fs::create_dir_all(&locked).expect("create locked dir");

        fs::set_permissions(&locked, fs::Permissions::from_mode(0o000))
            .expect("chmod 000 locked dir");
        let _restore = RestorePerms(locked.clone());

        let collection = walk(tree.path());

        let paths: Vec<&Path> = collection.items.iter().map(|i| i.path()).collect();
        let mut sorted = paths.clone();
        sorted.sort();
        assert_eq!(paths, sorted);
        assert!(collection
            .items
            .iter()
            .any(|i| matches!(i, CollectionItem::UnreadableDir { .. })));
    }
}
