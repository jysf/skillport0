//! Reading a SKILL.md off disk and splitting frontmatter from body.

use crate::skill::Skill;
use anyhow::{bail, Context, Result};
use serde_yaml::Mapping;
use std::fs;
use std::path::{Path, PathBuf};

/// Load a skill from either a `SKILL.md` file or a directory containing one.
pub fn load(path: &Path) -> Result<Skill> {
    let (md_path, source_dir) = resolve(path)?;
    let raw = fs::read_to_string(&md_path)
        .with_context(|| format!("reading {}", md_path.display()))?;

    let (fm_text, body) = split_frontmatter(&raw)
        .with_context(|| format!("parsing frontmatter in {}", md_path.display()))?;

    let frontmatter: Mapping = if fm_text.trim().is_empty() {
        Mapping::new()
    } else {
        serde_yaml::from_str(&fm_text).context("frontmatter is not valid YAML")?
    };

    Ok(Skill {
        frontmatter,
        body,
        source_dir,
    })
}

/// Accept a path to a file or a directory and figure out where the SKILL.md is.
fn resolve(path: &Path) -> Result<(PathBuf, Option<PathBuf>)> {
    if path.is_dir() {
        let candidate = path.join("SKILL.md");
        if candidate.is_file() {
            return Ok((candidate, Some(path.to_path_buf())));
        }
        bail!("no SKILL.md found in directory {}", path.display());
    }
    if path.is_file() {
        let dir = path.parent().map(Path::to_path_buf);
        return Ok((path.to_path_buf(), dir));
    }
    bail!("path not found: {}", path.display());
}

/// Split a `---`-delimited YAML frontmatter block from the Markdown body.
///
/// Lenient by design: leading blank lines and a UTF-8 BOM are tolerated, and a
/// file with no frontmatter is treated as an all-body document (empty
/// frontmatter) rather than an error.
fn split_frontmatter(raw: &str) -> Result<(String, String)> {
    let raw = raw.strip_prefix('\u{feff}').unwrap_or(raw);
    let lines: Vec<&str> = raw.split('\n').collect();

    // Skip leading blank lines to find the opening delimiter.
    let mut i = 0;
    while i < lines.len() && lines[i].trim().is_empty() {
        i += 1;
    }

    if i >= lines.len() || lines[i].trim_end() != "---" {
        // No frontmatter; the whole file is the body.
        return Ok((String::new(), raw.trim_start_matches('\n').to_string()));
    }

    let fm_start = i + 1;
    let mut j = fm_start;
    while j < lines.len() {
        let t = lines[j].trim_end();
        if t == "---" || t == "..." {
            break;
        }
        j += 1;
    }
    if j >= lines.len() {
        bail!("frontmatter opened with '---' but was never closed");
    }

    let fm_text = lines[fm_start..j].join("\n");
    let body = lines[(j + 1)..].join("\n");
    Ok((fm_text, body.trim_start_matches('\n').to_string()))
}
