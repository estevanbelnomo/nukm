# Ecosystem fit: Rust

## 1. Manifest signal

- `Cargo.toml` at the root of a crate or workspace. The presence of a `[workspace]` section distinguishes a workspace manifest (no `target/` of its own, but child members share one at the workspace root) from a leaf crate manifest.
- Optional: `Cargo.lock` committed in application crates, ignored in library crates. Not reclaimable; used as a signal of an active project.

## 2. Artefacts

- **`target/`** at the workspace root (or crate root for single-package crates). The dominant reclamation target. Typical size 200 MB to 10 GB for active workspaces.
- **`target/doc/`**, **`target/debug/`**, **`target/release/`** are subtrees inside `target/`. Reclaim as a whole unless the user explicitly opts into finer granularity.
- **Global cache (under `CacheProvider`):** `~/.cargo/registry/cache/`, `~/.cargo/registry/src/`, `~/.cargo/git/checkouts/`. Rebuildable from `Cargo.lock` plus network.

## 3. Safety gates

- Uncommitted changes anywhere in the workspace (`git status --porcelain`).
- Stashes present (`git stash list`).
- Upstream mismatch: local commits not reachable from a tracked remote branch.
- Recent filesystem mtime on `target/` (suggests active build; default threshold TBD in B1).
- Cargo daemon: `cargo` does not run a daemon, so no process-held locks to worry about.

## 4. Trait fit

- **`ProjectDetector`:** clean fit. Walk from `Cargo.toml`, find `target/`, return candidates. No ambiguity.
- **`CacheProvider`:** clean fit for the global registry and git checkouts. Enumerate by listing `~/.cargo/registry/` contents; no API needed.
- **`Scorer`:** size and age both meaningful. `target/` benefits most from age-based scoring (a week-old `target/` rebuilds in minutes; a year-old `target/` is almost certainly dormant).
- **`Platform`:** only `home()` and `trash()` needed for Rust. No daemon detection required.
- **`RuleEngine`:** straightforward; rules encode "skip if workspace has uncommitted changes" and similar.

## 5. Misfits flagged

None material. Rust is the simplest case and is a useful baseline against which other ecosystems should be compared. If a trait signature does not work cleanly for Rust, it is likely wrong.

Minor nuance worth watching in B1:
- **Workspace members' own `target/`.** A crate inside a workspace does not typically have its own `target/`; the workspace root owns it. A crate configured with `target-dir = ...` in `.cargo/config.toml` can override this. Detector must follow `target-dir` when set. Not a trait-level misfit, but a detector-level quirk to test for.
