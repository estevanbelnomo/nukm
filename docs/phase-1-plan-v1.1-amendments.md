# Nukm: Phase 1 plan amendments (v1.1)

> Historical amendment record against Phase 1 plan v1. Retained for traceability; the current canonical plan lives at `phase-1-plan.md`, which already incorporates every amendment below.

**Status:** accepted amendments, incorporated into canonical plan 2026-04-17.
**Relationship to canonical plan:** this file was the overlay that produced `phase-1-plan.md` from the original v1. Do not treat this file as a live planning document; treat it as the audit trail for why the plan reads as it does.

---

## 1. Summary of decisions

| # | Flag | Decision |
|---|---|---|
| 1 | CI matrix on day 3 too ambitious | Accept. Windows-only in C. macOS + Linux land in B1. |
| 2 | `RuleEngine` as trait is premature abstraction | Accept. Concrete `pub struct RuleEngine` in C. |
| 3 | `ActionManifest` missing from C public types | Accept with caveat. Define type shape in C, defer persistence strategy to B1. |
| 4 | Ingrid + "zero tests in C" tension | Accept, partial. Fixture scaffolding and compile-only sanity tests in C. No `proptest` or `insta` until their first real caller. |
| 5 | Test corpus has no owner or build step | Accept with correction. Corpus starts in B1, not B3. Ingrid owns it across all increments. |
| 6 | `ROADMAP.md` home | Accept option (b). `ROADMAP.md` as multi-phase index; detail lives in `docs/phase-N-plan.md`. |
| 7 | Ecosystem-fit doc ownership (added) | Split: Magnus writes Rust, Node, Go, Java. Kenji writes Docker and Python. |

## 2. Amendments to v1

### 2.1 C-phase CI scope

v1 Section 3 "Days 1-3 (C)" CI line replaced with:

- [ ] CI skeleton (`.github/workflows/ci.yml`): fmt, clippy, test, build on **Windows only**

Corresponding exit criterion replaced:

- CI green on Windows

v1 Section 3 "Days 4-10 (B1)" gained:

- [ ] CI matrix expanded to Windows, macOS, Linux (triggered by `git2` / libgit2 integration)
- [ ] All three runners green before B1 tag

**Rationale:** `git2` links against libgit2 and is platform-sensitive. Co-locating matrix expansion with its first real stressor avoids burning C-phase time on cross-platform linking issues before any logic exists.

### 2.2 `RuleEngine` is a concrete struct

v1 Section 3 "Days 1-3 (C)" public types list amended:

- Traits: `ProjectDetector`, `CacheProvider`, `Scorer`, `Platform`
- Concrete types: `RuleEngine` (struct with narrow public API, not a trait)

**Rationale:** the rule engine is deterministic. Parse TOML, evaluate conditions, return a decision. There is no foreseeable second implementation. If a plugin system in Phase 5 demands trait-ification, it lands then as an amendment ADR. Today it is architecture astronomy.

### 2.3 `ActionManifest` in C public types

v1 Section 3 "Days 1-3 (C)" public types list gained:

- `ActionManifest` (type only), `ActionRecord` (type only)

Scope guardrail, **in C:**
- Field definitions
- `serde` serialisation contract (JSON)
- Doc comments describing intent

Scope guardrail, **deferred to B1:**
- Persistence strategy (file format on disk, location, naming)
- Rotation / retention policy
- Concurrency and locking

**Rationale:** the type crosses detector to CLI to filesystem boundaries and defining its shape once is cheap. Defining persistence in C would expand the timebox into "where do manifests live, how many do we keep, how do we prune", which is exactly the rabbit hole the 3-day window exists to prevent.

### 2.4 Ingrid in C, scoped

v1 zero-tests posture replaced with these C-phase deliverables under Ingrid:

- [ ] `tests/` directory structure across all crates
- [ ] `tests/common/mod.rs` with `assert_fs` fixture helper stubs (empty bodies with doc comments)
- [ ] One compile-only sanity test per crate: `#[test] fn it_compiles() {}`
- [ ] `cargo-nextest` config if adopted (decide in C)

**Explicitly out of scope in C:**
- `proptest` setup (lands in B1 with its first real caller)
- `insta` snapshot setup (lands in B2 with its first real caller)
- Coverage tooling (Phase 2 at earliest)

**Rationale:** B1's first act should be writing an assertion, not wiring a framework. Unused test infrastructure rots; infrastructure added with its first caller does not.

### 2.5 Test corpus: start in B1, Ingrid owns

Per-increment ownership model replaces the v1 B3-only corpus deliverable.

- B1: `tests/corpus/` scaffold, Rust fixtures (clean, dirty-git, stashed, ancient > 90d, recent < 7d).
- B2: Node fixtures.
- B3: Python and Go fixtures.
- B4: Java/Gradle fixtures, Docker fixtures (scripted daemon state, not filesystem).

Owner across all increments: Ingrid. Consumer at Phase 1 close: Viktor, for the kondo comparison writeup.

**Rationale:** starting the corpus in B3 would mean hand-testing B1 and B2 against Estevan's personal filesystem. Slow, non-reproducible, and the Phase 1 exit criterion ("beats kondo on false-positive rate in a documented comparison") would be unverifiable without the corpus existing before the comparison runs.

### 2.6 `ROADMAP.md` as multi-phase index

`ROADMAP.md` is a one-screen multi-phase index. Phase-1 detail lives at `docs/phase-1-plan.md`. Future phases follow the same pattern as they become active. Distant phases remain as bullets in `../NUKM_PROJECT.md` Section 9 until promoted.

**Rationale:** granularity varies per phase. The roadmap stays scannable; detail lives alongside the ADRs and ecosystem-fit docs it references.

### 2.7 Ecosystem-fit doc ownership (new amendment)

- Magnus writes: `docs/ecosystem-fit/{rust,node,go,java}.md`
- Kenji writes: `docs/ecosystem-fit/{docker,python}.md`

**Rationale:** Docker is a daemon API interaction, not a filesystem walk, and is the primary stress test for separating `ProjectDetector` from `CacheProvider`. Python's venv and cache layout diverges sharply between Windows and Unix, stressing the `Platform` trait. Both are exactly where detector/platform abstractions rub against each other. Kenji's lens catches misfits Magnus's will not. Arthur reviews all six for consistency before the C-phase review.
