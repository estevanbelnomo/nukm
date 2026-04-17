# ADR-0003: Detector versus cache provider

Date: 2026-04-17
Status: Accepted

## Context

Most ecosystems Nukm targets are filesystem-walkable: start from a manifest (`Cargo.toml`, `package.json`), find build outputs and local caches in known subpaths, report them as candidates. Rust's `target/`, Node's `node_modules/`, Python's `.venv/`, and Go's `vendor/` all fit this pattern cleanly.

Docker does not. The Docker daemon owns its images, build cache, and volumes. They have a filesystem representation in `/var/lib/docker/` on Linux or a hidden `vhdx` on Windows, but that representation is an implementation detail. Nukm must talk to the daemon's API to enumerate and remove cache entries safely. Filesystem-walking Docker's on-disk storage would be unreliable and risk corrupting the daemon's state.

Two other cases sit in the middle. Python's global `pip` cache has an on-disk location but is best enumerated via `pip cache list`, which knows about ongoing downloads and can skip locked files. Gradle caches have a daemon that may be holding files open.

Options considered:

1. **Single `Detector` trait, generic enough to cover all cases.** Forces Docker to masquerade as a filesystem walker, or forces the trait signature to accept an `Option<&DaemonApi>` that is `None` for non-daemon ecosystems. Either way, every detector gets polluted with the Docker case.
2. **Two traits: `ProjectDetector` (filesystem walker) and `CacheProvider` (ecosystem-native enumeration).** Ecosystems pick whichever trait fits or implement both. Rust can be both: a `ProjectDetector` for local `target/` directories and a `CacheProvider` for the global Cargo registry.
3. **One trait per ecosystem.** Cleanest in isolation; explodes complexity in the rule engine and the CLI dispatch layer.

## Decision

Option 2. `nukm-core` defines two traits:

- `ProjectDetector`: walks from a manifest signal, returns candidates for local artefacts. Signature forces a `&Path` root so the ecosystem-fit docs all answer "what do you do with the path I hand you".
- `CacheProvider`: enumerates via the ecosystem's native mechanism (API, config file, environment variable), returns candidates, and owns the `reclaim` call so the provider can route through its own removal path (for example `docker image rm` instead of filesystem deletion).

Ecosystems may implement either, both, or neither. The Phase 1 mapping is:

| Ecosystem | `ProjectDetector` | `CacheProvider` |
|---|---|---|
| Rust | `target/`, `Cargo.lock` in workspaces | `~/.cargo/registry/cache`, `~/.cargo/git/checkouts` |
| Node | `node_modules/`, `.next/`, `dist/` | `~/.npm/_cacache`, pnpm and yarn stores |
| Python | `.venv/`, `__pycache__`, `build/`, `dist/`, `.tox/`, `.pytest_cache/` | `pip` global cache (via `pip cache`) |
| Go | `vendor/` | `$GOMODCACHE` |
| Java | `target/`, `build/`, `.gradle/` | `~/.m2/repository`, `~/.gradle/caches` |
| Docker | none | daemon API (dangling images, BuildKit cache, dangling volumes) |

See `docs/ecosystem-fit/*.md` for the per-ecosystem detail.

## Consequences

**Makes easy:**
- Docker's daemon interaction is a natural `CacheProvider` implementation with no pretending to be a filesystem walker.
- Adding a provider-native cache tier for any ecosystem is an additive change. Existing `ProjectDetector` users are untouched.
- The CLI dispatch layer iterates two trait-object lists rather than one heterogeneous "thing".

**Makes hard:**
- Ecosystems that span both (Rust, Node, Python, Java) need two implementations and must keep them consistent. A single candidate should not appear in both. Deduplication is the rule engine's responsibility and lands in B1.
- Two traits means two slots in the plugin registry if Phase 5 exposes external rule authors.

**Defers:**
- `CacheProvider` method for incremental enumeration (pagination for very large caches). Not needed before B4.
- A shared base trait or tagging mechanism to let code reason about "all sources of candidates" uniformly. So far not needed; revisit if callers accumulate duplication.
