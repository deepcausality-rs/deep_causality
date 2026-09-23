## Context

`tempfile` is a dev-dependency of `deep_causality_file`, `deep_causality_cfd` and
`deep_causality_discovery`. The 33 test files that use it touch this surface and nothing more:

| `tempfile` item | Use |
|---|---|
| `tempdir()` → `TempDir` | scratch directory; `path()`; removed on drop |
| `TempDir` (type name) | returned from helpers such as `write_temp` so the directory outlives the helper |
| `NamedTempFile::new()` | scratch file; `path()` |
| `Builder::new().suffix(s).tempfile()` | scratch file with an extension the loader dispatches on (`.csv`, `.clk`, `.sp3`) |
| `impl Write for NamedTempFile` | `write_all`, `flush` |

Searches for `persist`, `keep`, `reopen`, `as_file`, `into_temp_path`, `prefix`, `tempfile_in`,
`tempdir_in` and `Seek` return nothing.

Both build systems read dev-dependencies from `Cargo.toml`. Bazel test suites link them through
`all_crate_deps(normal = True, normal_dev = True)`, and that call also covers internal path
crates. A consumer's new dev-dependency therefore reaches Bazel with no hand edit.

`deep_causality_file`, the workspace's file crate, holds data loaders and writers. Its runtime
dependencies are `chrono`, `haft`, `num` and `algebra`. Scratch files are a separate, test-only
concern, so they get their own crate.

## Goals / Non-Goals

**Goals:**
- Provide the table's surface with std only, in a new zero-dependency crate
  `deep_causality_tempfile`.
- Leave every existing crate's runtime dependencies and public API unchanged.
- Migrate call sites mechanically. No test assertion changes.
- Follow `unified-math-tdd-protocol`: API, failing suite, defect audit, implementation, mutants.

**Non-Goals:**
- Parity with the rest of `tempfile`: `Builder`, `persist`, `keep`, `reopen`, custom directories and
  prefixes.
- Removing `tempfile` from `Cargo.lock`. It stays there as a transitive dependency of
  `candle-core → safetensors`.
- Recovering scratch entries left behind when a process aborts or is killed. `tempfile` does not
  recover them either.

## Decisions

### D1 — A dedicated crate, types exported from its root
The crate is `deep_causality_tempfile` at `deep_causality_utils/deep_causality_tempfile`, with an
empty `[dependencies]` table. Its types are `TempDir` and `NamedTempFile`, under
`src/types/temp_dir/` and `src/types/named_temp_file/`, both re-exported from `lib.rs`. The
`types` and `utils` modules stay private. Each module follows the
one-type-one-module rule: `mod.rs` holds the type and its constructors, and each trait impl has its
own file (`temp_dir_drop.rs`, `named_temp_file_drop.rs`, `named_temp_file_write.rs`). The
name-generation helper they share is `pub(crate)` in `src/utils/temp_name.rs`. It is tested only
through the public types, so the change adds no in-src `#[cfg(test)]` module that Bazel could miss.

*Alternatives (rejected):* putting the types in `deep_causality_file` would make every test-only
consumer, such as `deep_causality_discovery`, link that crate's data loaders and their runtime
dependencies to get two small types. A `utils_tests` module would have the same cost.

### D2 — Names from pid, clock and counter; exclusive creation; no retry
The name is `.dct-{pid}-{nanos}-{counter}{suffix}`. `pid` is `std::process::id()`. `nanos` is
`SystemTime::now()` since the Unix epoch. `counter` is a process-global `AtomicU64` incremented with
`fetch_add(1, Relaxed)`. Files are created with `OpenOptions::new().write(true).create_new(true)`,
which is `O_CREAT | O_EXCL` on Unix, so an existing path or a planted symlink is never opened.
Directories use `DirBuilder::create`, which fails if the path exists. `AlreadyExists` is returned to
the caller rather than retried.

