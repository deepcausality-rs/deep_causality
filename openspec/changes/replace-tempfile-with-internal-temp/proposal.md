## Why

Three workspace members declare the external crate `tempfile` as a dev-dependency, and they use a
small part of it: create a scratch directory or a scratch file, get its path, write bytes, and have
it removed on drop. That subset fits in a std-only crate with no dependencies. The crate can then
serve as a test dependency for any member without pulling in anything else.

## What Changes

- Add the crate `deep_causality_tempfile` at `deep_causality_utils/deep_causality_tempfile`. It has
  no dependencies and sits at Tier 0. It exports two types from the crate root:
  - `TempDir`: `TempDir::new()`, `path()`. On drop it removes the directory and everything in it.
  - `NamedTempFile`: `NamedTempFile::new()`, `NamedTempFile::with_suffix(&str)`, `path()`,
    `impl std::io::Write`. On drop it closes the handle and removes the file.
- Migrate every `tempfile` call site to the new crate:
  - `tempfile::tempdir()` / `tempdir()` → `TempDir::new()`.
  - `tempfile::Builder::new().suffix(s).tempfile()` → `NamedTempFile::with_suffix(s)`.
  - `tempfile::NamedTempFile::new()` → `NamedTempFile::new()`.
  - `tempfile::TempDir` in helper signatures → `deep_causality_tempfile::TempDir`.
- 33 test files across three crates: 10 in `deep_causality_file`, 3 in `deep_causality_cfd` and 20
  in `deep_causality_discovery`.
- In those three crates, replace the `tempfile` dev-dependency with `deep_causality_tempfile`.
  Replace the `tempfile` entry in the root `[workspace.dependencies]` with
  `deep_causality_tempfile`.
- Update `AGENTS.md` and the tier block in `deep_causality_unified_math/README.md`:
  - 33 library crates; four utility crates under `deep_causality_utils/`.
  - `deep_causality_tempfile` in Tier 0.
  - `deep_causality_tempfile` listed as an internal dev-only dependency of the three crates.
  - The `tempfile` bullet removed.
- Not replicated: `tempfile::Builder`, `prefix`, `rand_bytes`, `tempfile_in`, `tempdir_in`, `keep`,
  `persist`, `reopen`, `into_temp_path` and `as_file`. No call site uses them.

## Capabilities

### New Capabilities
- `tempfile-crate`: the `deep_causality_tempfile` crate. It covers:
  - creation in the OS temp directory
  - unique names
  - suffix handling and validation
  - restrictive permissions on Unix
  - writes through `std::io::Write`
  - removal on drop
  - zero dependencies
  - the requirement that no workspace member declares `tempfile`

### Modified Capabilities
<!-- None. No existing spec states requirements on temporary files or the tempfile crate. -->

## Impact

- **New published crate:** `deep_causality_tempfile` 0.1.0. The three consumers are published
  crates, and they inherit the dev-dependency with a version. `cargo publish` resolves a versioned
  dev-dependency against crates.io, so `deep_causality_tempfile` must be released before or with
  them. release-plz orders workspace releases by dependency.
- **Existing crates:** the public API of `deep_causality_file`, `deep_causality_cfd` and
  `deep_causality_discovery` does not change, and neither does their runtime dependency set.
- **Dependencies:** `tempfile` leaves every library crate's manifest and the workspace dependency
  table. It stays in `Cargo.lock` as a transitive dependency of
  `candle-core → safetensors`, which `examples/causal_discovery_examples` pulls in. The lock entry
  is outside this change's scope.
- **Bazel:** the new crate needs a `BUILD.bazel`. Consumers link dev-dependencies through
  `all_crate_deps(normal = True, normal_dev = True)`, which reads `Cargo.toml`, so their
  `BUILD.bazel` files need no edits.
- **Behaviour:** test behaviour does not change. The new types create entries under
  `std::env::temp_dir()`, the directory `tempfile` uses by default.
