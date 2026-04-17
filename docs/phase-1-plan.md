# Nukm: Phase 1 plan

> Canonical plan for Phase 1 (Core + CLI MVP). Supersedes the Phase 1 section of `../NUKM_PROJECT.md`. Incorporates v1.1 amendments; see `phase-1-plan-v1.1-amendments.md` for the amendment record.

**Decision:** C then B. Architectural skeleton first (3 days), then vertical slices by ecosystem (27 days).
**Rationale:** Designing a rule engine from a single detector biases the abstractions toward that ecosystem's quirks. Rust's `target/` is unusually clean; Node's `node_modules/` is nested; Docker is not a filesystem scan at all. Validating trait shapes against all six ecosystems on paper is a two-hour exercise that prevents a multi-day refactor mid-phase.

---

## 1. Why C alone is wrong, and why B alone is wrong

**Pure C (design doc first, then implement):** risks becoming waterfall. Spec bloats, never ships, reality is deferred.

**Pure B (vertical slice first, extract abstractions later):** the first slice quietly becomes the architecture. Rule engine shaped by one ecosystem misfits the rest. Docker will not fit a filesystem-walker trait and needs its own `CacheProvider` abstraction. Discovering this in increment 3 costs a full refactor.

**C then B:** paper-validate trait shapes against all six ecosystems, ship the skeleton as compiling Rust (not prose), then iterate with evidence.

## 2. Constraints on the C phase (anti-waterfall guardrails)

1. **Timebox: 3 days.** Arthur enforces. If the skeleton is not compiling and green on CI by end of day 3, cut scope rather than extend.
2. **Scope = interfaces, not algorithms.** In scope: crate boundaries, trait signatures, data types, error taxonomy. Out of scope: scoring formulas, concrete TOML rule schemas, CLI UX.
3. **Six-ecosystem smoke test.** For each of Rust, Node, Python, Go, Java/Gradle, Docker, write a half-page "how would this ecosystem be implemented against these traits?" Find misfits on paper. Docker is expected to fail this test; surfacing that failure is the point.
4. **Everything is provisional v0.** The skeleton is a hypothesis, not a contract. Increment 1 of B is allowed to amend it. Amendments land as dated ADRs in `docs/adr/`.
5. **Spec ships as runnable code, not a document.** `nukm-core` compiles with `todo!()` bodies and doc comments. `cargo test` runs (compile-only sanity tests per crate). CI green on Windows. Reviewable as a PR.

## 3. Phase 1 timeline

### Days 1-3 (C): architectural skeleton
**Lead agent:** Magnus. **Reviewer:** Arthur. **Support:** Kenji (CI, cross-platform-sensitive ecosystem docs), Ingrid (test scaffolding).

Deliverables:
- [ ] `nukm-core/src/lib.rs` with trait definitions and concrete `RuleEngine` struct, no logic
- [ ] Public types: `Candidate`, `ScanResult`, `Score`, `SafetyGate`, `Ecosystem`, `ActionManifest`, `ActionRecord`
- [ ] Traits: `ProjectDetector`, `CacheProvider`, `Scorer`, `Platform`
- [ ] Concrete types: `RuleEngine` (narrow public API, not a trait)
- [ ] Error taxonomy (`thiserror`-based)
- [ ] `nukm-cli/src/main.rs` with command stubs: `scan`, `clean`, `rules`, `doctor`
- [ ] Workspace `Cargo.toml`, MSRV pinned, `rust-toolchain.toml`
- [ ] CI skeleton (`.github/workflows/ci.yml`): fmt, clippy, test, build on **Windows only**
- [ ] `docs/adr/0001-crate-boundaries.md`, `0002-platform-trait.md`, `0003-detector-vs-cache-provider.md`
- [ ] `docs/ecosystem-fit/{rust,node,go,java}.md` (Magnus)
- [ ] `docs/ecosystem-fit/{docker,python}.md` (Kenji)
- [ ] `tests/` structure, `tests/common/mod.rs` with `assert_fs` fixture helper stubs, one compile-only sanity test per crate (Ingrid)
- [ ] `cargo-nextest` config if adopted; decide in C and record as ADR-0004 or defer
- [ ] `ROADMAP.md` at repo root as multi-phase index pointing to this plan

**`ActionManifest` scope guardrail.** In C: field definitions, `serde` serialisation contract (JSON), doc comments describing intent. Deferred to B1: on-disk persistence format, storage location, naming, rotation / retention, concurrency and locking.

**Exit criteria:**
- `cargo build --workspace` green
- `cargo test --workspace` green (compile-only sanity tests pass)
- `cargo clippy --workspace -- -D warnings` green
- CI green on Windows
- All six ecosystem-fit docs written and cross-reviewed
- At least one ADR records a design change caused by the smoke test

### Days 4-10 (B1): Rust detector end-to-end plus cross-platform CI
**Lead agent:** Magnus. **Support:** Ingrid (tests), Kenji (CI expansion).

Deliverables:
- [ ] `ProjectDetector` impl for Rust (`Cargo.toml` manifest, `target/` artefact)
- [ ] Filesystem-age scorer
- [ ] Git safety gate via `git2` (clean state check, stash check)
- [ ] `trash` crate integration
- [ ] Dry-run default, `--execute` flag
- [ ] `nukm scan` and `nukm clean` for Rust only
- [ ] Integration tests with `assert_fs` fixtures
- [ ] `tests/corpus/` scaffold; Rust fixtures (clean, dirty-git, stashed, ancient > 90d, recent < 7d)
- [ ] `ActionManifest` persistence strategy (on-disk format, location, rotation) implemented and documented
- [ ] CI matrix expanded to Windows, macOS, Linux; all three runners green before B1 tag
- [ ] `proptest` introduced with its first real caller
- [ ] Tag `0.1.0-alpha`

