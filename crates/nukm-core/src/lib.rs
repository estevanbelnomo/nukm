//! Nukm core library.
//!
//! Manifest-aware, git-aware developer disk reclaim engine. This crate owns
//! detection, scoring, the rule engine, and the shared data vocabulary used by
//! [`nukm-cli`](https://crates.io/crates/nukm-cli) and, later, the Tauri GUI.
//! Consumers should program against the traits and types re-exported here and
//! should not depend on internal module paths.
//!
//! # Architecture summary
//!
//! - [`Platform`] abstracts OS-specific behaviour (trash, home directory,
//!   global cache roots, volume compaction).
//! - [`ProjectDetector`] walks a directory tree from a manifest signal and
//!   returns [`Candidate`]s for a single ecosystem.
//! - [`CacheProvider`] enumerates ecosystem caches that do not fit the
//!   filesystem-walking model, such as the Docker daemon or the pip global
//!   cache.
//! - [`Scorer`] assigns a reclamation score to each candidate.
//! - [`RuleEngine`] is a concrete TOML-driven rule evaluator. It is a struct,
//!   not a trait, because the rule evaluation algorithm is deterministic and
//!   has no foreseeable second implementation.
//! - [`ActionManifest`] is the reversibility record written by `clean` runs.
//!
//! # Status
//!
//! C-phase skeleton. Trait and type shapes are stable targets; method bodies
//! are `todo!()` until B1 lands the first detector.

#![warn(missing_docs)]

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};

/// Crate version surfaced for diagnostic output.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Unified error type for `nukm-core`.
///
/// Variants are intentionally broad for the C-phase; expect them to split as
/// real detectors and platforms land in B1 and beyond.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Filesystem or other I/O failure.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// Rule file, manifest, or TOML parse failure.
    #[error("parse error: {0}")]
    Parse(String),

    /// Platform-specific operation failed (trash send, volume compaction, ...).
    #[error("platform error: {0}")]
    Platform(String),

    /// Detector failed to enumerate candidates.
    #[error("detector error ({ecosystem:?}): {message}")]
    Detector {
        /// Ecosystem whose detector raised the error.
        ecosystem: Ecosystem,
        /// Human-readable message.
        message: String,
    },

    /// Action manifest could not be read, written, or parsed.
    #[error("manifest error: {0}")]
    Manifest(String),

    /// JSON serialisation or deserialisation failure.
    #[error("serde error: {0}")]
    Serde(#[from] serde_json::Error),
}

/// Convenience alias for `Result<T, Error>` inside `nukm-core`.
pub type Result<T> = std::result::Result<T, Error>;

// ---------------------------------------------------------------------------
// Ecosystems and artefacts
// ---------------------------------------------------------------------------

/// Ecosystems recognised by Nukm. New variants require a detector or cache
/// provider plus an ecosystem-fit document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Ecosystem {
    /// Rust and Cargo.
    Rust,
    /// Node.js and the npm / pnpm / yarn family.
    Node,
    /// Python, pip, venv, poetry, uv.
    Python,
    /// Go modules.
    Go,
    /// Java with Maven or Gradle.
    Java,
    /// Docker daemon caches (images, build cache, volumes).
    Docker,
}

/// What kind of artefact a candidate represents. Influences safety gates,
/// scoring heuristics, and the UI grouping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtefactKind {
    /// Build output: `target/`, `build/`, `dist/`, `.next/`.
    BuildOutput,
    /// Project-local dependency cache: `node_modules/`, `.venv/`, `vendor/`.
    DependencyCache,
    /// Package-manager global store: `~/.cargo/registry`, `~/.npm`, `~/.m2/repository`.
    PackageCache,
    /// Container or image store: Docker dangling images, BuildKit cache.
    ImageStore,
    /// State held by a long-running daemon: `.gradle/`, Metro cache.
    DaemonState,
    /// Anything that does not fit the other variants.
    Other,
}

