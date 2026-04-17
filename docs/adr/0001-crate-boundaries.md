# ADR-0001: Crate boundaries

Date: 2026-04-17
Status: Accepted

## Context

Nukm needs to ship a CLI (`cargo install nukm`), a GUI (later, Phase 4), and expose a reusable library surface so third parties can embed the engine in scripts, CI tooling, or alternative front-ends. The workspace structure locks in these boundaries for the next several months and is expensive to revisit once detectors and scorers start to depend on module paths.

Options considered:

1. **Single crate with features.** `nukm` crate exposes both library and binary, with `cli`, `gui`, and `core` as optional features. Simple to publish; hides the library-vs-app distinction badly, and makes the GUI's heavier dependencies transitive even when the feature is off.
2. **Two crates: `nukm-core` plus `nukm-cli`.** Library-plus-binary split. Clean separation; easy to add `nukm-gui` in Phase 4 without disturbing existing crates.
3. **Three crates from day one: `nukm-core`, `nukm-cli`, `nukm-gui`.** Scaffolds the GUI early so its Tauri dependencies do not surprise us later. Adds a crate that will sit mostly empty until Phase 4.

## Decision

Option 2. The workspace has exactly two member crates in Phase 1: `nukm-core` (library) and `nukm-cli` (binary named `nukm`). `nukm-gui` joins the workspace when Phase 4 starts and will have its own entry in `Cargo.toml` at that time.

Workspace-level policies in `Cargo.toml`:

- Shared package metadata (`version`, `edition`, `rust-version`, `license`, `authors`, `repository`, `homepage`, `readme`) lives on `[workspace.package]`.
- Shared dependencies live on `[workspace.dependencies]`. Member crates consume them with `{ workspace = true }`.
- Workspace-level lints (`[workspace.lints]`) apply to every member; members opt in with `[lints] workspace = true`.

C-phase lint relaxations (documented here because every contributor will notice them):

- `clippy::pedantic` retained, but `missing_errors_doc`, `missing_panics_doc`, `must_use_candidate`, and `module_name_repetitions` are allowed. The first three are noisy on `todo!()` skeleton stubs; `module_name_repetitions` is spurious in a small-surface crate like `nukm-core`.
- `clippy::nursery` is not enabled. Its lints are by definition unstable and not worth a C-phase fight.
- `clippy::todo` is allowed so `todo!()` bodies do not fail CI.

B1 revisits each allowance once real error paths, panics, and return values exist to document.

## Consequences

**Makes easy:**
- `cargo install nukm` resolves to the CLI. Users do not need to know the crate split.
- Library consumers (scripts, CI, future IDE integrations) can depend on `nukm-core` without pulling the CLI's `clap` surface.
- Adding `nukm-gui` in Phase 4 is a workspace member addition, not a refactor.

**Makes hard:**
- Two crates means two `Cargo.toml` files to keep in sync. Mitigated by `[workspace.dependencies]`.
- Publishing is a two-step release: `nukm-core` before `nukm-cli` because the latter depends on the former. `cargo release` or `cargo-dist` will handle this; the ordering becomes a release-checklist item.

**Defers:**
- GUI crate creation to Phase 4 (see roadmap).
- Any decision about publishing `nukm-core` separately on crates.io versus keeping it workspace-internal. Likely published so the library surface is stable and discoverable; confirm during Phase 2.
