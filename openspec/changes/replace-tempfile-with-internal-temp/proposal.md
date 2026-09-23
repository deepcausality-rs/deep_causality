## Why

Three workspace members declare the external crate `tempfile` as a dev-dependency, and they use a
small part of it: create a scratch directory or a scratch file, get its path, write bytes, and have
it removed on drop. `deep_causality_file` is the workspace's file crate. A std-only implementation of
that subset belongs there, and it removes one external declaration from the workspace manifest.

## What Changes

- Add two public types to `deep_causality_file`, both std-only and exported from the crate root:
  - `TempDir`: `TempDir::new()`, `path()`. On drop it removes the directory and everything in it.
  - `NamedTempFile`: `NamedTempFile::new()`, `NamedTempFile::with_suffix(&str)`, `path()`,
    `impl std::io::Write`. On drop it closes the handle and removes the file.
- Migrate every `tempfile` call site to the new types:
  - `tempfile::tempdir()` / `tempdir()` → `TempDir::new()` (37 `dir.path()` uses).
  - `tempfile::Builder::new().suffix(s).tempfile()` → `NamedTempFile::with_suffix(s)`.
  - `tempfile::NamedTempFile::new()` → `NamedTempFile::new()`.
  - `tempfile::TempDir` in helper signatures → `deep_causality_file::TempDir`.
- 33 test files across three crates: 10 in `deep_causality_file`, 3 in `deep_causality_cfd` and 20
  in `deep_causality_discovery`.
- Remove `tempfile` from `[dev-dependencies]` in `deep_causality_file`, `deep_causality_cfd` and
  `deep_causality_discovery`, and from `[workspace.dependencies]` in the root `Cargo.toml`.
- Add `deep_causality_file` as a dev-dependency of `deep_causality_discovery`.
  `deep_causality_cfd` already depends on it at runtime.
- Update `AGENTS.md`: drop the `tempfile` line under external dev-only dependencies, and add
  `deep_causality_file` as a dev-dependency of `deep_causality_discovery`.
- Not replicated: `tempfile::Builder`, `prefix`, `rand_bytes`, `tempfile_in`, `tempdir_in`, `keep`,
  `persist`, `reopen`, `into_temp_path` and `as_file`. No call site uses them.

## Capabilities

### New Capabilities
- `file-temp-paths`: scratch files and directories in `deep_causality_file`. It covers creation in
  the OS temp directory, unique names, suffix handling and validation, restrictive permissions on
  Unix, writes through `std::io::Write`, and removal on drop. It also requires that no workspace
  member declares `tempfile`.

### Modified Capabilities
<!-- None. No existing spec states requirements on temporary files or the tempfile crate. -->

## Impact

- **Public API**: `deep_causality_file` gains `TempDir` and `NamedTempFile`. This addition is not a
  breaking change. The crate stays at Tier 3, and its runtime dependency set does not change.
- **Dependencies**: `tempfile` leaves every library crate's manifest and the workspace dependency
  table. It stays in `Cargo.lock` as a transitive dependency of
  `candle-core → safetensors`, which `examples/causal_discovery_examples` pulls in. The lock entry
  is outside this change's scope.
- **Bazel**: test suites resolve dev-dependencies through `all_crate_deps(normal_dev = True)`,
  which reads `Cargo.toml`. The `BUILD.bazel` files need no edits beyond what that implies. The
  suites of all three crates are re-run under `bazel test` to confirm.
- **Behaviour**: test behaviour does not change. The new types create entries under
  `std::env::temp_dir()`, the directory `tempfile` uses by default.