**Exit criteria:** `nukm scan P:\Work\Programming` on the real machine produces correct, actionable output; CI matrix green on all three platforms.

### Days 11-17 (B2): Node detector
**Lead agent:** Magnus. **Support:** Arthur (abstraction review), Ingrid (fixtures).

Deliverables:
- [ ] `ProjectDetector` impl for Node (`package.json`, `node_modules/`)
- [ ] First exercise of the rule engine's generality
- [ ] Amendment ADRs if the C-phase trait shape misfits Node
- [ ] Node fixtures in `tests/corpus/` (equivalent coverage to B1)
- [ ] `insta` snapshot testing introduced with its first real caller
- [ ] Tag `0.2.0-alpha`

**Exit criteria:** Rust and Node coexist in one codebase without special-casing either in the CLI layer.

### Days 18-24 (B3): Python + Go
**Lead agent:** Magnus. **Support:** Ingrid (fixtures).

Deliverables:
- [ ] Python: `pyproject.toml`, `__pycache__`, `.venv`, `build/`, `dist/`, `.tox/`, `.pytest_cache/`
- [ ] Go: `go.mod`, module cache awareness
- [ ] Two more ecosystems stress-testing the abstractions
- [ ] Python and Go fixtures in `tests/corpus/`

### Days 25-30 (B4): Java/Gradle + Docker, Phase 1 close
**Lead agent:** Magnus. **Support:** Kenji (Docker daemon interaction), Viktor (benchmark writeup).

Deliverables:
- [ ] Java/Gradle detector
- [ ] Docker via daemon API (dangling images, build cache), not filesystem walk
- [ ] The `CacheProvider` split predicted on paper lands here
- [ ] Java/Gradle fixtures in `tests/corpus/`
- [ ] Docker fixtures (scripted daemon state, not filesystem)
- [ ] Kondo comparison writeup at `docs/benchmarks/phase-1-vs-kondo.md` (Viktor)
- [ ] Tag `0.3.0-alpha` as Phase 1 completion
- [ ] Phase 1 retrospective in `docs/retrospectives/phase-1.md`

**Phase 1 exit criteria:**
- Six ecosystems working end-to-end
- Zero false positives on the seeded test corpus
- Beats `kondo` on false-positive rate in the documented comparison
- All abstractions shaped by evidence, not speculation

## 4. Agent assignments for Phase 1

| Agent | Phase 1 role |
|---|---|
| Arthur | Orchestrator. Enforces 3-day C-phase timebox. Enforces non-goals. Reviews all ADRs. Socratic with Estevan on every architectural decision. |
| Magnus | Primary implementer of `nukm-core` and detectors. Owns trait design and refactors driven by ADRs. Writes Rust, Node, Go, Java ecosystem-fit docs. |
| Kenji | CI matrix ownership from day 1 (Windows in C, matrix in B1). Writes Docker and Python ecosystem-fit docs. Consulted in B4 for Docker daemon integration. |
| Luca | Dormant in Phase 1. GUI work starts in Phase 4. |
| Ingrid | Test infrastructure from day 1 (scaffolding in C, assertions from B1). Enforces "no feature without tests" from B1 onward. Owns `tests/corpus/` across all increments. |
| Viktor | Writes the kondo comparison benchmark post at Phase 1 close. Dormant otherwise. |

## 5. ADR workflow

Every architectural decision, including those made in the C phase, lands as a numbered ADR under `docs/adr/`. Format:

```markdown
# ADR-NNNN: <title>

Date: YYYY-MM-DD
Status: Proposed | Accepted | Superseded by ADR-MMMM

## Context
<one paragraph: what problem or question>

## Decision
<one paragraph: what we chose>

## Consequences
<bullets: what this makes easy, what this makes hard, what we defer>
```

Amendments during B phases are new ADRs that supersede earlier ones, never edits.

## 6. Execution script

Arthur executes these steps in order:

1. Land the planning set (this file, `phase-1-plan-v1.1-amendments.md`, `ROADMAP.md`, placeholder READMEs under `docs/adr/` and `docs/ecosystem-fit/`, handoff blocks under `docs/handoffs/`) as a single `docs:` commit.
2. Hand off to Magnus for his portion of the C-phase skeleton (core crate, CLI stubs, Rust/Node/Go/Java ecosystem-fit docs, ADR-0001 through ADR-0003).
3. Hand off to Kenji in parallel for Docker and Python ecosystem-fit docs, plus CI skeleton (Windows only).
4. Hand off to Ingrid in parallel for `tests/` scaffolding, compile-only sanity tests, and the `cargo-nextest` adoption decision (ADR-0004 or deferral note).
5. Once all three hand back, Arthur runs the six-ecosystem smoke test across the merged skeleton and flags every misfit as a proposed amendment ADR.
6. Stop. Wait for Estevan's review and approval before any B-phase code.

**Rules that still apply:**
- British English, no em dashes, no teaching tone.
- Estevan writes the first draft of any architectural decision. Arthur reviews.
- Non-goals from `../NUKM_PROJECT.md` Section 4 are binding.
- Dry-run default is non-negotiable from B1 onward.
- Timebox on C-phase is 3 days. If not green by end of day 3, Arthur cuts scope rather than extending.
