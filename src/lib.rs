//! skillport — validate and audit agent `SKILL.md` files.
//!
//! Collection-first substrate (DEC-004): [`parse`] turns raw `SKILL.md` bytes
//! into a canonical [`skill::Skill`]; [`walk`] discovers a [`walk::Collection`];
//! [`lint_skill`] applies the open-spec rules; [`report::Report::from_collection`]
//! assembles a sectioned, path-sorted report; [`emit`] renders it (human /
//! `--json` / `--sarif`). The `skillport lint` binary (`main.rs`) wires these
//! together. (`--target` and the remaining rules are still to come.)

pub mod emit;
pub mod parse;
pub mod report;
pub mod rules;
pub mod skill;
pub mod walk;

pub use emit::{human, json, sarif};
pub use parse::parse;
pub use report::{Finding, Report, Section, Severity, Summary};
pub use rules::{lint_skill, lint_skill_with_target, Target};
pub use skill::{Frontmatter, FrontmatterStatus, Skill, YamlValue};
pub use walk::{walk, Collection, CollectionItem};