// ---------------------------------------------------------------------------
// Candidate and scoring
// ---------------------------------------------------------------------------

/// A reclamation candidate proposed by a detector or cache provider.
///
/// Candidates are inert data. Reclamation is performed through [`Platform`]
/// and recorded in an [`ActionManifest`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candidate {
    /// Absolute path to the artefact, or a provider-defined identifier for
    /// non-filesystem candidates (for example a Docker image digest).
    pub path: PathBuf,

    /// Ecosystem that produced this candidate.
    pub ecosystem: Ecosystem,

    /// Kind of artefact, used for grouping and safety gating.
    pub kind: ArtefactKind,

    /// Size in bytes. For non-filesystem candidates, provider-defined
    /// (typically uncompressed size).
    pub size_bytes: u64,

    /// Most recent modification time observed for the artefact. `None` when
    /// the provider does not expose one (for example, Docker image metadata).
    pub last_modified: Option<SystemTime>,

    /// Safety gates that currently apply. Empty means no known obstruction.
    pub gates: Vec<SafetyGate>,

    /// Assigned score. `None` until a [`Scorer`] has processed this candidate.
    pub score: Option<Score>,
}

/// A reclamation score. Higher means "more worth reclaiming" (older, larger,
/// dormant). The exact formula is [`Scorer`]-defined.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Score {
    /// Aggregate score in the range `[0.0, 1.0]`.
    pub total: f64,
    /// Contribution from size in the range `[0.0, 1.0]`.
    pub size_component: f64,
    /// Contribution from age in the range `[0.0, 1.0]`.
    pub age_component: f64,
}

/// Context passed to [`Scorer::score`] so scorers can rank candidates relative
/// to the run as a whole (for example, "largest in the scan sets the ceiling").
#[derive(Debug, Clone)]
pub struct ScoringContext {
    /// Reference time; usually the scan start.
    pub now: SystemTime,

    /// Size of the largest candidate in the current run. Allows a scorer to
    /// normalise its size component without scanning twice.
    pub max_size_bytes: u64,
}

/// A reason a candidate might be unsafe to reclaim.
///
/// The set is intentionally open for additions; add new variants in an
/// ADR-amended minor release rather than shoehorning into `CustomRule`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "gate", rename_all = "snake_case")]
pub enum SafetyGate {
    /// Working tree contains uncommitted changes.
    UncommittedChanges,
    /// Local branch is ahead of its upstream by `count` commits.
    UnpushedCommits {
        /// Number of unpushed commits.
        count: u32,
    },
    /// Repository holds `count` stashed change sets.
    Stashes {
        /// Number of stashes.
        count: u32,
    },
    /// Filesystem activity newer than the configured threshold.
    RecentActivity {
        /// Age of the most recent modification in seconds.
        age_seconds: u64,
    },
    /// A long-running daemon or process is believed to hold the artefact.
    DaemonRunning {
        /// Daemon or process name (for example `"gradle"`).
        name: String,
    },
    /// A user-defined rule file blocked the candidate.
    CustomRule {
        /// Rule identifier.
        name: String,
        /// Human-readable explanation.
        reason: String,
    },
}

// ---------------------------------------------------------------------------
// Scan result
// ---------------------------------------------------------------------------

/// Result of a complete scan run across any number of detectors and
/// cache providers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    /// When the scan began.
    pub started_at: SystemTime,
    /// When the scan completed.
    pub finished_at: SystemTime,
    /// Total wall-clock duration of the scan.
    pub duration: Duration,
    /// Number of filesystem paths visited.
    pub scanned_paths: usize,
    /// Sum of `size_bytes` across all candidates.
    pub total_bytes: u64,
    /// Every candidate found, in no particular order.
    pub candidates: Vec<Candidate>,
}

// ---------------------------------------------------------------------------
// Action manifest (reversibility record)
// ---------------------------------------------------------------------------

