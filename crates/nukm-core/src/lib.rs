//! Nukm core library.
//!
//! Manifest-aware, git-aware developer disk reclaim engine. This crate owns
//! detection, scoring, and the rule engine. It performs no user-facing I/O;
//! consumers are `nukm-cli` and (later) `nukm-gui`.

#![warn(missing_docs)]

/// Crate version surfaced for diagnostic output.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
