//! Nukm command-line entry point.

#![allow(missing_docs)]

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

/// Nukm: manifest-aware developer disk reclaim tool.
#[derive(Debug, Parser)]
#[command(
    name = "nukm",
    version,
    about = "Find and reclaim developer disk waste safely.",
    long_about = "Nukm scans for developer disk waste (build outputs, dependency caches, \
                  dormant artefacts) and reclaims space safely via the OS trash. \
                  Destructive operations require --execute; otherwise, nukm runs in \
                  dry-run mode and describes what it would do without making changes."
)]
struct Cli {
    /// The operation to perform.
    #[command(subcommand)]
    command: Command,

    /// Emit machine-readable JSON on stdout instead of human-formatted text.
    #[arg(long, global = true)]
    json: bool,

    /// Perform destructive operations. Without this flag, nukm runs in
    /// dry-run mode and describes what it would do without making changes.
    /// There is no `--dry-run` flag: dry-run is the default.
    #[arg(long, global = true)]
    execute: bool,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Scan for reclaimable developer disk waste.
    Scan {
        /// Root path to scan from. Defaults to the current directory.
        #[arg(default_value = ".")]
        root: PathBuf,
    },

    /// Clean reclaimable artefacts. Requires `--execute` to actually delete.
    Clean {
        /// Root path to clean from. Defaults to the current directory.
        #[arg(default_value = ".")]
        root: PathBuf,
    },

    /// List or inspect the loaded rule set.
    Rules,

    /// Audit environment health and configuration.
    Doctor,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let mode = if cli.execute { "EXECUTE" } else { "DRY-RUN" };
    let format = if cli.json { "json" } else { "text" };

    match cli.command {
        Command::Scan { root } => {
            println!(
                "[{mode}] scan {} (output={format}) - not yet implemented (C-phase skeleton)",
                root.display()
            );
        }
        Command::Clean { root } => {
            println!(
                "[{mode}] clean {} (output={format}) - not yet implemented (C-phase skeleton)",
                root.display()
            );
        }
        Command::Rules => {
            println!(
                "[{mode}] rules (output={format}) - not yet implemented (C-phase skeleton). \
                 Core library version: {}",
                nukm_core::VERSION
            );
        }
        Command::Doctor => {
            println!(
                "[{mode}] doctor (output={format}) - not yet implemented (C-phase skeleton). \
                 Core library version: {}",
                nukm_core::VERSION
            );
        }
    }

    ExitCode::SUCCESS
}
