# Ecosystem fit: Python

## 1. Manifest signal

Python has more manifest candidates than any other ecosystem Nukm targets:

- **`pyproject.toml`** (PEP 518, PEP 621): the modern standard. Used by setuptools, poetry, hatch, flit, pdm, uv.
- **`setup.py`** and **`setup.cfg`**: legacy setuptools. Still common.
- **`requirements.txt`**: loose signal; may appear without a packaging manifest at all. Reasonable projects have it alongside `pyproject.toml`.
- **`Pipfile` / `Pipfile.lock`**: Pipenv projects.
- **`environment.yml`** / **`conda-env.yml`**: Conda projects. Conda is a separate ecosystem in all but name and is out of scope for Phase 1.
- **Absence of any manifest with a `.venv/`** or **`venv/`** at the root: a common "I just ran `python -m venv`" workflow that has no formal manifest but still has reclaimable caches.

A detector must accept any of the above as a signal and not double-count when multiple coexist (for example, `pyproject.toml` + `requirements.txt` is common).

## 2. Artefacts

- **`.venv/`**, **`venv/`**, **`env/`**: project-local virtual environments. Dominant reclamation target. Typical size 50 MB to 2 GB depending on dependencies.
- **`__pycache__/`**: per-directory bytecode cache. Many small directories scattered through the source tree. Individually small; aggregate reclaim worthwhile.
- **`.pytest_cache/`**: pytest test cache and last-run state.
- **`.tox/`**: tox test matrix environments. Often contains multiple `.venv`-equivalent trees.
- **`.nox/`**: nox (tox alternative) equivalent.
- **`.mypy_cache/`** and **`.ruff_cache/`**: static analysis caches. Safe to delete; regenerate on next run.
- **`build/`**, **`dist/`**, **`*.egg-info/`**: packaging output from `python -m build` or `python setup.py sdist bdist_wheel`.
- **Global cache (under `CacheProvider`):** pip download cache. Location varies by platform (see section 5). `pip cache list` is the canonical enumeration mechanism.
- **Poetry / uv / pdm caches:** each tool has its own store. Out of scope for Phase 1 project detector; address in Phase 2 under `CacheProvider` if the tool is detected.

## 3. Safety gates

- **Active venv.** The `VIRTUAL_ENV` environment variable points at the active venv. Deleting the one currently in use breaks the user's shell. The detector must check `VIRTUAL_ENV` and gate any `.venv/` that matches.
- **Running Python processes holding open files** in the venv. Windows will refuse deletion; Unix will silently remove files but leave running processes with stale handles until they exit. Neither is safe during an active run.
- **Editable installs pointing into the project.** `pip install -e .` creates an egg-link back to the source tree. Removing `build/` or `*.egg-info/` can break the editable install until the user runs `pip install -e .` again. Warn, do not auto-reclaim.
- **Uncommitted changes** in a poetry / pdm-managed project that tracks `poetry.lock` in git.
- **Custom `PYTHON*` environment variables** pointing inside the project. Rare but possible.

## 4. Trait fit

- **`ProjectDetector`:** fits, with the caveat that Python has *many* manifest signals. The detector must enumerate all candidates without double-counting when multiple manifests coexist. Returning a deduplicated `Vec<Candidate>` keyed by canonical path is the minimum; the trait is sufficient.
- **`CacheProvider`:** fits for the pip global cache. Path discovery is platform-specific; see section 5.
- **`Scorer`:** size-dominated for `.venv/`, age-dominated for `__pycache__/` and `.pytest_cache/`.
- **`Platform`:** stressed more than any other Phase 1 ecosystem. Windows vs Unix divergence is not just path separators; it is entire sub-layout differences. See section 5.
- **`RuleEngine`:** rules distinguish "safe to delete always" (`__pycache__`, `.ruff_cache`) from "safe to delete unless active" (`.venv/`) from "warn first" (`build/`, `*.egg-info/` near an editable install).

## 5. Misfits flagged

- **`.venv/` layout differs between Windows and Unix.** On Unix the executables live at `.venv/bin/python`; on Windows they live at `.venv/Scripts/python.exe`. The `Platform` trait currently offers no method to resolve "where is the venv executable for a given venv root". Candidates in a venv are usually reclaimed whole so this rarely matters for deletion, but `doctor` output, safety gating ("is the venv active?"), and scorer heuristics will need it. Proposed amendment: `Platform::venv_bin_dir(&self, venv_root: &Path) -> PathBuf` or a broader `platform.python().bin_dir(venv_root)` sub-trait in Phase 2.

- **Pip cache location varies by OS.** Three different default paths:
  - Linux: `$XDG_CACHE_HOME/pip` or `~/.cache/pip`
  - macOS: `~/Library/Caches/pip`
  - Windows: `%LOCALAPPDATA%\pip\Cache`
  
  Overridable by `PIP_CACHE_DIR`. `Platform::global_cache_roots()` currently returns `Vec<PathBuf>`, which is adequate once the platform implementation knows to include the pip cache path, but the ecosystem-fit contract is: the Python `CacheProvider` asks the Platform for "the pip cache root", not for a fixed subdirectory. Either add `Platform::pip_cache_root()` (narrow, grows the trait) or make Python's `CacheProvider` shell out to `pip cache dir` (robust, requires `pip` on the PATH). Recommendation: shell out to `pip cache dir` in B3 since it handles `PIP_CACHE_DIR` and virtualenv-specific caches correctly without trait growth.

- **`__pycache__/` directories are fractal.** Every directory with `.py` files contains a `__pycache__/`. A medium project has hundreds of them. Returning each as a separate `Candidate` is cacophonous; returning "all `__pycache__/` under this project root" as a single synthetic candidate with aggregated `size_bytes` is cleaner for the UI but requires a `Candidate` variant that represents a set of paths, not a single path. Proposed B3 amendment: widen `Candidate::path` to `PathBuf` or a new `Location` enum permitting `Paths(Vec<PathBuf>)`. Alternatively, surface this as a UI-layer concern and keep `Candidate` one-per-path at the core. Recommendation: keep the core model one-per-path; let `nukm-cli` and the GUI group for display.

- **Conda is a parallel universe.** `environment.yml` signals a Conda project, but Conda manages its own environments at `~/miniconda3/envs/<name>/` or equivalent, not in the project tree. Phase 1 explicitly defers Conda. Flag here so the decision is visible in the ecosystem-fit docs rather than buried in a plan footnote.
