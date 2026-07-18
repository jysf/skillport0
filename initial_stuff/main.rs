//! skillport — convert and sync agent SKILL.md files between platform formats.

mod emit;
mod lint;
mod parse;
mod profiles;
mod skill;

use anyhow::{anyhow, bail, Result};
use clap::{Parser, Subcommand};
use lint::{Finding, Severity};
use skill::Skill;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(
    name = "skillport",
    version,
    about = "Convert and sync agent SKILL.md files between platform formats.",
    long_about = "All supported platforms share the open Agent Skills (SKILL.md) standard.\n\
                  skillport parses a skill once, then re-emits it for a target platform:\n\
                  adjusting the frontmatter it honors and writing it into that platform's\n\
                  directory convention (.claude/skills/, .cursor/skills/, ...)."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Show the parsed skill: name, description, fields, resources.
    Inspect {
        /// Path to a SKILL.md file or a folder containing one.
        path: PathBuf,
    },
    /// Convert a skill and write it under an output directory.
    Convert {
        /// Path to a SKILL.md file or a folder containing one.
        path: PathBuf,
        /// Target platform.
        #[arg(short, long, value_name = "PROFILE")]
        to: String,
        /// Output root directory (default: ./out).
        #[arg(short, long, default_value = "out")]
        out: PathBuf,
        /// Keep every frontmatter field instead of the target's honored set.
        #[arg(long)]
        keep_all: bool,
        /// Print the converted SKILL.md to stdout instead of writing files.
        #[arg(long)]
        stdout: bool,
    },
    /// Sync a skill directly into a project (its platform install folder).
    Push {
        /// Path to a SKILL.md file or a folder containing one.
        path: PathBuf,
        /// Target platform.
        #[arg(short, long, value_name = "PROFILE")]
        to: String,
        /// Destination project root that will receive .claude/, .cursor/, etc.
        #[arg(short, long, default_value = ".")]
        dest: PathBuf,
        /// Keep every frontmatter field instead of the target's honored set.
        #[arg(long)]
        keep_all: bool,
    },
    /// Validate skill(s) against the open spec (and optionally a platform).
    Lint {
        /// A SKILL.md, a skill folder, or a directory tree of skills.
        path: PathBuf,
        /// Also apply a platform's recognized-field set.
        #[arg(short, long, value_name = "PROFILE")]
        target: Option<String>,
        /// Emit machine-readable JSON instead of text.
        #[arg(long)]
        json: bool,
        /// Treat warnings as failures (non-zero exit).
        #[arg(long)]
        strict: bool,
    },
    /// List the available target platform profiles.
    Profiles,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    match Cli::parse().command {
        Command::Inspect { path } => inspect(&path),
        Command::Convert {
            path,
            to,
            out,
            keep_all,
            stdout,
        } => convert(&path, &to, &out, keep_all, stdout),
        Command::Push {
            path,
            to,
            dest,
            keep_all,
        } => convert(&path, &to, &dest, keep_all, false),
        Command::Lint {
            path,
            target,
            json,
            strict,
        } => lint_cmd(&path, target.as_deref(), json, strict),
        Command::Profiles => {
            list_profiles();
            Ok(())
        }
    }
}

fn inspect(path: &Path) -> Result<()> {
    let s = parse::load(path)?;
    let detected = detect_source(&s, path);

    println!("name:        {}", s.name().unwrap_or_else(|| "(missing)".into()));
    println!(
        "description: {}",
        s.description().unwrap_or_else(|| "(missing)".into())
    );
    println!("likely source format: {detected}");
    println!("frontmatter fields: {}", join(&s.keys()));
    println!("body: {} chars", s.body.chars().count());

    if let Some(dir) = &s.source_dir {
        let mut found = Vec::new();
        for r in ["scripts", "references", "assets"] {
            if dir.join(r).is_dir() {
                found.push(r.to_string());
            }
        }
        println!(
            "resource folders: {}",
            if found.is_empty() { "none".into() } else { join(&found) }
        );
    }
    Ok(())
}

fn convert(path: &Path, to: &str, root: &Path, keep_all: bool, to_stdout: bool) -> Result<()> {
    let profile = match profiles::find(to) {
        Some(p) => p,
        None => bail!(
            "unknown target '{to}'. Available: {}",
            profiles::ids().join(", ")
        ),
    };

    let s = parse::load(path)?;
    if s.name().is_none() {
        eprintln!("warning: skill has no 'name' field; using slug '{}'", s.slug());
    }
    if s.description().is_none() {
        eprintln!("warning: skill has no 'description' field (required by the spec)");
    }

    let rendered = emit::render(&s, profile, keep_all);

    if to_stdout {
        print!("{}", rendered.content);
        return Ok(());
    }

    let detected = detect_source(&s, path);
    let md_path = emit::write(&s, profile, &rendered, root)?;

    println!("{detected} -> {} ({})", profile.id, profile.title);
    println!("wrote {}", md_path.display());
    if !rendered.dropped.is_empty() {
        eprintln!(
            "note: dropped {} field(s) not honored by '{}': {} (use --keep-all to preserve)",
            rendered.dropped.len(),
            profile.id,
            join(&rendered.dropped)
        );
    }
    Ok(())
}

