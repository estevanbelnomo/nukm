# Handoff: Arthur to Ingrid

## Context

- Phase: Phase 1, C-phase (Days 1-3)
- Crate: cross-cutting (test infrastructure)
- Files: `crates/nukm-core/tests/`, `crates/nukm-cli/tests/`, `tests/common/mod.rs` (workspace root), possibly `.config/nextest.toml`

## State

- **Done:** workspace scaffold, Magnus-owned types and traits will land alongside this work.
- **In progress:** nothing. This handoff starts the C-phase test scaffolding.
- **Blocked:** your compile-only sanity tests import public items from `nukm_core` and build a `clap` command from `nukm-cli`; they will not compile until Magnus's skeleton is in place. Sequence your work after Magnus's first commit lands, or coordinate merging order with Arthur.

## Request

Land the test scaffolding, one compile-only sanity test per crate, and a decision on `cargo-nextest` adoption (accept or defer).

## Acceptance criteria

**`tests/common/mod.rs` at workspace root:**
- Helper stubs for the `assert_fs` patterns expected in B1 and beyond. Empty bodies, rich doc comments explaining intended use.
- At minimum: a helper to build a temporary directory with a named manifest fixture, a helper to seed a git repository with a given state (clean, dirty, stashed), a helper to tear down predictably. Bodies may be `unimplemented!()` - they exist as contract, not implementation.

**Per-crate sanity tests:**
- `crates/nukm-core/tests/sanity.rs` with `#[test] fn it_compiles() { }`. Must import at least one public item from `nukm_core` (for example, reference `nukm_core::Ecosystem` or `nukm_core::VERSION`) so the linker actually exercises the crate API.
- `crates/nukm-cli/tests/sanity.rs` with `#[test] fn it_compiles() { }`. Must construct a `clap` command from the CLI crate so its public surface is exercised.

**`cargo-nextest` decision:**
- Either: ADR-0004 proposing adoption, with rationale and a minimal `.config/nextest.toml` config. Ingrid's call.
- Or: a short note in the commit message deferring the decision to B1, with a one-line reason.

**Explicitly out of scope in C:**
- `proptest` - lands in B1 with its first real caller.
- `insta` snapshot testing - lands in B2 with its first real caller.
- Coverage tooling (`cargo-llvm-cov`, `tarpaulin`) - Phase 2 at earliest.
- Actual assertions inside the sanity tests.

**Quality gates:**
- `cargo test --workspace` green once Magnus's skeleton is in place. Every `it_compiles` test must pass.
- `cargo fmt --all --check` green.
- `cargo clippy --workspace --all-targets -- -D warnings` green.

## Return path

Return to Arthur. Expected follow-up: Arthur confirms the scaffolding is consumable by B1 without rework and, if nextest is adopted, that CI uses it.
