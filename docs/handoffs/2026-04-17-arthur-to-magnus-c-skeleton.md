# Handoff: Arthur to Magnus

## Context

- Phase: Phase 1, C-phase (Days 1-3)
- Crate: `nukm-core`, `nukm-cli` (cross-cutting)
- Files: `crates/nukm-core/src/lib.rs`, `crates/nukm-cli/src/main.rs`, workspace `Cargo.toml`, `docs/adr/*`, `docs/ecosystem-fit/{rust,node,go,java}.md`

## State

- **Done:** workspace scaffold, MSRV 1.85 pinned via `rust-toolchain.toml`, strict workspace-wide lints, CI skeleton (Windows only) green, Phase 1 plan approved (`docs/phase-1-plan.md`).
- **In progress:** nothing. This handoff starts the C-phase.
- **Blocked:** no feature work. `cargo test --workspace` depends on Ingrid's compile-only sanity tests; coordinate order with Arthur if your order of landing matters for local green builds.

## Request

Land the architectural skeleton for `nukm-core` and `nukm-cli` as a single reviewable artefact, plus four of the six ecosystem-fit docs (`rust`, `node`, `go`, `java`), plus the three opening ADRs. Scope is interfaces, signatures, and design documents only: no algorithmic logic.

## Acceptance criteria

**`nukm-core`:**
- Public types defined (no logic, doc comments required): `Candidate`, `ScanResult`, `Score`, `SafetyGate`, `Ecosystem`, `ActionManifest`, `ActionRecord`. `ActionManifest` / `ActionRecord` carry field definitions and a `serde` JSON serialisation contract. On-disk persistence is explicitly deferred to B1 and must be stated as such in doc comments.
- Traits defined (signatures and doc comments, no implementations): `ProjectDetector`, `CacheProvider`, `Scorer`, `Platform`.
- Concrete struct defined: `RuleEngine` with narrow public API (method signatures with `todo!()` bodies).
- Error taxonomy via `thiserror`: `nukm_core::Error` covering at minimum IO, parse, platform, detector, manifest variants.

**`nukm-cli`:**
- Command stubs using `clap` derive: `scan`, `clean`, `rules`, `doctor`. Each prints a one-line "not yet implemented" message and exits `0`.
- Shared flags surfaced at the top level: `--json`, `--dry-run` (default `true`), `--execute` (mutually exclusive with `--dry-run`).
- Binary name remains `nukm`.

**Workspace `Cargo.toml`:**
- Add `[workspace.dependencies]` entries for: `thiserror`, `serde` (with `derive`), `serde_json`, `clap` (with `derive`), `tracing`. Pin to recent minor versions; commit `Cargo.lock`.
- Individual crates consume from `{ workspace = true }`.

**Ecosystem-fit docs (Magnus owns):**
- `docs/ecosystem-fit/rust.md`, `node.md`, `go.md`, `java.md`.
- Each follows the schema in `docs/ecosystem-fit/README.md`.
- Each flags misfits (or confirms a clean fit) against the current traits.

**ADRs:**
- `docs/adr/0001-crate-boundaries.md`: why `nukm-core` + `nukm-cli` now, with `nukm-gui` deferred to Phase 4.
- `docs/adr/0002-platform-trait.md`: shape of the `Platform` trait and why a trait rather than `#[cfg]`-gated concrete types.
- `docs/adr/0003-detector-vs-cache-provider.md`: why `ProjectDetector` and `CacheProvider` are separate traits.

**Quality gates:**
- `cargo fmt --all --check` green.
- `cargo clippy --workspace --all-targets -- -D warnings` green.
- `cargo build --workspace` green.
- `cargo test --workspace` green once Ingrid's scaffolding lands (coordinate).

## Return path

Return to Arthur. Expected follow-up: Arthur runs the six-ecosystem smoke test across the merged skeleton (your four docs plus Kenji's two) and flags misfits as proposed amendment ADRs before Estevan's approval.
