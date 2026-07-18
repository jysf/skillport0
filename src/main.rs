//! Binary entry point: the `skillport` CLI.
//!
//! `skillport lint <PATH> [--json] [--strict]` (SPEC-005, `docs/api-contract.md`):
//! walk `<PATH>` into a [`skillport::Collection`], run [`skillport::lint_skill`]
//! over it via [`skillport::Report::from_collection`], print a human report
//! (default) or `--json`, and exit with the CI contract code
//! (`Report::exit_code(strict)`; `2` for a usage error). Results go to
//! **stdout**, diagnostics/usage errors to **stderr** (machine consumers read
//! stdout only). `Commands` is a subcommand enum so `audit` (PROJ-002) can be
//! added later without reshaping `Lint` (DEC-001: lint only, no
//! convert/push/profiles here).

use clap::{Parser, Subcommand};
use skillport::{emit, walk, Report};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Parser)]
#[command(
    name = "skillport",
    version,
    about = "Validate and audit agent SKILL.md files."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate skill(s) against the open Agent Skills spec.
    Lint {
        /// A SKILL.md file, a skill folder, or a directory tree.
        path: PathBuf,
        /// Emit the stable JSON schema instead of human-readable text.
        #[arg(long)]
        json: bool,
        /// Treat warnings as failures (affects exit code only).
        #[arg(long)]
        strict: bool,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        Commands::Lint { path, json, strict } => lint(&path, json, strict),
    }
}

/// Run the `lint` subcommand and return the process exit code.
fn lint(path: &Path, json: bool, strict: bool) -> ExitCode {
    // Usage error BEFORE walking (`walk` is total and returns an empty
    // collection for a missing path — the CLI must check existence itself,
    // per the spec's exit-code table and Notes).
    if !path.exists() {
        eprintln!("skillport: path does not exist: {}", path.display());
        return ExitCode::from(2);
    }

    let collection = walk(path);
    let report = Report::from_collection(&collection, skillport::lint_skill);

    if json {
        println!("{}", emit::json(&report, None));
    } else {
        print!("{}", emit::human(&report));
    }

    let code = report.exit_code(strict);
    // `Report::exit_code` returns 0 or 1 (DEC-003); ExitCode::from wants a u8.
    ExitCode::from(code as u8)
}
