//! Turning the raw bytes of a `SKILL.md` into a canonical [`Skill`].
//!
//! [`parse`] is a **total** function of `(path, raw)`: it never returns a
//! `Result` and never panics. Every tolerated or malformed case (BOM, CRLF,
//! blank lines, missing / unclosed / invalid frontmatter) is captured as a
//! [`FrontmatterStatus`] on the returned [`Skill`], so a bulk run never aborts
//! on one bad file (DEC-005). It does no filesystem I/O — the walker (a later
//! spec) maps this over discovered files (DEC-004, collection-first).

use crate::skill::{Frontmatter, FrontmatterStatus, Skill, YamlValue};
use std::path::{Path, PathBuf};

/// A UTF-8 byte-order mark. Stripped for frontmatter *detection* only; `raw`
/// keeps it (losslessness).
const BOM: char = '\u{feff}';

/// One source line within the (BOM-stripped) content.
///
/// `text` excludes the trailing `\n` but may still end with `\r` (CRLF); `end`
/// is the byte offset of the start of the next line, i.e. just past this line's
/// terminator, so `content[end..]` is everything after this line.
struct Line<'a> {
    end: usize,
    text: &'a str,
}

/// Parse the raw contents of a `SKILL.md` (plus its path) into a [`Skill`].
///
/// Total: never errors, never panics. See the module docs.
pub fn parse(path: PathBuf, raw: &str) -> Skill {
    let dir_name = path
        .parent()
        .and_then(Path::file_name)
        .map(|s| s.to_string_lossy().into_owned())
        // A bare "SKILL.md" has a parent of "" — no directory name.
        .filter(|s| !s.is_empty());

    // Detect on the BOM-stripped content, but keep `raw` untouched.
    let content = raw.strip_prefix(BOM).unwrap_or(raw);
    let lines = split_lines(content);

    let (frontmatter, body, frontmatter_status) = split(content, &lines);

    Skill {
        path,
        dir_name,
        frontmatter,
        body,
        raw: raw.to_string(),
        frontmatter_status,
    }
}

/// Split `content` into lines with byte offsets, preserving everything (an empty
/// input yields no lines).
fn split_lines(content: &str) -> Vec<Line<'_>> {
    let mut lines = Vec::new();
    let mut start = 0;
    for segment in content.split_inclusive('\n') {
        let end = start + segment.len();
        let text = segment.strip_suffix('\n').unwrap_or(segment);
        lines.push(Line { end, text });
        start = end;
    }
    lines
}

/// A delimiter line is exactly `---` at column 0, after trimming a trailing
/// `\r` (CRLF) and trailing spaces. We match on whole lines, so CRLF and blank
/// lines are cheap and losslessness is preserved.
fn is_fence(text: &str) -> bool {
    text.trim_end() == "---"
}

