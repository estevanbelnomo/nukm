# Ecosystem-fit docs

One half-page document per ecosystem supported by Nukm, answering:

**How would this ecosystem be implemented against the current `nukm-core` traits?**

The purpose is to surface misfits on paper before any code lands. If a trait signature cannot cleanly express an ecosystem's reality, the misfit is documented here and triggers a proposed amendment ADR.

## Phase 1 scope

Six ecosystems, each a separate document:

- `rust.md` - Magnus
- `node.md` - Magnus
- `go.md` - Magnus
- `java.md` - Magnus
- `docker.md` - Kenji (expected primary `CacheProvider` stress test)
- `python.md` - Kenji (expected `Platform` trait stress test, Windows vs Unix divergence)

Arthur cross-reviews all six for consistency before the C-phase exit check.

## Format

Each document is approximately half a page and covers:

1. **Manifest signal.** What file(s) mark a project of this ecosystem?
2. **Artefacts.** What directories or files are candidates for reclamation?
3. **Safety gates.** What makes reclamation unsafe (uncommitted changes, active processes, shared caches, ...)?
4. **Trait fit.** Which of `ProjectDetector`, `CacheProvider`, `Scorer`, `Platform`, `RuleEngine` does this ecosystem exercise, and are the current signatures sufficient?
5. **Misfits flagged.** Anything that does not fit cleanly. These become amendment ADRs.

## Relationship to ADRs

Ecosystem-fit docs describe ecosystems. ADRs describe decisions. Misfits documented here motivate ADRs; ADRs do not restate ecosystem reality.
