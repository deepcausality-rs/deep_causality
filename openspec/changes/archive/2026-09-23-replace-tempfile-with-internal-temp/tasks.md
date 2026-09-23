## 0. Baseline

- [x] 0.1 Resolve placement: a dedicated crate `deep_causality_tempfile`, types exported from its
      root (design D1).
- [x] 0.2 Record the executed test counts of `deep_causality_file`, `deep_causality_cfd` and
      `deep_causality_discovery` under `cargo test -p <crate>` and `bazel test //<pkg>/...`, in
      `notes.md` in this change.

## 1. Phase 1 — Crate scaffold and API with unimplemented bodies

All paths in groups 1–5 are relative to `deep_causality_utils/deep_causality_tempfile`.

- [x] 1.0 Scaffold the crate: `Cargo.toml` (no dependencies, `[lints] workspace = true`),
      `BUILD.bazel` (library, doc, doc test), `README.md` and `LICENSE`. Add
      `deep_causality_tempfile` to the root `[workspace.dependencies]`.
- [x] 1.1 Add `src/utils/mod.rs` and `src/utils/temp_name.rs`, with a `pub(crate)` name generator
      whose body is `unimplemented!()`. Register `mod utils;` privately in `lib.rs`.
- [x] 1.2 Add `src/types/temp_dir/mod.rs` with `TempDir` (private field `path: PathBuf`),
      `TempDir::new() -> io::Result<TempDir>` and `path(&self) -> &Path`, both `unimplemented!()`.
      Add `temp_dir_drop.rs` with an empty `Drop` body (design D7).
- [x] 1.3 Add `src/types/named_temp_file/mod.rs` with `NamedTempFile` (private fields `path`,
      `file`), `new()`, `with_suffix(&str)` and `path()`, all `unimplemented!()`. Add
      `named_temp_file_write.rs`, where `write` and `flush` are `unimplemented!()`, and
      `named_temp_file_drop.rs` with an empty `Drop` body.
- [x] 1.4 Register both modules in `src/types/mod.rs`, keep `types` private in `lib.rs`, re-export
      `TempDir` and `NamedTempFile` from `lib.rs`, and add rustdoc stating the D2 naming scheme, the D4 suffix rule and the D6 drop
      behaviour.
- [x] 1.5 Verify: `cargo build -p deep_causality_tempfile` and
      `bazel build //deep_causality_utils/deep_causality_tempfile` succeed.

## 2. Phase 2 — Full suite, observed failing

- [x] 2.1 Write the corner-case enumeration into `notes.md` and name the test that covers each
      case. The cases are: an empty suffix; a suffix without a dot; `/` and `\` in the suffix; `.` and `..` as
      the suffix; an empty `TempDir`; a nested populated `TempDir`; an entry removed externally before
      drop; 1000 sequential creations; 8×100 concurrent creations; consecutive writes; an overwrite
      through another writer; Unix modes.
- [x] 2.2 Add `tests/types/temp_dir/temp_dir_tests.rs` and `temp_dir_drop_tests.rs`, one test per
      `TempDir` scenario in `specs/tempfile-crate/spec.md`.
- [x] 2.3 Add `tests/types/named_temp_file/named_temp_file_tests.rs`,
      `named_temp_file_write_tests.rs` and `named_temp_file_drop_tests.rs`, covering every
      `NamedTempFile` scenario, the `InvalidInput` variant (asserting the kind, not only
      `is_err()`) and the Unix mode scenarios under `#[cfg(unix)]`. Expected byte strings are
      literals.
- [x] 2.4 Register the test modules through `tests/mod.rs` → `tests/types/mod.rs` → the two
      directory `mod.rs` files. In `BUILD.bazel`, add `rust_test_suite` targets for
      `tests/types/temp_dir/*_tests.rs` and `tests/types/named_temp_file/*_tests.rs`.
- [x] 2.5 Run the suite under cargo and Bazel. Confirm that every new test fails with the
      `unimplemented` panic, or on the drop assertion (D7). Record the output and the test count in
      `notes.md`.

## 3. Phase 3 — Defect audit

- [x] 3.1 Write a throwaway correct implementation, then introduce each defect one at a time and
      confirm that a test whose subject is that behaviour fails:
      the counter not incremented; the pid or nanos term dropped; the suffix prepended rather than
      appended; the `/` check removed; the `\` check removed; `create(true)` in place of
      `create_new(true)`; the parent set to the current directory rather than `temp_dir()`; `remove_dir`
      in place of `remove_dir_all`; the drop removal skipped; `unwrap()` on the drop removal; mode
      `0o644` / `0o755`; `write` returning `Ok(0)`; `flush` as a no-op on a `BufWriter`.
- [x] 3.2 Add a test for each defect that survives, and repeat the audit. Record the results in
      `notes.md`, then discard the throwaway implementation.

## 4. Phase 4 — Implementation

- [x] 4.1 Implement `temp_name` as in D2, and `TempDir` and `NamedTempFile` as in D2, D4, D5 and D6.
- [x] 4.2 Verify: `cargo test -p deep_causality_tempfile` and
      `bazel test //deep_causality_utils/deep_causality_tempfile/...` pass, and the counts agree.
- [x] 4.3 Verify full line coverage with `cargo llvm-cov -p deep_causality_tempfile`.
- [x] 4.4 Verify that `cargo tree -p deep_causality_tempfile -e normal` lists no dependency.

## 5. Phase 5 — Mutation testing

- [x] 5.1 Run `scripts/mutants.sh deep_causality_tempfile` over `src/utils/temp_name.rs`,
      `src/types/temp_dir/` and `src/types/named_temp_file/`, and record the report in `notes.md`.
- [x] 5.2 Kill each survivor with a test, or add an escaped `.cargo/mutants.toml` entry that carries
      the measurement, verified with the file's `comm` check.

## 6. Migration

In each crate below, replace the `tempfile` dev-dependency with
`deep_causality_tempfile = { workspace = true }`. Rewrite call sites to
`use deep_causality_tempfile::{NamedTempFile, TempDir};` and
`Builder::new().suffix(s).tempfile()` → `NamedTempFile::with_suffix(s)`. Then run cargo and Bazel
tests and compare the counts with 0.2.

- [x] 6.1 `deep_causality_file`: 10 test files.
- [x] 6.2 `deep_causality_cfd`: 3 test files.
- [x] 6.3 `deep_causality_discovery`: 20 test files.
- [x] 6.4 Remove `tempfile` from the root `[workspace.dependencies]`. Run `cargo build --workspace
      --all-targets` and `bazel build //...`.

## 7. Close-out

- [x] 7.1 Verify the "No workspace member declares tempfile" scenarios. Run
      `grep -rn "tempfile" --include=Cargo.toml .` and `grep -rn "tempfile::" --include=*.rs .`,
      excluding `target/` and `yanked/`. Both must return no match.
- [x] 7.2 Update `AGENTS.md`:
      - 32 → 33 library crates, and four utility crates under `deep_causality_utils/`
      - the Core and Data Structures listing
      - Tier 0
      - `deep_causality_tempfile` as an internal dev-only dependency of the three crates
      - remove the `tempfile` bullet
      - 25 → 26 crates without external runtime dependencies; 47 → 48 workspace members

      `deep_causality_unified_math/README.md` needs no edit: its tier block covers only the
      mathematics crates, and `deep_causality_tempfile` is neither one of them nor a dependency of
      one.
- [x] 7.3 Run `make format && make fix` (three crates changed) and fix the lints rather than
      suppressing them.
- [x] 7.4 Prepare a commit message for the user. Do not commit.
