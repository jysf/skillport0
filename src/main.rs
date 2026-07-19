//! Binary entry point: the `skillport` CLI.
//!
//! `skillport lint <PATH> [--json | --sarif] [--strict]` (SPEC-005/SPEC-008,
//! `docs/api-contract.md`): walk `<PATH>` into a [`skillport::Collection`], run
//! [`skillport::lint_skill`] over it via [`skillport::Report::from_collection`],
//! print a human report (default), `--json`, or `--sarif` (mutually exclusive
//! with `--json`), and exit with the CI contract code
//! (`Report::exit_code(strict)`; `2` for a usage error). Results go to
//! **stdout**, diagnostics/usage errors to **stderr** (machine consumers read
//! stdout only). `Commands` is a subcommand enum so `audit` (PROJ-002) can be
//! added later without reshaping `Lint` (DEC-001: lint only, no
//! convert/push/profiles here).

use clap::{Parser, Subcommand, ValueEnum};
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
        /// Emit SARIF 2.1.0 for code-scanning ingestion. Mutually exclusive with `--json`.
        #[arg(long, conflicts_with = "json")]
        sarif: bool,
        /// Treat warnings as failures (affects exit code only).
        #[arg(long)]
        strict: bool,
        /// Widen recognized fields/behavior for a specific agent platform,
        /// verified from that platform's primary docs (DEC-002). Only
        /// `claude` is verified so far.
        #[arg(long, value_enum)]
        target: Option<TargetArg>,
    },
}

/// The clap-facing `--target` values. Kept separate from `skillport::Target`
/// so the CLI's arg-parsing enum (and its clap `ValueEnum` labels) don't leak
/// into the library; `into()` maps it 1:1.
#[derive(Debug, Clone, Copy, ValueEnum)]
enum TargetArg {
    Claude,
}

impl From<TargetArg> for skillport::Target {
    fn from(arg: TargetArg) -> Self {
        match arg {
            TargetArg::Claude => skillport::Target::Claude,
        }
    }
}

impl TargetArg {
    /// The `--json` `target` label for this value (DEC-005).
    fn label(self) -> &'static str {
        match self {
            TargetArg::Claude => "claude",
        }
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        Commands::Lint {
            path,
            json,
            sarif,
            strict,
            target,
        } => lint(&path, json, sarif, strict, target),
    }
}

/// Run the `lint` subcommand and return the process exit code.
fn lint(path: &Path, json: bool, sarif: bool, strict: bool, target: Option<TargetArg>) -> ExitCode {
    // Usage error BEFORE walking (`walk` is total and returns an empty
    // collection for a missing path — the CLI must check existence itself,
    // per the spec's exit-code table and Notes).
    if !path.exists() {
        eprintln!("skillport: path does not exist: {}", path.display());
        return ExitCode::from(2);
    }

    let collection = walk(path);
    let rule_target: Option<skillport::Target> = target.map(Into::into);
    let report = Report::from_collection(&collection, |s| {
        skillport::lint_skill_with_target(s, rule_target)
    });

    if sarif {
        println!("{}", emit::sarif(&report));
    } else if json {
        println!("{}", emit::json(&report, target.map(TargetArg::label)));
    } else {
        print!("{}", emit::human(&report));
    }

    let code = report.exit_code(strict);
    // `Report::exit_code` returns 0 or 1 (DEC-003); ExitCode::from wants a u8.
    ExitCode::from(code as u8)
}