/// A record of a single `clean` run, intended for audit and undo.
///
/// C-phase defines the type shape and JSON serialisation contract. Persistence
/// (on-disk location, naming, rotation, concurrency) lands in B1. See
/// [`docs/phase-1-plan.md`](../../../docs/phase-1-plan.md).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionManifest {
    /// Run identifier: opaque string, typically an ISO 8601 timestamp plus a
    /// short suffix.
    pub run_id: String,
    /// When the run began.
    pub started_at: SystemTime,
    /// When the run ended. `None` while a run is in progress.
    pub finished_at: Option<SystemTime>,
    /// Whether this run was a dry run. `true` means no destructive action
    /// was taken.
    pub dry_run: bool,
    /// One entry per candidate considered by the run.
    pub actions: Vec<ActionRecord>,
}

/// A single entry within an [`ActionManifest`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionRecord {
    /// Identifies the artefact (filesystem path or provider identifier).
    pub path: PathBuf,
    /// Ecosystem the candidate came from.
    pub ecosystem: Ecosystem,
    /// Kind of artefact.
    pub kind: ArtefactKind,
    /// Size at the moment of the action.
    pub size_bytes: u64,
    /// What was attempted (or simulated) for this candidate.
    pub action: ActionKind,
    /// Outcome of the attempt.
    pub result: ActionResult,
    /// When the action ran.
    pub timestamp: SystemTime,
}

/// What was attempted for a candidate.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum ActionKind {
    /// Sent to the OS trash via [`Platform::trash`].
    SendToTrash,
    /// Removed without going through the trash. Requires explicit opt-in.
    HardDelete,
    /// Deliberately not acted on. Reason is carried by the result.
    Skip,
}

/// Outcome of an [`ActionKind`].
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum ActionResult {
    /// The action completed successfully.
    Success,
    /// The candidate was skipped, with a reason.
    Skipped {
        /// Why the candidate was not acted on.
        reason: String,
    },
    /// The action failed, with a reason.
    Failed {
        /// What went wrong.
        reason: String,
    },
}

// ---------------------------------------------------------------------------
// Platform trait
// ---------------------------------------------------------------------------

/// OS-specific behaviour abstracted behind a trait so tests can supply a
/// fake implementation.
///
/// Implementations live alongside the binary (Windows, macOS, Linux) and
/// are selected at runtime.
pub trait Platform: Send + Sync {
    /// Short identifier for the platform, for example `"windows"`.
    fn name(&self) -> &'static str;

    /// Current user's home directory.
    fn home(&self) -> PathBuf;

    /// Roots of global package or tool caches appropriate to scan. Used by
    /// [`CacheProvider`] implementations to discover their stores.
    fn global_cache_roots(&self) -> Vec<PathBuf>;

    /// Send a path to the OS trash or recycle bin. The path must exist.
    ///
    /// On platforms without a trash service, implementations should return
    /// [`Error::Platform`] rather than silently hard-deleting.
    fn trash(&self, path: &Path) -> Result<()>;

    /// Compact a mountable volume image, for example a WSL `vhdx` or an
    /// APFS snapshot. Returns bytes reclaimed. `Ok(0)` means "not supported
    /// on this platform" and is not an error.
    fn compact_volume(&self, path: &Path) -> Result<u64>;
}

// ---------------------------------------------------------------------------
// Detector and cache provider traits
// ---------------------------------------------------------------------------

/// Finds reclamation candidates by walking a directory tree from a manifest
/// signal (for example `Cargo.toml`, `package.json`).
///
/// Detectors are per-ecosystem and stateless.
pub trait ProjectDetector: Send + Sync {
    /// Which ecosystem this detector handles.
    fn ecosystem(&self) -> Ecosystem;

    /// Walk `root` looking for this ecosystem's projects and return all
    /// reclamation candidates discovered. Implementations should respect
    /// `.gitignore` and cap depth sanely.
    ///
    /// This method does not score or rank; callers pair the returned
    /// candidates with a [`Scorer`].
    fn detect(&self, root: &Path, platform: &dyn Platform) -> Result<Vec<Candidate>>;
}

