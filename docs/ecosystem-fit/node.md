# Ecosystem fit: Node

## 1. Manifest signal

- `package.json` at a project root, or at every workspace member root in a monorepo.
- Monorepo signals: `pnpm-workspace.yaml` (pnpm), `lerna.json` (Lerna), `turbo.json` (Turborepo), `"workspaces"` field in the root `package.json` (npm and yarn). A detector that handles only the root `package.json` misses nested workspace members; a detector that handles every `package.json` double-counts. Monorepos force the detector to understand the workspace layout, not just the manifest.

## 2. Artefacts

- **`node_modules/`** per project or per workspace member. Typically dominant (50 MB to several GB).
- **`.next/`**, **`dist/`**, **`build/`**, **`.turbo/`**, **`.cache/`**, **`coverage/`** are common build or tooling outputs.
- **`.pnpm-store/`** (project-local if not symlinked to the global store).
- **Global caches (under `CacheProvider`):** `~/.npm/_cacache` (npm), `~/.pnpm-store` (pnpm, typically), `~/.yarn/berry/cache` (yarn Berry). Each has its own enumeration mechanism.

## 3. Safety gates

- Uncommitted changes in the repo.
- Presence of a modified `package-lock.json` / `pnpm-lock.yaml` / `yarn.lock` (indicates in-flight dependency work).
- Recent mtime on `node_modules/` (active install; the npm install process touches many files).
- Missing lockfile: reclamation is still safe but flag because restoration may produce a different `node_modules/` than expected.
- No long-running daemon to worry about. The watchers used by dev servers (webpack, vite) do hold file handles on Windows; reclamation during `npm run dev` will fail loudly.

## 4. Trait fit

- **`ProjectDetector`:** fits, with nesting as the complication. The detector must walk into a workspace and enumerate per-member `node_modules/` correctly, not stop at the first `package.json`.
- **`CacheProvider`:** fits for the global stores. pnpm's content-addressable store is especially attractive because deletion is safe and the store rebuilds from the network.
- **`Scorer`:** size-dominated. `node_modules/` entries are enormous relative to builds; age is a secondary signal.
- **`Platform`:** `home()` for cache roots; `trash()` for removal. Windows needs file-handle awareness because dev servers hold open files (see 3 above).
- **`RuleEngine`:** rules encode monorepo behaviour ("reclaim all `node_modules/` in this workspace" versus "only the root").

## 5. Misfits flagged

- **Nested `node_modules/` deduplication.** A monorepo can contain a top-level `node_modules/` plus per-member `node_modules/`. Both are reclaimable, but treating them as independent candidates under-estimates the effective cost (since pnpm may share content) and over-reports the count. The `ProjectDetector` trait currently returns `Vec<Candidate>` with no hint of hierarchy. Option: add an `id: CandidateId` plus an optional `parent: Option<CandidateId>` to allow grouped display in the UI without touching the trait shape. Proposed as amendment ADR in B2.
- **Windows file-handle sensitivity.** The `Platform::trash` signature does not surface a "path is in use" error distinctly. On Windows, `trash` will return an opaque IO error if a dev server holds a handle. A dedicated `Error::PathInUse` variant or a `Platform::is_in_use(&Path) -> bool` probe would improve UX. See ADR-0002 Consequences for the parallel Java/Gradle motivation.
