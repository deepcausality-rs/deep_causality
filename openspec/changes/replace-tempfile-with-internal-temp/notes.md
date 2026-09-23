# Notes — replace-tempfile-with-internal-temp

Machine: M3 Max, 16 cores, 128 GB. Branch `feature/replace-tempfile`, based on `9cf711e40`.

## 0.2 Baseline test counts (before any change)

Passed test cases, summed over `test result:` lines. Bazel counts include only targets that
`bazel query "tests(//<pkg>/...)"` currently defines. Stale logs from deleted targets would
otherwise inflate the total (`cfd` read 1880 before this filter).

| Crate | `cargo test -p` | `bazel test //<pkg>/...` | ignored (cargo / bazel) |
|---|---|---|---|
| `deep_causality_file` | 154 | 154 | 0 / 0 |
| `deep_causality_cfd` | 943 | 943 | 2 / — |
| `deep_causality_discovery` | 365 | 205 | 0 / 0 |

The two build systems disagree for `deep_causality_discovery`, and the difference predates this
change. `tests/config_tests.rs` is a second Cargo test binary whose whole body is `mod types;`, so
Cargo runs every `tests/types` test twice. Bazel's 205 matches the authored count: 194 `#[test]`
in `tests/`, 8 in `src/`, and 3 doc tests. This change compares each build system against its own
baseline.

## Phase 1 deviation — crate placement

Phase 1 first placed the API inside `deep_causality_file`. It moved to the dedicated crate
`deep_causality_utils/deep_causality_tempfile` before any test was written (commit `a6e3e314e`),
and `deep_causality_file` was restored unchanged.

## Phase 2 deviations — recorded

1. **An internal seam was added after phase 1.** The requirement "creation SHALL fail rather than
   open an existing path" cannot be reached through the public API: names contain the clock, so a
   test cannot plant an entry at the next name. Without a seam, the phase-3 defect
   `create(true)` in place of `create_new(true)` would survive. Both types therefore gained
   `pub(crate) fn create_at(path: PathBuf)` with an `unimplemented!()` body, and the public
   constructors will call it. The public surface is unchanged.
2. **The dot-name suffix rule was removed from the spec.** Rejecting `.` or `..` as a suffix
   protects nothing: the suffix is appended to a non-empty generated stem, so it can never form a
   `.` or `..` path component. Only a separator can move the file out of the temp directory. The
   spec, design D4, the rustdoc and the README now reject `/` and `\` only, and a test pins that
   `.` and `..` are accepted and stay inside `temp_dir()`.
3. **Drop tests fail inside the constructor, not on their assertion.** Design D7 expected them to
   fail on "path still exists". They call `new()` first, which panics `unimplemented`. The empty
   `Drop` bodies still matter, because a panicking `Drop` during unwind would abort the binary.
4. **In-src probe cleanup uses a guard.** The first phase-2 runs panicked between planting a probe
   entry and removing it, which left `dct-unit-<pid>-*` entries in `$TMPDIR`. A `Probe` guard now
   removes the entry on drop, including during a panic. Leftovers from pids 92523, 93548 and 93643
   remain in `$TMPDIR` pending the user's permission to remove them.

## 2.1 Corner-case enumeration

| Case | Test |
|---|---|
| empty suffix | `named_temp_file_tests::empty_suffix_creates_a_file_under_temp_dir` |
| suffix without a dot | `named_temp_file_tests::undotted_suffix_is_appended_verbatim` |
| dotted suffix → extension | `named_temp_file_tests::dotted_suffix_is_the_extension` |
| `/` in suffix (escape attempt) | `named_temp_file_tests::slash_in_suffix_is_invalid_input_and_creates_nothing` |
| `\` in suffix, every platform | `named_temp_file_tests::backslash_in_suffix_is_invalid_input_on_every_platform` |
| `.` and `..` as the whole suffix | `named_temp_file_tests::dot_suffixes_stay_inside_temp_dir` |
| suffix placed after the counter | `temp_name::tests::suffix_is_appended_after_the_counter` |
| empty `TempDir` dropped | `temp_dir_drop_tests::drop_removes_an_empty_directory` |
| nested populated `TempDir` dropped | `temp_dir_drop_tests::drop_removes_a_populated_tree` |
| dir removed externally before drop | `temp_dir_drop_tests::drop_after_external_removal_does_not_panic` |
| file removed externally before drop | `named_temp_file_drop_tests::drop_after_external_removal_does_not_panic` |
| written file removed on drop | `named_temp_file_drop_tests::drop_removes_a_written_file` |
| 1000 sequential dirs / files | `temp_dir_tests::thousand_directories_have_distinct_paths`, `named_temp_file_tests::thousand_files_have_distinct_paths` |
| 1000 back-to-back names (clock ties) | `temp_name::tests::thousand_back_to_back_paths_are_distinct` |
| 8×100 concurrent creations | `named_temp_file_tests::concurrent_creation_yields_distinct_paths` |
| name fields: pid, clock, counter | `temp_name::tests::name_carries_pid_clock_and_counter`, `counter_strictly_increases_within_a_thread` |
| consecutive writes | `named_temp_file_write_tests::consecutive_writes_append` |
| write + flush read back | `named_temp_file_write_tests::written_bytes_are_read_back_by_path` |
| overwrite through another writer | `named_temp_file_write_tests::another_writer_can_overwrite_the_path` |
| existing file at the path | `named_temp_file::tests::create_at_existing_file_is_already_exists_and_keeps_content` |
| planted dangling symlink (Unix) | `named_temp_file::tests::create_at_planted_symlink_is_already_exists_and_not_followed` |
| existing dir / file at the dir path | `temp_dir::tests::create_at_existing_directory_is_already_exists`, `create_at_existing_file_is_already_exists` |
| Unix modes | `named_temp_file_tests::file_mode_is_owner_read_write_only`, `temp_dir_tests::directory_mode_is_owner_only` |
| fresh, empty, absolute, parent = `temp_dir()` | `temp_dir_tests::new_creates_an_empty_absolute_directory_under_temp_dir`, `named_temp_file_tests::new_creates_an_empty_absolute_file_under_temp_dir` |

Error kinds constructed: `InvalidInput` (the two separator tests) and `AlreadyExists` (the four
`create_at` tests). Each test asserts the kind.

Independent oracles: the name-field test compares against `std::process::id()` and against clock
readings the test takes before and after the call. Byte expectations are literals. Distinctness is
checked with a `HashSet` count. No expectation is computed through the code under test.

## 2.5 Failure run against the unimplemented API

| Target | cargo | bazel |
|---|---|---|
| in-src (`lib_unit_tests`) | 0 passed, 10 failed | 0 passed, 10 failed |
| `tests/` (5 files) | 0 passed, 21 failed | 0 passed, 21 failed |
| doc tests | 0 passed, 2 failed | 0 passed, 2 failed |
| **total** | **33 failed** | **33 failed** |

Authored count: 10 `#[test]` in `src/`, 21 in `tests/` and 2 doc examples, 33 in total. Every panic
raised in `src/` is `not implemented` (40 panics under cargo, counting the 8 worker threads of the
concurrent test). The one panic raised in a test file is `h.join().unwrap()` in
`concurrent_creation_yields_distinct_paths`, which passes on a worker thread's `unimplemented`
panic.