/// Enumerates reclamation candidates that do not map onto a filesystem walk.
///
/// The canonical example is the Docker daemon: images, build cache, and
/// dangling volumes are owned by a daemon and exposed via an API, not a
/// directory. Global pip and Gradle caches are also natural fits, even though
/// they have a filesystem representation, because the ecosystem provides a
/// native enumeration mechanism that is more reliable than walking.
pub trait CacheProvider: Send + Sync {
    /// Ecosystem this provider handles.
    fn ecosystem(&self) -> Ecosystem;

    /// Enumerate reclaimable cache entries. Providers may perform network or
    /// daemon calls here.
    fn enumerate(&self, platform: &dyn Platform) -> Result<Vec<Candidate>>;

    /// Remove (or simulate removing) a previously enumerated candidate.
    ///
    /// When `dry_run` is `true`, the provider must not perform destructive
    /// work but must still return a complete [`ActionRecord`] describing
    /// what would happen.
    fn reclaim(
        &self,
        candidate: &Candidate,
        dry_run: bool,
        platform: &dyn Platform,
    ) -> Result<ActionRecord>;
}

// ---------------------------------------------------------------------------
// Scorer trait
// ---------------------------------------------------------------------------

/// Assigns a [`Score`] to a candidate.
///
/// Implementations are swappable so the community can experiment with scoring
/// strategies without forking the core engine.
pub trait Scorer: Send + Sync {
    /// Compute a score for `candidate` in the given `context`.
    fn score(&self, candidate: &Candidate, context: &ScoringContext) -> Score;
}

// ---------------------------------------------------------------------------
// Rule engine (concrete)
// ---------------------------------------------------------------------------

/// A single rule loaded by the [`RuleEngine`].
///
/// C-phase skeleton: fields beyond `id` and `ecosystem` land in B1 when the
/// TOML rule format is drafted and the first detector exercises it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    /// Human-readable identifier, unique across loaded rule sets.
    pub id: String,
    /// Ecosystem this rule applies to.
    pub ecosystem: Ecosystem,
}

/// Decision produced by evaluating a candidate against the rule set.
#[derive(Debug, Clone)]
pub enum Decision {
    /// The candidate may be reclaimed subject to safety gates.
    Clean,
    /// The candidate should not be reclaimed. `reason` is human-readable.
    Skip {
        /// Why the candidate was rejected.
        reason: String,
    },
    /// The candidate needs more information before a decision is possible.
    Defer {
        /// What is missing.
        reason: String,
    },
}

/// Evaluates [`Candidate`]s against a loaded rule set.
///
/// C-phase skeleton with `todo!()` bodies. B1 lands:
/// - TOML parser for the rule schema (see `NUKM_PROJECT.md` Section 5.3)
/// - Condition evaluator with access to git state and scoring context
/// - User-config overlay at `~/.config/nukm/rules/`
pub struct RuleEngine {
    rules: Vec<Rule>,
}

impl RuleEngine {
    /// Build an engine pre-loaded with the rules shipped in the binary.
    pub fn with_embedded() -> Result<Self> {
        todo!("load embedded rules; lands in B1 with the first real detector")
    }

    /// Build an engine from a rules directory, overlaid on top of embedded
    /// rules. Missing directories are not an error: the caller gets the
    /// embedded set only.
    pub fn load(rules_dir: &Path) -> Result<Self> {
        let _ = rules_dir;
        todo!("load and overlay rules from disk; lands in B1")
    }

    /// Return a [`Decision`] for `candidate` given the current rule set and
    /// platform state.
    pub fn evaluate(&self, candidate: &Candidate, platform: &dyn Platform) -> Decision {
        let _ = (candidate, platform);
        todo!("condition evaluation; lands in B1")
    }

    /// Loaded rules, for introspection by `nukm rules` and the GUI.
    pub fn rules(&self) -> &[Rule] {
        &self.rules
    }
}