/// Compute `(frontmatter, body, status)` from the BOM-stripped content.
fn split(content: &str, lines: &[Line<'_>]) -> (Frontmatter, String, FrontmatterStatus) {
    // Skip leading blank lines to find the opening fence.
    let opening = lines.iter().position(|l| !l.text.trim().is_empty());

    let open_idx = match opening {
        Some(i) if is_fence(lines[i].text) => i,
        // No opening fence (including an empty or all-blank file): the whole
        // content (after BOM) is the body.
        _ => {
            return (
                Frontmatter::new(),
                content.to_string(),
                FrontmatterStatus::Missing,
            );
        }
    };

    // Find the closing fence after the opening one. Only `---` closes a block;
    // `...` is deliberately not recognized (out of scope for this spec).
    let close_idx = lines[open_idx + 1..]
        .iter()
        .position(|l| is_fence(l.text))
        .map(|rel| open_idx + 1 + rel);

    let close_idx = match close_idx {
        Some(j) => j,
        // Opening fence, never closed: no frontmatter, empty body, no panic.
        None => {
            return (
                Frontmatter::new(),
                String::new(),
                FrontmatterStatus::Unclosed,
            );
        }
    };

    // Body is everything after the closing fence line's terminator; the single
    // newline right after the closing fence is consumed, so the body does not
    // start with a stray blank line. Sliced verbatim (CRLF preserved).
    let body = content[lines[close_idx].end..].to_string();

    // Build the YAML text from the block's lines, normalizing CRLF -> LF for the
    // parser only. `raw`/`body` are untouched, so this does not affect
    // losslessness — it just makes CRLF frontmatter parse like LF.
    let fm_text = lines[open_idx + 1..close_idx]
        .iter()
        .map(|l| l.text.trim_end_matches('\r'))
        .collect::<Vec<_>>()
        .join("\n");

    let (frontmatter, status) = parse_frontmatter(&fm_text);
    (frontmatter, body, status)
}

/// Parse the YAML text of a (closed) frontmatter block into an order-preserving
/// map. An empty/whitespace-only block is a `Present` skill with no keys; a
/// non-mapping root or a YAML error is `Invalid(msg)` — never a panic (DEC-005).
fn parse_frontmatter(text: &str) -> (Frontmatter, FrontmatterStatus) {
    if text.trim().is_empty() {
        return (Frontmatter::new(), FrontmatterStatus::Present);
    }

    match serde_yaml_ng::from_str::<YamlValue>(text) {
        Ok(YamlValue::Mapping(map)) => {
            let mut frontmatter = Frontmatter::new();
            for (key, value) in map {
                match key.as_str() {
                    Some(k) => {
                        // Preserve source order; last write wins on a dup key
                        // (a `key.duplicate` rule is STAGE-002's concern).
                        frontmatter.insert(k.to_string(), value);
                    }
                    None => {
                        return (
                            Frontmatter::new(),
                            FrontmatterStatus::Invalid(
                                "frontmatter has a non-string key".to_string(),
                            ),
                        );
                    }
                }
            }
            (frontmatter, FrontmatterStatus::Present)
        }
        Ok(other) => (
            Frontmatter::new(),
            FrontmatterStatus::Invalid(format!(
                "frontmatter root is not a mapping (found {})",
                yaml_kind(&other)
            )),
        ),
        Err(e) => (
            Frontmatter::new(),
            FrontmatterStatus::Invalid(e.to_string()),
        ),
    }
}

/// A short, stable name for a YAML value's shape, for `Invalid` messages.
fn yaml_kind(value: &YamlValue) -> &'static str {
    match value {
        YamlValue::Null => "null",
        YamlValue::Bool(_) => "boolean",
        YamlValue::Number(_) => "number",
        YamlValue::String(_) => "string",
        YamlValue::Sequence(_) => "sequence",
        YamlValue::Mapping(_) => "mapping",
        YamlValue::Tagged(_) => "tagged value",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_str(raw: &str) -> Skill {
        parse(PathBuf::from("SKILL.md"), raw)
    }

    #[test]
    fn wellformed_splits_frontmatter_and_body() {
        let input = "---\nname: foo\n---\n# Body\n\ntext\n";
        let skill = parse_str(input);
        assert_eq!(skill.frontmatter_status, FrontmatterStatus::Present);
        assert_eq!(
            skill.frontmatter.get("name").and_then(YamlValue::as_str),
            Some("foo")
        );
        assert!(skill.body.starts_with("# Body"));
    }

    #[test]
    fn frontmatter_key_order_is_preserved() {
        let input = "---\nname: x\ndescription: d\nlicense: MIT\n---\nbody";
        let skill = parse_str(input);
        assert_eq!(skill.frontmatter_status, FrontmatterStatus::Present);
        let keys: Vec<&str> = skill.keys().map(String::as_str).collect();
        assert_eq!(keys, ["name", "description", "license"]);
    }

    #[test]
    fn typed_values_are_distinguishable() {
        let input =
            "---\nname: n\nallowed-tools: [Bash, Read]\nmetadata:\n  author: acme\n---\nbody";
        let skill = parse_str(input);
        assert_eq!(skill.frontmatter_status, FrontmatterStatus::Present);

        // `name` is a scalar string.
        assert!(matches!(
            skill.frontmatter.get("name"),
            Some(YamlValue::String(_))
        ));
        // `allowed-tools` given as a YAML list is a sequence, not a string.
        assert!(matches!(
            skill.frontmatter.get("allowed-tools"),
            Some(YamlValue::Sequence(_))
        ));
        // `metadata` given as a map is a mapping.
        assert!(matches!(
            skill.frontmatter.get("metadata"),
            Some(YamlValue::Mapping(_))
        ));
    }

    #[test]
    fn lossless_raw_equals_input() {
        // BOM + CRLF sample: raw must be byte-for-byte the input.
        let input = "\u{feff}---\r\nname: x\r\n---\r\n# body\r\n";
        let skill = parse_str(input);
        assert_eq!(skill.raw, input);
    }

    #[test]
    fn strips_a_utf8_bom_for_detection() {
        let input = "\u{feff}---\nname: x\n---\nbody";
        let skill = parse_str(input);
        assert_eq!(skill.frontmatter_status, FrontmatterStatus::Present);
        assert_eq!(
            skill.frontmatter.get("name").and_then(YamlValue::as_str),
            Some("x")
        );
    }

    #[test]
    fn leading_blank_lines_before_frontmatter() {
        let input = "\n\n---\nname: x\n---\nbody";
        let skill = parse_str(input);
        assert_eq!(skill.frontmatter_status, FrontmatterStatus::Present);
    }

    #[test]
    fn crlf_endings_parse_like_lf() {
        let lf = parse_str("---\nname: x\ndescription: d\n---\nbody");
        let crlf = parse_str("---\r\nname: x\r\ndescription: d\r\n---\r\nbody");
        assert_eq!(crlf.frontmatter_status, FrontmatterStatus::Present);
        let lf_keys: Vec<&str> = lf.keys().map(String::as_str).collect();
        let crlf_keys: Vec<&str> = crlf.keys().map(String::as_str).collect();
        assert_eq!(lf_keys, crlf_keys);
        assert_eq!(
            crlf.frontmatter.get("name").and_then(YamlValue::as_str),
            Some("x")
        );
    }

    #[test]
    fn missing_frontmatter_is_full_body() {
        let input = "# Just markdown\n";
        let skill = parse_str(input);
        assert_eq!(skill.frontmatter_status, FrontmatterStatus::Missing);
        assert!(skill.frontmatter.is_empty());
        assert_eq!(skill.body, "# Just markdown\n");
    }

    #[test]
    fn unclosed_frontmatter_is_unclosed() {
        let input = "---\nname: x\n\n# body, no close";
        let skill = parse_str(input);
        assert_eq!(skill.frontmatter_status, FrontmatterStatus::Unclosed);
        assert!(skill.frontmatter.is_empty());
    }

    #[test]
    fn invalid_yaml_still_separates_body() {
        let input = "---\nname: [oops\n---\n# b\n";
        let skill = parse_str(input);
        assert!(matches!(
            skill.frontmatter_status,
            FrontmatterStatus::Invalid(_)
        ));
        assert!(skill.frontmatter.is_empty());
        assert_eq!(skill.body, "# b\n");
    }

    #[test]
    fn non_mapping_root_is_invalid() {
        let input = "---\n- a\n- b\n---\nbody";
        let skill = parse_str(input);
        assert!(matches!(
            skill.frontmatter_status,
            FrontmatterStatus::Invalid(_)
        ));
        assert!(skill.frontmatter.is_empty());
    }

    #[test]
    fn empty_file_is_missing_empty_body() {
        let skill = parse_str("");
        assert_eq!(skill.frontmatter_status, FrontmatterStatus::Missing);
        assert_eq!(skill.body, "");
        assert!(skill.frontmatter.is_empty());
    }

    #[test]
    fn output_is_deterministic() {
        let input = "---\nname: x\ndescription: d\nlicense: MIT\n---\nbody";
        let a = parse_str(input);
        let b = parse_str(input);
        let a_keys: Vec<&str> = a.keys().map(String::as_str).collect();
        let b_keys: Vec<&str> = b.keys().map(String::as_str).collect();
        assert_eq!(a_keys, b_keys);
        assert_eq!(a.raw, b.raw);
        assert_eq!(a.body, b.body);
        assert_eq!(a.frontmatter_status, b.frontmatter_status);
    }

    #[test]
    fn good_fixtures_parse_present_with_expected_keys() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("lint-fixtures/good");
        let mut checked = 0;
        for path in collect_skill_files(&root) {
            let raw = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("reading {}: {e}", path.display()));
            let skill = parse(path.clone(), &raw);
            assert_eq!(
                skill.frontmatter_status,
                FrontmatterStatus::Present,
                "{} should parse Present",
                path.display()
            );
            assert!(
                skill.frontmatter.contains_key("name"),
                "{} missing `name`",
                path.display()
            );
            assert!(
                skill.frontmatter.contains_key("description"),
                "{} missing `description`",
                path.display()
            );
            checked += 1;
        }
        assert!(
            checked > 0,
            "no good fixtures found under {}",
            root.display()
        );
    }

    /// Recursively collect every `SKILL.md` under `dir` (test helper; the real
    /// walker is a later spec). Deterministically ordered by path.
    fn collect_skill_files(dir: &Path) -> Vec<PathBuf> {
        let mut out = Vec::new();
        let mut stack = vec![dir.to_path_buf()];
        while let Some(d) = stack.pop() {
            let Ok(entries) = std::fs::read_dir(&d) else {
                continue;
            };
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    stack.push(p);
                } else if p.file_name().and_then(|n| n.to_str()) == Some("SKILL.md") {
                    out.push(p);
                }
            }
        }
        out.sort();
        out
    }
}