*Alternatives:* random names from `deep_causality_rand` would add a runtime dependency and move
the crate up a tier. A retry loop would add an exhaustion branch that the public API cannot reach,
so under the protocol it would need a recorded unreachability argument. The pid and counter already
make names unique inside a process and across live processes. The nanos term separates a new
process from an old one that reused its pid.

### D3 — `std::io::Result`, not `DataLoadingError`
The constructors return `std::io::Result`. They are OS operations, and callers need
`ErrorKind::InvalidInput` and `AlreadyExists` intact. Converting to `DataLoadingError::OsError(String)`
would drop the kind. Call sites keep `.unwrap()` and `.expect("tempdir")` unchanged.

### D4 — `with_suffix` replaces `Builder`
`NamedTempFile::with_suffix(&str)` replaces the one `Builder` chain in use, and `tempfile` offers a
function with the same name. The suffix is rejected with `InvalidInput` when it contains `/` or `\`
on any platform. Without that check, `"/../x"` would create the file
outside the temp directory.

### D5 — Unix permissions 0o600 / 0o700
`OpenOptionsExt::mode(0o600)` and `DirBuilderExt::mode(0o700)` are applied under `#[cfg(unix)]`.
The file mode matches `tempfile` 3.27. The directory mode is stricter: `tempfile` creates
directories as `0o777`, reduced only by the umask. No call site needs group or other access. With
names this predictable, these modes keep other local users from reading scratch entries in a shared
`/tmp`.

### D6 — Drop ignores removal errors
`TempDir`'s drop calls `fs::remove_dir_all`, and `NamedTempFile`'s drop calls `fs::remove_file`.
Both results are discarded, because drop has no error channel and must not panic. `NamedTempFile`
keeps its `File` as a plain field. On Unix, unlinking an open file is allowed. On Windows, std opens
files with `FILE_SHARE_DELETE`, so the unlink is accepted and completes when the handle closes.

### D7 — The phase-1 drop is a no-op, not `unimplemented!()`
The protocol requires every phase-1 body to panic as unimplemented. A panicking `Drop` that runs
while a failing test unwinds aborts the whole test binary, so no per-test failure would be
reported. In phase 1 the two `Drop` impls are therefore empty. Their tests then fail on the intended
assertion ("path still exists"). This deviation is recorded in the phase-2 notes.

## Risks / Trade-offs

- [The Bazel sandbox points `TMPDIR` at a per-test directory, or leaves it unset] → Both types read
  `std::env::temp_dir()`, which is what `tempfile` read, so behaviour under Bazel does not change.
  The migration step compares Bazel test counts before and after.
- [A stale entry from a killed process collides on pid, nanos and counter] → `create_new` fails
  with `AlreadyExists`. The failure is visible and nothing is overwritten. The probability is
  negligible because nanos differ between process lifetimes.
- [Coarse clock resolution, such as microseconds on macOS, gives equal nanos inside a tight loop] →
  The counter keeps names distinct. The 1000-iteration uniqueness scenario pins this, and phase-5
  mutants on the counter confirm it.
- [A new published crate] → The three consumers inherit the dev-dependency with a version, and
  `cargo publish` resolves that version on crates.io, so `deep_causality_tempfile` 0.1.0 must be
  released first. release-plz orders workspace releases by dependency. The first release of the
  consumers after this change carries the new crate with it.

## Migration Plan

1. Record the pre-migration test counts of the three crates under `cargo test` and `bazel test`.
2. Scaffold `deep_causality_tempfile` (`Cargo.toml`, `BUILD.bazel`, `README.md`, `LICENSE`) and
   add it to the root `[workspace.dependencies]`. The `deep_causality_utils/*` member glob already
   covers it. Then run phases 1–5 in that crate.
3. Rewrite call sites crate by crate, `file`, then `cfd`, then `discovery`. In each crate, replace
   the `tempfile` dev-dependency with `deep_causality_tempfile`, then build and test it and compare
   counts.
4. Remove `tempfile` from `[workspace.dependencies]` and update `AGENTS.md`.
5. Run the source and manifest checks from the spec.

Rollback reverts the commit. No public item is removed.

## Open Questions

None.
