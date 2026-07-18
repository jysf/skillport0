//! Rendering a `Skill` back out for a target profile, and writing it (plus any
//! resource folders) into the correct directory layout.

use crate::profiles::{Keep, Profile};
use crate::skill::Skill;
use anyhow::{Context, Result};
use serde_yaml::{Mapping, Value};
use std::fs;
use std::path::{Path, PathBuf};

/// Conventional resource subdirectories carried along on a convert.
const RESOURCE_DIRS: &[&str] = &["scripts", "references", "assets"];

pub struct Rendered {
    pub content: String,
    /// Frontmatter keys that were present in the source but not honored by the
    /// target profile (empty when `--keep-all` is used or nothing was lost).
    pub dropped: Vec<String>,
}

/// Build the target-flavored SKILL.md text.
pub fn render(skill: &Skill, profile: &Profile, keep_all: bool) -> Rendered {
    let (frontmatter, dropped) = shape_frontmatter(skill, profile, keep_all);

    let mut content = String::new();
    if !frontmatter.is_empty() {
        let yaml = serde_yaml::to_string(&frontmatter)
            .unwrap_or_else(|_| String::from("# <failed to serialize frontmatter>\n"));
        content.push_str("---\n");
        content.push_str(&yaml);
        if !yaml.ends_with('\n') {
            content.push('\n');
        }
        content.push_str("---\n\n");
    }
    content.push_str(skill.body.trim_end());
    content.push('\n');

    Rendered { content, dropped }
}

/// Apply renames, then filter by the profile's keep-list.
fn shape_frontmatter(skill: &Skill, profile: &Profile, keep_all: bool) -> (Mapping, Vec<String>) {
    // 1. Normalize field names.
    let mut work = Mapping::new();
    for (k, v) in &skill.frontmatter {
        let key = k.as_str().unwrap_or_default();
        let target = profile
            .renames
            .iter()
            .find(|(from, _)| *from == key)
            .map(|(_, to)| *to)
            .unwrap_or(key);
        if !work.contains_key(target) {
            work.insert(Value::from(target), v.clone());
        }
    }

    // 2. Filter, keeping name + description pinned to the front.
    let mut out = Mapping::new();
    for req in ["name", "description"] {
        if let Some(v) = work.get(req) {
            out.insert(Value::from(req), v.clone());
        }
    }

    let mut dropped = Vec::new();
    for (k, v) in &work {
        let key = k.as_str().unwrap_or_default();
        if key == "name" || key == "description" {
            continue;
        }
        let keep = keep_all
            || match profile.keep {
                Keep::All => true,
                Keep::Only(list) => list.contains(&key),
            };
        if keep {
            out.insert(k.clone(), v.clone());
        } else {
            dropped.push(key.to_string());
        }
    }

    (out, dropped)
}

/// Write the rendered skill into `root` using the profile's path template,
/// copying conventional resource folders alongside it. Returns the SKILL.md path.
pub fn write(skill: &Skill, profile: &Profile, rendered: &Rendered, root: &Path) -> Result<PathBuf> {
    let rel = profile.install_path(&skill.slug());
    let md_path = root.join(&rel);
    let skill_dir = md_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| root.to_path_buf());

    fs::create_dir_all(&skill_dir)
        .with_context(|| format!("creating {}", skill_dir.display()))?;
    fs::write(&md_path, &rendered.content)
        .with_context(|| format!("writing {}", md_path.display()))?;

    if let Some(src) = &skill.source_dir {
        for dir in RESOURCE_DIRS {
            let from = src.join(dir);
            if from.is_dir() {
                copy_dir(&from, &skill_dir.join(dir))
                    .with_context(|| format!("copying {} resources", dir))?;
            }
        }
    }

    Ok(md_path)
}

fn copy_dir(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let target = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}
