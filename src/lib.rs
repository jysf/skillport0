//! skillport — validate and audit agent `SKILL.md` files.
//!
//! This crate is the collection-first substrate (DEC-004): a tolerant, lossless
//! parser turns raw `SKILL.md` bytes into a canonical [`skill::Skill`]. Later
//! specs add the walker, rule engine, report, and CLI on top; nothing here
//! judges a skill or emits findings.

pub mod parse;
pub mod report;
pub mod skill;
pub mod walk;

pub use parse::parse;
pub use report::{Finding, Report, Section, Severity, Summary};
pub use skill::{Frontmatter, FrontmatterStatus, Skill, YamlValue};
pub use walk::{walk, Collection, CollectionItem};
