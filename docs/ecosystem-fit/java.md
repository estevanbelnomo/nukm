# Ecosystem fit: Java

## 1. Manifest signal

- **Maven:** `pom.xml` at the project root. Multi-module projects have a parent `pom.xml` with `<modules>` entries; each module also has its own `pom.xml`.
- **Gradle:** `build.gradle` or `build.gradle.kts` at the project root; `settings.gradle[.kts]` marks a multi-project root.
- **Kotlin DSL variants** are functionally equivalent.

## 2. Artefacts

- **Maven per-project:** `target/` (Maven's output directory, unrelated to Rust's `target/` except by name). Contains compiled classes, packaged JARs, reports.
- **Gradle per-project:** `build/` for compiled output; `.gradle/` for Gradle's per-project state including the daemon's working files, lock files, and cached classpath resolutions.
- **Gradle daemon working state:** `.gradle/<version>/` directories hold state keyed by Gradle version; wrong to delete while a daemon is running.
- **Global caches (under `CacheProvider`):**
  - `~/.m2/repository/` (Maven local repository): downloaded dependencies. Can exceed several GB.
  - `~/.gradle/caches/`: Gradle's equivalent, plus cached resolution data and transformed artefacts. Often larger than `.m2`.
  - `~/.gradle/daemon/`: daemon logs and state across Gradle versions.
  - `~/.gradle/wrapper/dists/`: downloaded Gradle distributions per version. Accumulates silently.

## 3. Safety gates

- Uncommitted changes in the repo.
- **Gradle daemon is running.** This is the live wire. Running `./gradlew build` starts a persistent daemon (`org.gradle.daemon.GradleDaemon`) that holds file handles on `.gradle/` and parts of `~/.gradle/caches/`. Deleting these while the daemon is alive corrupts its state and necessitates a `./gradlew --stop`. Reclamation must check for running daemons (`ps`, `tasklist`, or the Gradle tooling API) before touching `.gradle/` or `~/.gradle/caches/`.
- **Maven daemon (`mvnd`)** has the same problem if adopted.
- IDE activity: IntelliJ IDEA re-indexes after `~/.gradle/caches/` is cleared. Not a blocker, but worth warning the user.

## 4. Trait fit

- **`ProjectDetector`:** fits for `target/` (Maven) and `build/` (Gradle). `.gradle/` per-project is candidate material but gated behind daemon-running.
- **`CacheProvider`:** the strongest fit for Java. Gradle and Maven global caches are both enormous and reliably rebuildable.
- **`Scorer`:** size-dominated for global caches, age-dominated for `build/` and `target/`.
- **`Platform`:** cache roots under `home()`; `trash()` for removal. Missing: a mechanism to ask "is the Gradle daemon running?".
- **`RuleEngine`:** must encode "do not reclaim `.gradle/` if a `GradleDaemon` process exists" as a safety gate. This is a process-presence check, not a filesystem or git check.

## 5. Misfits flagged

- **Daemon-running detection is missing from `Platform`.** ADR-0002 called this out as deferred; Java makes the motivation concrete. Proposed `Platform` extension:
  ```rust
  fn is_process_running(&self, name: &str) -> Result<bool>;
  ```
  or, more robustly, a first-class `ProcessProbe` trait. Recommended as an amendment ADR during B4 when the Gradle detector lands.
- **`.gradle/<version>/` granularity.** Reclaiming `.gradle/` as a whole loses IDE integration caches that are expensive to rebuild. The `ArtefactKind::DaemonState` variant exists partly to let rules target daemon state distinctly from build output. Gradle is the ecosystem that exercises this distinction first.
- **Wrapper distributions accumulate invisibly.** `~/.gradle/wrapper/dists/` holds every Gradle version the wrapper has ever fetched, including old release candidates. No upstream tool prunes this. A `CacheProvider` method for "entries older than N" would be convenient but is not strictly required for Phase 1.
