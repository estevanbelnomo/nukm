# Ecosystem fit: Go

## 1. Manifest signal

- `go.mod` at the module root. Modules are flat (no workspace nesting in the Cargo / Node sense), though `go.work` files (Go 1.18+) compose multiple modules.
- `go.work` presence indicates a multi-module workspace; all listed modules are in scope.

## 2. Artefacts

- **`vendor/`** when present: per-project copy of dependencies. Optional; many modern Go projects do not vendor.
- No standard per-project build directory. `go build` emits the binary in the current working directory by default; `go test` uses a temporary directory that is cleaned up automatically. This is a real difference from Rust and Node.
- **Global module cache (under `CacheProvider`):** `$GOMODCACHE`, defaulting to `$GOPATH/pkg/mod` which defaults to `~/go/pkg/mod`. Entries are read-only; deletion is safe because Go re-downloads from the proxy.
- **Build cache (under `CacheProvider`):** `$GOCACHE`, defaulting to `~/.cache/go-build` on Linux, `~/Library/Caches/go-build` on macOS, `%LocalAppData%\go-build` on Windows. Can grow large on CI machines. Safe to delete (`go clean -cache`).

## 3. Safety gates

- Uncommitted changes in the repo.
- Go workspaces (`go.work`) referenced from multiple modules: the module cache should be treated as shared across the workspace.
- Active Go processes: Go does not run a daemon, but `gopls` (the language server) is commonly running from editors. It holds read handles on the module cache on Windows; reclamation can fail if `gopls` is active.

## 4. Trait fit

- **`ProjectDetector`:** workable but weak. The trait expects local artefacts to reclaim. `vendor/` is the only real candidate, and most projects do not use it. Returns mostly empty results.
- **`CacheProvider`:** the stronger fit. Both `GOMODCACHE` and `GOCACHE` are straightforward to enumerate.
- **`Scorer`:** module cache entries are individually small but collectively large; size-dominated aggregate. Build cache is age-dominated.
- **`Platform`:** `home()` for cache paths; platform-specific logic for `GOCACHE` default (`%LocalAppData%` on Windows versus `~/.cache` on Linux) is exactly what the trait exists to hide.
- **`RuleEngine`:** rules distinguish module cache (always safe) from build cache (safe but may slow next build noticeably).

## 5. Misfits flagged

- **Per-project artefacts are the minority case.** `ProjectDetector`'s assumption that "walking from a manifest finds local reclaim targets" is weak for Go. The detector will largely return no candidates while the `CacheProvider` does the real work. This is not a trait-shape bug; it is a genuine characteristic of Go. Worth documenting so users do not read `nukm scan ~/Work/Go-project` and conclude Nukm is broken when it reports nothing local.
- **Recommendation for B3:** the CLI's `scan` output should surface global caches for ecosystems where `ProjectDetector` returns nothing, not just for ecosystems the user's current path happens to match. Not a trait change; a CLI UX design note.