## 3. Defect audit

A throwaway implementation passed all 33 tests. Each defect below was applied alone as a
single-site patch, the suite was run, and the patch was reverted. The patched string had to occur
exactly once in its file.

**Miscalibration found and fixed.** The first run reported three survivors: nanos dropped, file
mode `0o644` and dir mode `0o755`. Each patch had replaced the first occurrence of its string, and
in each case that first occurrence was in a rustdoc comment, so the code never changed. After the
patches were retargeted to code-only strings, all three are killed. The audit script now requires
every patch target to be unique in its file.

| Defect | Killed by (subject-matched test) |
|---|---|
| counter not incremented | `counter_strictly_increases_within_a_thread`, `thousand_files_have_distinct_paths`, `concurrent_creation_yields_distinct_paths` |
| pid term dropped | `name_carries_pid_clock_and_counter` |
| nanos term dropped | `name_carries_pid_clock_and_counter` |
| suffix prepended | `suffix_is_appended_after_the_counter`, the three suffix tests |
| `with_suffix` ignores the suffix | `dotted_suffix_is_the_extension`, `undotted_suffix_is_appended_verbatim`, `dot_suffixes_stay_inside_temp_dir` |
| `/` check removed | `slash_in_suffix_is_invalid_input_and_creates_nothing` |
| `\` check removed | `backslash_in_suffix_is_invalid_input_on_every_platform` |
| `create(true)` for `create_new(true)` | `create_at_existing_file_is_already_exists_and_keeps_content`, `create_at_planted_symlink_is_already_exists_and_not_followed` |
| parent = current dir | the four parent-equals-`temp_dir()` tests, the three `temp_name` tests |
| dir created with `recursive(true)` (reuses an existing dir) | `create_at_existing_directory_is_already_exists` |
| `remove_dir` for `remove_dir_all` | `drop_removes_a_populated_tree` |
| dir drop removal skipped | `drop_removes_a_populated_tree`, `drop_removes_an_empty_directory` |
| file drop removal skipped | `drop_removes_a_written_file` |
| `unwrap()` on dir drop removal | `temp_dir_drop_tests::drop_after_external_removal_does_not_panic` |
| `unwrap()` on file drop removal | `named_temp_file_drop_tests::drop_after_external_removal_does_not_panic` |
| file mode `0o644` | `file_mode_is_owner_read_write_only` |
| dir mode `0o755` | `directory_mode_is_owner_only` |
| `write` returns `Ok(0)` | `written_bytes_are_read_back_by_path`, `consecutive_writes_append` |
| `write` drops half the buffer | `written_bytes_are_read_back_by_path`, `consecutive_writes_append` |
| `flush` a no-op over a `BufWriter` | `written_bytes_are_read_back_by_path`, `consecutive_writes_append` |
| `path()` returns the parent | 14 `NamedTempFile` tests |

21 of 21 are killed, and no suite widening was needed.

**Input-variety review.** One coincidence remains, and it comes from the environment rather than
the suite. The mode tests read `mode & 0o777` after the umask has applied. Under umask `0022`,
which this machine uses, `0o644` and `0o600` differ. Under umask `0077` they coincide, and a
mode defect would pass. std has no way to set the umask without `unsafe`, which the workspace
forbids, so the mode tests discriminate only under a umask that leaves group or other bits set.
The other cases in the enumeration keep distinct quantities apart. The dotted-suffix test asserts
both `ends_with` and `extension()`. The distinctness tests hold values alive, so a path freed by
drop cannot be reused. The clock test brackets the value between two independent clock readings.

The throwaway implementation was discarded (`git checkout -- src`). A copy outside the repository
served as the audit's restore point.
