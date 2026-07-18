//! Target "format" profiles.
//!
//! All these platforms share the open Agent Skills SKILL.md standard
//! (agentskills.io). A profile therefore only needs to describe how a given
//! platform *diverges* from the portable core:
//!
//!   * `keep`          — which frontmatter fields it honors (others are dropped
//!                       on convert, with a warning, unless `--keep-all`).
//!   * `renames`       — field-name differences to normalize on the way in.
//!   * `path_template` — the install-directory convention for that platform.
//!
//! These are sensible, editable defaults reflecting the open spec plus each
//! platform's documented extensions — not a frozen authority. Tweak freely.

pub enum Keep {
    /// Pass every frontmatter field through untouched.
    #[allow(dead_code)]
    All,
    /// Keep only these fields (plus name/description, which are always kept).
    Only(&'static [&'static str]),
}

pub struct Profile {
    pub id: &'static str,
    pub title: &'static str,
    pub keep: Keep,
    /// (source_key, target_key): if the source uses `source_key`, rename it.
    pub renames: &'static [(&'static str, &'static str)],
    /// Install path relative to a destination root. `{slug}` is substituted.
    pub path_template: &'static str,
    pub notes: &'static str,
}

impl Profile {
    pub fn install_path(&self, slug: &str) -> String {
        self.path_template.replace("{slug}", slug)
    }
}

/// Fields Claude Code / claude.ai recognize beyond the portable core.
const CLAUDE_FIELDS: &[&str] = &[
    "name",
    "description",
    "license",
    "metadata",
    "allowed-tools",
    "argument-hint",
    "arguments",
    "model",
    "context",
    "effort",
    "hooks",
    "when_to_use",
    "disable-model-invocation",
];

const CURSOR_FIELDS: &[&str] = &[
    "name",
    "description",
    "license",
    "metadata",
    "when_to_use",
    "agents",
];

const CODEX_FIELDS: &[&str] = &[
    "name",
    "description",
    "license",
    "metadata",
    "when_to_use",
    "arguments",
];

const VERCEL_FIELDS: &[&str] = &["name", "description", "license", "metadata", "agents"];

/// The portable core: only what every compliant runtime is guaranteed to read.
const OPEN_FIELDS: &[&str] = &["name", "description", "license", "metadata"];

pub const PROFILES: &[Profile] = &[
    Profile {
        id: "open",
        title: "Open Agent Skills spec (portable)",
        keep: Keep::Only(OPEN_FIELDS),
        renames: &[("compatible_agents", "agents")],
        path_template: "skills/{slug}/SKILL.md",
        notes: "Portable core only. Publish anywhere; every compliant agent reads it.",
    },
    Profile {
        id: "claude",
        title: "Claude Code / claude.ai",
        keep: Keep::Only(CLAUDE_FIELDS),
        renames: &[("compatible_agents", "agents")],
        path_template: ".claude/skills/{slug}/SKILL.md",
        notes: "Drop into a repo's .claude/skills/ or zip for upload to claude.ai.",
    },
    Profile {
        id: "cursor",
        title: "Cursor",
        keep: Keep::Only(CURSOR_FIELDS),
        renames: &[("compatible_agents", "agents")],
        path_template: ".cursor/skills/{slug}/SKILL.md",
        notes: "Installs under .cursor/skills/ in the project root.",
    },
    Profile {
        id: "codex",
        title: "OpenAI Codex CLI",
        keep: Keep::Only(CODEX_FIELDS),
        renames: &[("compatible_agents", "agents")],
        path_template: ".codex/skills/{slug}/SKILL.md",
        notes: "Honors when_to_use for extended trigger guidance.",
    },
    Profile {
        id: "vercel",
        title: "Vercel skills (skills.sh)",
        keep: Keep::Only(VERCEL_FIELDS),
        renames: &[("compatible_agents", "agents")],
        path_template: "skills/{slug}/SKILL.md",
        notes: "Publishable package. Consumers install with `npx skills add`.",
    },
];

pub fn find(id: &str) -> Option<&'static Profile> {
    PROFILES.iter().find(|p| p.id == id)
}

pub fn ids() -> Vec<&'static str> {
    PROFILES.iter().map(|p| p.id).collect()
}
