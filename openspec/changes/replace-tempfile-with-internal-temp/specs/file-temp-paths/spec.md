## ADDED Requirements

### Requirement: TempDir creates a fresh empty directory under the OS temp directory
`TempDir::new()` SHALL create a new, empty directory whose parent is `std::env::temp_dir()`, and SHALL return `std::io::Result<TempDir>`. `TempDir::path()` SHALL return the absolute path of that directory.

#### Scenario: A new directory exists and is empty
- **WHEN** `TempDir::new()` succeeds
- **THEN** `path()` names an existing directory
- **AND** that directory has no entries
- **AND** `path().parent()` equals `std::env::temp_dir()`
- **AND** `path()` is absolute

#### Scenario: Two directories never share a path
- **WHEN** `TempDir::new()` is called 1000 times in one thread and the values are kept alive
- **THEN** all 1000 paths are distinct

### Requirement: TempDir removes its directory tree on drop
When a `TempDir` is dropped, it SHALL remove its directory together with every file and subdirectory inside it. Drop SHALL NOT panic, including when the directory has already been removed.

#### Scenario: A populated tree is removed
- **WHEN** a `TempDir` contains a file, a nested subdirectory, and a file inside that subdirectory, and the `TempDir` is dropped
- **THEN** the directory path no longer exists

#### Scenario: An empty directory is removed
- **WHEN** a `TempDir` with no entries is dropped
- **THEN** the directory path no longer exists

#### Scenario: Drop after external removal does not panic
- **WHEN** the directory is removed through `std::fs::remove_dir_all` and the `TempDir` is then dropped
- **THEN** the drop completes without panicking

### Requirement: NamedTempFile creates a fresh empty file under the OS temp directory
`NamedTempFile::new()` SHALL create a new, empty regular file whose parent is `std::env::temp_dir()`, open for writing, and SHALL return `std::io::Result<NamedTempFile>`. Creation SHALL fail rather than open an existing path. `NamedTempFile::path()` SHALL return the file's absolute path.

#### Scenario: A new file exists and is empty
- **WHEN** `NamedTempFile::new()` succeeds
- **THEN** `path()` names an existing regular file of length 0
- **AND** `path().parent()` equals `std::env::temp_dir()`
- **AND** `path()` is absolute

#### Scenario: Two files never share a path
- **WHEN** `NamedTempFile::new()` is called 1000 times in one thread and the values are kept alive
- **THEN** all 1000 paths are distinct

#### Scenario: Concurrent creation yields distinct paths
- **WHEN** 8 threads each create 100 `NamedTempFile` values and keep them alive
- **THEN** all 800 paths are distinct and every call succeeds

### Requirement: NamedTempFile::with_suffix ends the file name with the suffix
`NamedTempFile::with_suffix(suffix)` SHALL create a file as `NamedTempFile::new()` does, and the file name SHALL end with `suffix` exactly. An empty suffix SHALL behave as `NamedTempFile::new()`. A suffix that contains a path separator, or that is `.` or `..`, SHALL be rejected with an `std::io::Error` of kind `InvalidInput`, and no file SHALL be created.

#### Scenario: A dotted suffix is the file extension
- **WHEN** `NamedTempFile::with_suffix(".csv")` succeeds
- **THEN** the file name ends with `.csv`
- **AND** `path().extension()` is `csv`

#### Scenario: A suffix without a dot is appended verbatim
- **WHEN** `NamedTempFile::with_suffix("_clk")` succeeds
- **THEN** the file name ends with `_clk`

#### Scenario: An empty suffix is accepted
- **WHEN** `NamedTempFile::with_suffix("")` is called
- **THEN** it returns a file in `std::env::temp_dir()`

#### Scenario: A separator in the suffix is rejected
- **WHEN** `NamedTempFile::with_suffix("/../<marker>.csv")` is called, where `<marker>` is a name unique to the test
- **THEN** it returns an error of kind `InvalidInput`
- **AND** no entry ending in `<marker>.csv` exists in `std::env::temp_dir()` or in its parent

#### Scenario: A Windows separator and the dot names are rejected
- **WHEN** `with_suffix` is called with `"a\\b"`, `"."` and `".."` in turn
- **THEN** each call returns an error of kind `InvalidInput` on every platform

### Requirement: NamedTempFile accepts writes through std::io::Write
`NamedTempFile` SHALL implement `std::io::Write`. Bytes written and flushed SHALL be readable by path through `std::fs::read`, in the order written, while the `NamedTempFile` is still alive.

#### Scenario: Written bytes are read back by path
- **WHEN** `b"a,b\n1,2\n"` is written with `write_all` and then flushed
- **THEN** `std::fs::read(path())` returns exactly `b"a,b\n1,2\n"`

#### Scenario: Consecutive writes append
- **WHEN** `b"ab"` is written, then `b"cd"`, then flushed
- **THEN** `std::fs::read(path())` returns exactly `b"abcd"`

#### Scenario: The path can be overwritten by another writer
- **WHEN** `std::fs::write(path(), b"x")` runs while the `NamedTempFile` is alive
- **THEN** `std::fs::read(path())` returns `b"x"`

### Requirement: NamedTempFile removes its file on drop
When a `NamedTempFile` is dropped, it SHALL close its handle and remove its file. Drop SHALL NOT panic, including when the file has already been removed.

#### Scenario: The file is removed
- **WHEN** a `NamedTempFile` that has been written to is dropped
- **THEN** its path no longer exists

#### Scenario: Drop after external removal does not panic
- **WHEN** the file is removed through `std::fs::remove_file` and the `NamedTempFile` is then dropped
- **THEN** the drop completes without panicking

### Requirement: Created entries are private to the owner on Unix
On Unix, a `NamedTempFile` SHALL be created with permission bits `0o600` and a `TempDir` with `0o700`, before the process umask is applied.

#### Scenario: File mode
- **WHEN** a `NamedTempFile` is created on Unix
- **THEN** `metadata(path()).permissions().mode() & 0o777` equals `0o600`

#### Scenario: Directory mode
- **WHEN** a `TempDir` is created on Unix
- **THEN** `metadata(path()).permissions().mode() & 0o777` equals `0o700`

### Requirement: No workspace member declares tempfile
The root `Cargo.toml` `[workspace.dependencies]` table and the `Cargo.toml` of every workspace member SHALL NOT declare `tempfile` under any dependency table, and no source file of a workspace member outside `yanked/` SHALL reference the path `tempfile::`.

#### Scenario: Manifests are free of tempfile
- **WHEN** every `Cargo.toml` in the workspace is searched for a `tempfile` dependency key
- **THEN** there is no match

#### Scenario: Sources are free of tempfile
- **WHEN** every `.rs` file outside `target/` and `yanked/` is searched for `tempfile::`
- **THEN** there is no match

#### Scenario: Migrated suites pass under both build systems
- **WHEN** `cargo test -p deep_causality_file -p deep_causality_cfd -p deep_causality_discovery` and `bazel test` for the same three packages run
- **THEN** both pass
- **AND** the executed test counts of `deep_causality_cfd` and `deep_causality_discovery` equal those recorded before the migration
- **AND** the executed test count of `deep_causality_file` equals its pre-migration count plus the tests this change adds
