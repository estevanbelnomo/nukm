# Ecosystem fit: Docker

## 1. Manifest signal

Docker has **no filesystem manifest**. The signal that Docker is "present" is the availability of a daemon socket:

- **Linux:** `/var/run/docker.sock` (rootful) or `$XDG_RUNTIME_DIR/docker.sock` (rootless), or `podman.sock` when Podman is the chosen runtime.
- **Windows:** named pipe `\\.\pipe\docker_engine` when Docker Desktop is installed and running. The underlying storage lives inside a managed WSL2 distribution (`docker-desktop-data`), invisible to the host filesystem.
- **macOS:** `~/.docker/run/docker.sock` (Docker Desktop) or Colima-provided alternatives.

There is no `docker.toml` or equivalent at a project root. The daemon owns its state. A project-level `Dockerfile` is not a signal to reclaim against; users may have hundreds of Dockerfiles with zero relation to daemon state.

## 2. Artefacts

- **Dangling images** (`docker images -f dangling=true`): untagged images that no tagged image references. Always safe to reclaim.
- **Unused build cache** (`docker builder prune`): BuildKit layer cache not referenced by any currently-tagged image. Can be enormous (tens of GB on active machines).
- **Unused volumes** (`docker volume ls -f dangling=true`): volumes not attached to any container. Often contain data; reclamation must be behind an explicit opt-in flag.
- **Stopped containers** (`docker ps -a -f status=exited`): dead containers taking filesystem space. Generally safe but can hold useful logs; reclaim with warning.
- **Unused networks:** trivial space impact; skip in Phase 1.

These are **not filesystem paths** from the host's perspective. They are daemon-managed objects addressed by SHA256 digest, name, or volume identifier. Nukm cannot `walkdir` into `/var/lib/docker/` and reclaim safely; doing so corrupts the daemon's BoltDB metadata.

## 3. Safety gates

- **Running containers:** any image referenced by a running container must not be reclaimed, even if it appears dangling under strict interpretation.
- **Pinned tags:** images the user has tagged deliberately (non-`<none>`) must not be reclaimed even if unreferenced. Only `<none>:<none>` dangling images are candidates by default.
- **Active builds:** BuildKit cache in use by an in-progress build. The daemon reports this via the `Idle` flag on builder state; consult before pruning.
- **Named volumes:** reclamation must require explicit opt-in. Named volumes usually contain data the user named for a reason.
- **Docker Desktop running:** on Windows and macOS, the daemon is hosted inside a VM or WSL distribution. Reclaim calls proceed through the daemon API; the VM is off-limits to direct manipulation.

## 4. Trait fit

- **`ProjectDetector`:** does not fit. There is no manifest to walk from and no per-project directory to enumerate. A Docker detector returning candidates based on a `Dockerfile` would produce candidates unrelated to the user's actual daemon state. The trait is inapplicable.
- **`CacheProvider`:** the primary fit, and the reason this trait exists. Docker implements `enumerate` by calling the daemon API (or shelling out to `docker image ls --filter dangling=true`, `docker builder prune --dry-run`, `docker volume ls --filter dangling=true`). `reclaim` routes through `docker image rm`, `docker builder prune -f`, `docker volume rm`. Filesystem deletion is never used.
- **`Scorer`:** size-dominated. Age is reported by the daemon per object. Meaningful ordering.
- **`Platform`:** stressed hard. The trait currently has no notion of "daemon present" or "daemon reachable". A Docker `CacheProvider` must either (a) accept that enumeration returns an error on machines without Docker and surface that cleanly to the user, or (b) the `Platform` trait gains a capability-probing method. Option (a) is simpler.
- **`RuleEngine`:** rules encode "skip pinned tags", "require --include-volumes for named volumes", "skip images referenced by running containers".

## 5. Misfits flagged

- **Runtime selection is not in the trait.** Docker, Podman, `nerdctl`, and Colima all present Docker-compatible APIs but differ in socket location, authentication, and quirks. `CacheProvider::enumerate(&self, platform: &dyn Platform)` has no obvious place to express "use socket at /custom/path" or "use Podman instead of Docker". Options:
  - Accept that each provider implementation hardcodes its default socket discovery and exposes a constructor for custom paths outside the trait surface.
  - Widen the trait to `fn enumerate(&self, platform: &dyn Platform, config: &ProviderConfig)` with a generic config bag. Increases boilerplate for the 90% case where defaults work.
  - Introduce a `DaemonLocator` helper the provider uses internally. Locator is platform-aware (checks `$DOCKER_HOST`, default socket paths, Docker Desktop presence).

  Recommendation: amendment ADR proposing `DaemonLocator` in B4 when the Docker provider lands. Not blocking Phase 1 since no other ecosystem needs daemon-selection yet.

- **Cross-platform runtime variance is extreme.**
  - Linux: rootful vs rootless; `/var/run/docker.sock` perms require group membership.
  - Windows: Docker Desktop uses a WSL2 VM; CLI on the host talks to it via the pipe. Podman Desktop is an alternative with a similar model.
  - macOS: Docker Desktop VM; Colima; `OrbStack` as a newer option. Lima underlying several of these.

  A Windows user who has never installed Docker but has WSL installed may still have an old `docker.exe` on the PATH. The provider must probe for a live daemon, not the binary. `cargo` must not assume availability.

- **BuildKit cache introspection.** BuildKit is the default builder since Docker 23.0; its cache format differs from classic image layer cache. The `docker system df` output summarises both, but `docker builder prune --dry-run` only addresses BuildKit. Dual-cache handling must be explicit in the provider.

- **No per-object mtime equivalent.** Image and container metadata carry a `Created` timestamp, not `LastUsed`. Age-based scoring works on creation age, not on dormancy age, which is a weaker signal than for filesystem ecosystems. Flag in `docs/ecosystem-fit/README.md` review: consider whether `Candidate::last_modified` is the right name given that some providers cannot distinguish "modified" from "created".
