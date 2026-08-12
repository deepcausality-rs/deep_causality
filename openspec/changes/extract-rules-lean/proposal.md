# Extract the Lean rules into a standalone MIT ruleset, released through BCR

## Why

The Lean layer is the machine-checked foundation of this project, so it runs on every PR — and
until this week every one of those runs re-fetched 7.5 GB of Lake/Mathlib, because `@lake_deps` was
built by `git clone` and `lake exe cache get` inside `rctx.execute`. Bazel's repository cache and
`--experimental_remote_downloader` both key on sha256 and cannot see into a subprocess, so nothing
could serve it, and the repo it produced was not byte-reproducible, which kept Lean action results
from being shared across machines.

That is now fixed, in a vendored copy of `tomato-bazel/rules_lean` at `thirdparty/rules_lean/`. The
fixes are ~180 lines of patch carrying real measurements behind them, and they are stranded: the
upstream maintainer is unresponsive, the vendored copy cannot receive upstream fixes, and nobody
else can use the work. A survey of the alternative (`pulseengine/rules_lean`) found it solves two of
the four problems independently, leaves the expensive one unsolved, and ships no LICENSE file at
all — so it cannot be vendored into an MIT repository.

The gap is specific and verified: neither existing ruleset resolves Mathlib through Bazel's
downloader, so neither can be repository-cached, remote-downloaded, or made byte-reproducible.
`rules_lean` is unclaimed in the Bazel Central Registry (404, probed against a `rules_rust` 200).

## What Changes

- Extract the patched rules from `thirdparty/rules_lean/` into a standalone MIT-licensed
  repository, designed for BCR publication from the first commit rather than retrofitted.
- Carry over the four fixes made here, with their measurements as the regression baseline:
  - **content-addressed fetch** — a committed lockfile pins every Lake package and the prebuilt
    olean archive by URL + sha256, so the workspace materializes with `download_and_extract` and
    nothing else. Validated against the manifest and toolchain on every fetch, failing closed.
  - **lazy per-platform toolchains** — `toolchain()` declarations in a downloadless repo, one per
    execution platform with `exec_compatible_with`, so resolution stops dragging Mathlib into
    unrelated builds and remote execution gets a binary that runs on the worker.
  - **olean tree-shaking** — `cache_roots` plus a closure-cut archive; measured here at 1,918 of
    9,450 modules (77.8% smaller, 1,874 MB → 452 MB of oleans).
  - **drift detection** — a check that recomputes the import closure and fails when the tree-shaken
    artifacts no longer cover it, or when a machine-local URL is pinned.
- Adopt three things this survey found in `pulseengine/rules_lean`:
  - a hash policy that **fails closed** on a missing sha256 rather than warning and downloading
    unverified — the weakest link in a toolchain whose purpose is trustworthy proof checking;
  - a **shallow** (`--depth 1`) Mathlib fetch for the no-lock fallback path, replacing a
    full-history clone;
  - CI that exercises the **failure modes** — missing hash, wrong platform, version skew — not only
    the happy path.
- Migrate this repository from the vendored copy to the published module.
- **BREAKING** for downstream consumers of the vendored copy: the module name, repository names and
  a small part of the rule surface change. Only this repository consumes it today.

## Capabilities

### New Capabilities

- `lean-content-addressed-fetch`: the lockfile contract — what is pinned, how it is validated,
  what happens on drift, and the fallback when no lock is supplied.
- `lean-toolchain-selection`: per-execution-platform toolchain resolution, the downloadless
  declaration repo, and the fail-closed hash policy.
- `lean-olean-tree-shaking`: cutting the Mathlib artifact set to the proofs' import closure, the
  reproducibility of the resulting archive, and the drift gate that keeps it honest.
- `lean-ruleset-packaging`: the standalone repository — licensing, module identity, public rule
  surface, examples, and the migration of this repository onto it.
- `lean-bcr-release`: registry publication — the `.bcr` templates, a presubmit that stays fast,
  version and compatibility policy, and the release automation.

### Modified Capabilities

None. No existing spec covers the build system; the 154 specs in `openspec/specs/` describe
solver, DSL and formalization behaviour.

## Impact

- **New repository** (outside this tree): the extracted ruleset, MIT licensed, with `.bcr`
  templates and a BCR-facing example module.
- **`MODULE.bazel`**: `bazel_dep` on the published module replaces the `local_path_override` onto
  `thirdparty/rules_lean`; `use_repo`/`register_toolchains` names follow.
- **`thirdparty/rules_lean/`**: retired once the published module builds this repository green.
  Per repository convention it moves aside rather than being deleted.
- **`lean/BUILD.bazel`**, **`lean/lake-lock.json`**, **`lean/lake_packages.bzl`**: consumer-side
  wiring tracks any rule-surface change.
- **`build/scripts/lean_lock.sh`**, **`lean_oleans.py`**, **`lean_closure.py`**,
  **`lean_packages_bzl.py`**: the repin and drift-check tooling is consumer-side and stays here;
  the parts that are generic to the ruleset move with it.
- **`.github/workflows/formalization.yml`**: the `lean-closure` gate continues to run here.
- **Unblocked**: the olean archive still pins a `file://` URL and must be hosted; this change is
  where that lands, alongside the ruleset's own artifact hosting story.
