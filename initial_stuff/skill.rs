//! The canonical, platform-neutral representation of a skill.
//!
//! Every supported "format" (Claude, Cursor, Codex, Vercel, the open
//! agentskills.io spec) is fundamentally the same artifact: a `SKILL.md`
//! file with a YAML frontmatter block followed by a Markdown body. The
//! differences are *which* frontmatter fields a platform honors and *where*
//! the file is installed. So we parse everything into one `Skill` and let the
//! emitter re-shape it per target profile — losing nothing along the way.

use serde_yaml::{Mapping, Value};
use std::path::PathBuf;

pub struct Skill {
    /// Full original frontmatter, order-preserving. Source of truth: nothing
    /// is discarded at parse time, only (optionally) at emit time.
    pub frontmatter: Mapping,
    /// Markdown body that follows the frontmatter block.
    pub body: String,
    /// Directory that contained the SKILL.md, if known. Used to carry along
    /// resource folders (scripts/, references/, assets/) during a convert.
    pub source_dir: Option<PathBuf>,
}

impl Skill {
    pub fn get_str(&self, key: &str) -> Option<String> {
        self.frontmatter
            .get(key)
            .and_then(Value::as_str)
            .map(str::to_string)
    }

    pub fn name(&self) -> Option<String> {
        self.get_str("name")
    }

    pub fn description(&self) -> Option<String> {
        self.get_str("description")
    }

    /// All frontmatter keys, in original order.
    pub fn keys(&self) -> Vec<String> {
        self.frontmatter
            .iter()
            .filter_map(|(k, _)| k.as_str().map(str::to_string))
            .collect()
    }

    /// A filesystem-safe slug derived from the skill name, used to name the
    /// output directory (`.claude/skills/<slug>/`).
    pub fn slug(&self) -> String {
        let raw = self.name().unwrap_or_else(|| "skill".to_string());
        let mut out = String::new();
        let mut prev_dash = false;
        for c in raw.chars() {
            if c.is_ascii_alphanumeric() {
                out.push(c.to_ascii_lowercase());
                prev_dash = false;
            } else if !prev_dash {
                out.push('-');
                prev_dash = true;
            }
        }
        let trimmed = out.trim_matches('-').to_string();
        if trimmed.is_empty() {
            "skill".to_string()
        } else {
            trimmed
        }
    }
}
