# ADR-0002: Platform as a trait

Date: 2026-04-17
Status: Accepted

## Context

OS-specific behaviour in Nukm spans at least:

- Sending a path to the trash / recycle bin
- Locating the user's home directory
- Enumerating global cache roots (Cargo, npm, pip, Gradle, Maven, ...)
- Compacting volume images (WSL `vhdx`, APFS snapshots)
- Detecting running daemons (Gradle, Metro, Docker)

Detectors and cache providers call into this behaviour from ecosystem code that should remain platform-agnostic. The test harness needs to substitute a fake for every one of these calls to avoid hitting a real trash bin during `cargo test`.

Options considered:

1. **`#[cfg]`-gated concrete functions.** `platform::trash(path)` compiles to different bodies on different OSes. Simple; impossible to mock without a feature flag that ships in production.
2. **`#[cfg]`-gated concrete types plus mock feature.** A `Platform` struct with OS-specific implementations behind `#[cfg]`, plus a `MockPlatform` gated by `#[cfg(test)]` or a `mock` feature. Testable; but every detector function either takes `&Platform` (locking in the concrete type) or uses a type alias that switches under `#[cfg]`, both of which leak.
3. **Trait with OS implementations as structs.** `pub trait Platform` with `WindowsPlatform`, `MacosPlatform`, `LinuxPlatform` implementations. Detectors accept `&dyn Platform`. Mocks implement the trait directly in tests with no `#[cfg]` gymnastics.

## Decision

Option 3. `nukm-core` defines `pub trait Platform: Send + Sync`. Detectors, cache providers, and the rule engine accept `&dyn Platform` or `impl Platform`. The binary crates contain the Windows / macOS / Linux implementations and select one at startup (initially via `#[cfg]`; runtime override envisaged for WSL edge cases).

Trait methods are narrow and orthogonal:

- `name(&self) -> &'static str`
- `home(&self) -> PathBuf`
- `global_cache_roots(&self) -> Vec<PathBuf>`
- `trash(&self, &Path) -> Result<()>`
- `compact_volume(&self, &Path) -> Result<u64>` (returns `Ok(0)` when not supported)

`Send + Sync` is required so that scan workers can share a single `Arc<dyn Platform>` across threads in Phase 2 when parallel scanning lands.

## Consequences

**Makes easy:**
- Unit and integration tests substitute a `MockPlatform` implementation with zero ceremony. No feature gates that ship in release builds.
- Adding a new OS (BSD, Android-on-WSL, ...) is one implementation file plus a selection branch.
- WSL-specific behaviour can be modelled as a `WslPlatform` wrapping `LinuxPlatform`, with only the overridden methods rewritten.

**Makes hard:**
- `&dyn Platform` dynamic dispatch adds a vtable lookup per call. Negligible against filesystem and daemon I/O; measured in nanoseconds.
- Static-dispatch optimisations (inlined trash calls, for example) are not available; if benchmarks in Phase 2 show this matters, a generic-over-`P: Platform` path can be added alongside without removing the dynamic form.

**Defers:**
- WSL and network-mounted filesystem handling to Phase 3 (Cross-platform hardening).
- A `is_in_use(&self, &Path) -> bool` method that would help Gradle / Metro daemon cases. See `docs/ecosystem-fit/java.md` for the motivation; likely lands as ADR-000N during B4.