fn list_profiles() {
    println!("Available target profiles:\n");
    for p in profiles::PROFILES {
        println!("  {:<8} {}", p.id, p.title);
        println!("           install: {}", p.install_path("<name>"));
        println!("           {}\n", p.notes);
    }
}

/// One skill's lint result.
struct Report {
    path: PathBuf,
    findings: Vec<Finding>,
    parse_error: Option<String>,
}

fn lint_cmd(path: &Path, target: Option<&str>, json: bool, strict: bool) -> Result<()> {
    let profile = match target {
        Some(id) => Some(
            profiles::find(id)
                .ok_or_else(|| anyhow!("unknown target '{id}'. Available: {}", profiles::ids().join(", ")))?,
        ),
        None => None,
    };

    let targets = collect_targets(path)?;
    let mut reports = Vec::new();
    for md in targets {
        let dir_name = md
            .parent()
            .and_then(Path::file_name)
            .and_then(|s| s.to_str())
            .map(str::to_string);
        match parse::load(&md) {
            Ok(skill) => {
                let findings = lint::lint(&skill, dir_name.as_deref(), profile);
                reports.push(Report {
                    path: md,
                    findings,
                    parse_error: None,
                });
            }
            Err(e) => reports.push(Report {
                path: md,
                findings: Vec::new(),
                parse_error: Some(format!("{e:#}")),
            }),
        }
    }

    if json {
        print_json(&reports);
    } else {
        print_human(&reports);
    }

    let fail = reports.iter().any(|r| {
        r.parse_error.is_some()
            || r.findings.iter().any(|f| {
                matches!(f.severity, Severity::Error)
                    || (strict && matches!(f.severity, Severity::Warning))
            })
    });
    std::process::exit(if fail { 1 } else { 0 });
}

/// Resolve a lint target into the list of SKILL.md files to check.
fn collect_targets(path: &Path) -> Result<Vec<PathBuf>> {
    if path.is_file() {
        return Ok(vec![path.to_path_buf()]);
    }
    if path.is_dir() {
        let direct = path.join("SKILL.md");
        if direct.is_file() {
            return Ok(vec![direct]);
        }
        let mut out = Vec::new();
        walk_skills(path, &mut out);
        if out.is_empty() {
            bail!("no SKILL.md found under {}", path.display());
        }
        out.sort();
        return Ok(out);
    }
    bail!("path not found: {}", path.display());
}

fn walk_skills(dir: &Path, out: &mut Vec<PathBuf>) {
    const SKIP: &[&str] = &[".git", "node_modules", "target"];
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let Ok(ft) = entry.file_type() else { continue };
        if ft.is_dir() {
            let name = entry.file_name();
            if SKIP.contains(&name.to_string_lossy().as_ref()) {
                continue;
            }
            walk_skills(&entry.path(), out);
        } else if ft.is_file() && entry.file_name() == "SKILL.md" {
            out.push(entry.path());
        }
    }
}

fn print_human(reports: &[Report]) {
    let (mut errors, mut warnings, mut infos) = (0u32, 0u32, 0u32);
    for r in reports {
        println!("{}", r.path.display());
        if let Some(e) = &r.parse_error {
            println!("  {:<5}  {:<24}  {e}", "error", "parse");
            errors += 1;
            println!();
            continue;
        }
        if r.findings.is_empty() {
            println!("  ok");
        }
        for f in &r.findings {
            println!("  {:<5}  {:<24}  {}", f.severity.label(), f.rule, f.message);
            match f.severity {
                Severity::Error => errors += 1,
                Severity::Warning => warnings += 1,
                Severity::Info => infos += 1,
            }
        }
        println!();
    }
    println!(
        "{} skill(s): {} error(s), {} warning(s), {} info",
        reports.len(),
        errors,
        warnings,
        infos
    );
}

fn print_json(reports: &[Report]) {
    let mut s = String::from("[");
    for (i, r) in reports.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push_str("{\"path\":");
        s.push_str(&json_str(&r.path.display().to_string()));
        if let Some(e) = &r.parse_error {
            s.push_str(",\"parse_error\":");
            s.push_str(&json_str(e));
        }
        s.push_str(",\"findings\":[");
        for (j, f) in r.findings.iter().enumerate() {
            if j > 0 {
                s.push(',');
            }
            s.push_str("{\"severity\":");
            s.push_str(&json_str(f.severity.label()));
            s.push_str(",\"rule\":");
            s.push_str(&json_str(f.rule));
            s.push_str(",\"message\":");
            s.push_str(&json_str(&f.message));
            s.push('}');
        }
        s.push_str("]}");
    }
    s.push(']');
    println!("{s}");
}

fn json_str(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Best-effort guess at which platform a skill came from, for friendly output.
fn detect_source(s: &Skill, path: &Path) -> String {
    let p = path.to_string_lossy();
    if p.contains(".claude") {
        return "claude".into();
    }
    if p.contains(".cursor") {
        return "cursor".into();
    }
    if p.contains(".codex") {
        return "codex".into();
    }

    let has = |k: &str| s.frontmatter.contains_key(k);
    if has("context") || has("effort") || has("hooks") || has("allowed-tools") {
        "claude".into()
    } else if has("agents") || has("compatible_agents") {
        "vercel/generic".into()
    } else {
        "open".into()
    }
}

fn join(items: &[String]) -> String {
    if items.is_empty() {
        "(none)".into()
    } else {
        items.join(", ")
    }
}
