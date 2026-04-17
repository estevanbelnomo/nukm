# Nukm

Cross-platform, manifest-aware developer disk reclaim tool. Rust core, CLI first.

Nukm finds gigabytes of reclaimable developer waste safely. It reads manifests (`Cargo.toml`, `package.json`, `go.mod`, ...), checks git state, scores candidates by age and activity, and sends deletions to the OS trash so every action is reversible.

**Status:** scaffold. Not yet usable.

## Workspace layout

```
nukm/
├── Cargo.toml                  workspace manifest
├── CLAUDE.md                   agent rules (Arthur, Magnus, Kenji, Luca, Ingrid, Viktor)
├── HANDOFF_PROTOCOL.md         inter-agent handoff spec
├── rust-toolchain.toml         pinned Rust 1.85
├── crates/
│   ├── nukm-core/              detection, scoring, rule engine
│   └── nukm-cli/               clap-based CLI, binary name: `nukm`
├── rules/                      TOML rule files (shipped with the binary)
├── docs/                       mdBook source (later)
└── .github/workflows/          CI
```

## Build

```powershell
cargo build
cargo test
cargo fmt --all
cargo clippy --all-targets -- -D warnings
```

## Licence

Dual-licensed under MIT or Apache-2.0, at your option. See `LICENSE-MIT` and `LICENSE-APACHE`.

## Project direction

Full spec and roadmap: `../NUKM_PROJECT.md`. Commercial strategy, monetisation path, and scope gates live there. This repository is the implementation.
