//! The canonical, lossless, order-preserving representation of a skill.
//!
//! A `SKILL.md` is a YAML frontmatter block followed by a Markdown body. The
//! parser (`crate::parse`) turns raw file bytes into this model *tolerantly*
//! (BOM / CRLF / blank lines / missing or malformed frontmatter never crash)
//! and *losslessly* (`raw` is byte-for-byte the input; frontmatter key order is
//! preserved). It reports structural facts only — it never judges the skill;
//! severities and rule ids belong to the rule engine (STAGE-002). See DEC-004
//! (collection-first, order-preserving/lossless) and DEC-002 (keep frontmatter
//! typed so later rules can inspect value shapes).

use std::path::PathBuf;

/// A typed YAML value. Kept typed (not stringified) so rules can distinguish a
/// string from a sequence from a mapping (e.g. `allowed-tools` as a list vs. a
/// string, `metadata` as a map) — DEC-002.
pub type YamlValue = serde_yaml_ng::Value;

/// Order-preserving map of frontmatter key -> typed YAML value.
///
/// Insertion order is the source order of the keys in the file (DEC-004), so
/// this is an index-map, never a `HashMap`.
pub type Frontmatter = indexmap::IndexMap<String, YamlValue>;

/// The outcome of trying to split and parse the YAML frontmatter block. A
/// malformed skill is a *status*, never a panic or an aborting error (DEC-005).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrontmatterStatus {
    /// Opening + closing fence, and a valid YAML mapping (or an empty block).
    Present,
    /// No opening fence — the whole file is body.
    Missing,
    /// Opening fence, but no closing fence.
    Unclosed,
    /// Fenced block present but the YAML failed to parse or its root is not a
    /// mapping (e.g. a list or scalar). The message explains why.
    Invalid(String),
}

/// The canonical model of one parsed `SKILL.md`.
#[derive(Debug, Clone)]
pub struct Skill {
    /// Path to the `SKILL.md`. The ordering key for the collection.
    pub path: PathBuf,
    /// Parent directory name, for the later `name.dir-match` rule.
    pub dir_name: Option<String>,
    /// Parsed frontmatter, order-preserving. Empty unless `frontmatter_status`
    /// is `Present`.
    pub frontmatter: Frontmatter,
    /// The Markdown body after the frontmatter block, verbatim.
    pub body: String,
    /// The original file content, byte-for-byte (losslessness — DEC-004).
    pub raw: String,
    /// What happened when splitting/parsing the frontmatter.
    pub frontmatter_status: FrontmatterStatus,
}

impl Skill {
    /// A frontmatter value by key, if present.
    pub fn get(&self, key: &str) -> Option<&YamlValue> {
        self.frontmatter.get(key)
    }

    /// Frontmatter keys, in source order.
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.frontmatter.keys()
    }
}
