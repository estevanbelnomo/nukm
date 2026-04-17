# Handoff: Arthur to Kenji

## Context

- Phase: Phase 1, C-phase (Days 1-3)
- Crate: cross-cutting (CI) and `docs/ecosystem-fit/`
- Files: `.github/workflows/ci.yml`, `docs/ecosystem-fit/docker.md`, `docs/ecosystem-fit/python.md`

## State

- **Done:** CI skeleton present at `.github/workflows/ci.yml` (fmt + clippy + test on `windows-latest`; `cargo-audit` on `ubuntu-latest`). MSRV 1.85 pinned via `rust-toolchain.toml`. Phase 1 plan approved.
- **In progress:** nothing. This handoff starts the C-phase.
- **Blocked:** none.

## Request

Write the two ecosystem-fit docs you own (`docker`, `python`) per the schema in `docs/ecosystem-fit/README.md`, and confirm the CI workflow is appropriate for C-phase without expansion. Flag any CI changes that should land before B1.

## Acceptance criteria

**`docs/ecosystem-fit/docker.md`:**
- Answers the five schema questions: manifest signal, artefacts, safety gates, trait fit, misfits.
- Must flag at least one misfit against the `ProjectDetector` trait (Docker is not a filesystem walk; expect this to motivate `CacheProvider`).
- Addresses whether Docker state is read via the daemon API (`bollard` or similar) or the Docker CLI shell-out, and what that means for the `Platform` trait.
- Addresses cross-platform quirks: Docker Desktop on Windows vs `dockerd` on Linux vs Colima on macOS.

**`docs/ecosystem-fit/python.md`:**
- Answers the five schema questions.
- Must flag at least one Windows-vs-Unix divergence that stresses the `Platform` trait. Minimum coverage: `.venv` layout differences (`Scripts/` vs `bin/`), pip cache location (`%LOCALAPPDATA%\pip\cache` vs `~/.cache/pip`), pyc cache granularity.
- Covers `pyproject.toml`, `__pycache__`, `.venv`, `build/`, `dist/`, `.tox/`, `.pytest_cache/` as minimum detectable artefacts.

**CI workflow (`.github/workflows/ci.yml`):**
- Stays Windows-only for C-phase. `cargo-audit` on `ubuntu-latest` is retained since it does not build the crate.
- Any pre-B1 change you believe necessary (caching tweaks, concurrency, permissions) is applied minimally and justified in the commit message.
- No macOS or Linux build runners added in C. That expansion is a B1 deliverable triggered by `git2`.

## Return path

Return to Arthur. Expected follow-up: Arthur cross-reviews your two docs alongside Magnus's four (`rust`, `node`, `go`, `java`) for consistency, then runs the six-ecosystem smoke test and produces amendment ADRs for any misfit that demands a trait change.
