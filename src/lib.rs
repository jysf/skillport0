//! skillport — validate and audit agent `SKILL.md` files.
//!
//! This crate is the collection-first substrate (DEC-004): a tolerant, lossless
//! parser turns raw `SKILL.md` bytes into a canonical [`skill::Skill`]. Later
//! specs add the walker, rule engine, report, and CLI on top; nothing here
//! judges a skill or emits findings.

pub mod parse;
pub mod skill;

pub use parse::parse;
pub use skill::{Frontmatter, FrontmatterStatus, Skill, YamlValue};
