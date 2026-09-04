## ADDED Requirements

### Requirement: The public API is declared before any implementation exists
`deep_causality_linear` SHALL first be built as a compiling crate whose every public type, trait and function is declared with an unimplemented body, and SHALL NOT contain a working algorithm at that stage.

Writing the surface first forces the API to be designed against its callers rather than discovered
by whatever the implementation found convenient. It also makes the test suite compilable before
anything works, which is what the next requirement depends on.

#### Scenario: The mock crate compiles
- **WHEN** the API-only crate is built
- **THEN** compilation succeeds with no error
- **AND** every declared item is reachable from the crate root

#### Scenario: Nothing works yet
- **WHEN** any public function of the API-only crate is called
- **THEN** it panics as unimplemented rather than returning a value

#### Scenario: The surface is complete before tests are written
- **WHEN** the test suite is authored
- **THEN** it needs no addition to the public surface to compile

### Requirement: The full test suite is written and failing before implementation
The complete test suite SHALL be written against the unimplemented API and SHALL be observed failing before any algorithm is written.

A test written after the code it checks tends to encode what the code does. A test written before it
encodes what the code should do. Observing the failure is what distinguishes a test that will catch
a defect from one that passes vacuously.

#### Scenario: Every test fails for the intended reason
- **WHEN** the suite is run against the unimplemented API
- **THEN** every test fails
- **AND** each failure is the unimplemented panic, not a compile error, a missing import, or a panic from elsewhere

#### Scenario: No test is added after its implementation
- **WHEN** the commit history for a behaviour is read
- **THEN** its test appears in a commit at or before the one implementing it

### Requirement: Every specification scenario has a named test
Each scenario in this change's other capabilities SHALL map to at least one test, and the mapping SHALL be recorded so that an unmapped scenario is visible.

The scenarios are the agreed statement of what the crate must do. A scenario with no test is a
requirement nobody checks, and it is invisible unless the mapping is written down.

#### Scenario: The mapping is complete
- **WHEN** the scenarios of `linear-matrix-representations`, `linear-dense-algorithms`, `linear-f2-algebra` and `linear-crate-identity` are enumerated
- **THEN** each names at least one test function
- **AND** the mapping is committed alongside the suite

#### Scenario: An unmapped scenario is caught
- **WHEN** a scenario is added or renamed without a corresponding test
- **THEN** the mapping check fails

### Requirement: The corner cases are enumerated before the suite is judged complete
The suite SHALL cover the degenerate and boundary inputs of every operation, enumerated in advance rather than discovered during implementation.

Elimination and the packed representation both fail at their edges, not in the middle. The cases
below are the ones this change's research already identified as load-bearing; finding them after the
fact would mean finding them through a defect.

At minimum: empty matrices (0×0, 0×n, n×0); 1×1; non-square in both orientations; a zero row; a zero
column; a singular matrix; a matrix whose `(0,0)` entry is zero but which is non-singular
(Cayley-Menger); a rank-deficient matrix; a column count that is not a multiple of the packed word
width; the same matrix packed at two different word widths; a matrix whose entries are `{-1, 0, 1}`
reduced mod 2; an entry outside `{0, 1}` offered to the packed constructor; a sparse matrix with an
entirely empty row; an index outside the shape; and, for the float paths, a pivot candidate that is
near zero with a larger one below it.

#### Scenario: The enumerated cases are all covered
- **WHEN** the corner-case list is checked against the suite
- **THEN** each case names a test

#### Scenario: A degenerate input never panics unexpectedly
- **WHEN** any enumerated degenerate input is passed to any public function
- **THEN** the call returns a value or a typed error, and does not panic or overflow

### Requirement: The suite is verified against a defective implementation before it is trusted
The suite SHALL be shown to fail when the implementation is deliberately wrong, on each defect class this change's research identified.

A suite that passes on correct code proves nothing on its own; it has to be shown to reject
incorrect code. Each defect below is one this change already knows is reachable, so each is a
concrete thing the suite must catch rather than a hypothetical.

#### Scenario: Removing pivoting is caught
- **WHEN** the elimination is changed to take the diagonal entry as its pivot without searching
- **THEN** at least one test fails, and it is a Cayley-Menger test

#### Scenario: An off-by-one in the packed word index is caught
- **WHEN** the packed row update is changed to skip or repeat one word
- **THEN** at least one test fails

#### Scenario: A loosened tolerance is caught
- **WHEN** an exactness check in the 𝔽₂ path is replaced by a tolerance comparison
- **THEN** at least one test fails

#### Scenario: A silently wrong rank is caught
- **WHEN** the reported rank is changed by one in either direction
- **THEN** at least one test fails

### Requirement: Implementation is complete only when the suite passes at full coverage
Implementation SHALL be judged finished by the suite turning green with the coverage the repository requires, and the suite SHALL NOT be edited to accommodate an implementation.

The repository's standing rule is one hundred percent coverage of added or edited files, and that a
failing test means the implementation or the API is wrong. Neither is relaxed here.

#### Scenario: Coverage meets the standing requirement
- **WHEN** `cargo llvm-cov` runs over the crate
- **THEN** every added file reports full line coverage, except lines demonstrably unreachable

#### Scenario: A failing test changes the code, not the test
- **WHEN** a test fails during implementation
- **THEN** the implementation or the API changes
- **AND** the test's assertion is weakened only if the API itself was agreed to be wrong

#### Scenario: Both build systems run the suite
- **WHEN** `cargo test` and `bazel test //...` are run
- **THEN** both execute the full suite and both pass

### Requirement: Downstream migration begins only after the suite is green
No consumer of `deep_causality_sparse` or `deep_causality_tensor` SHALL be repointed at `deep_causality_linear` until the crate's own suite passes at the required coverage.

Migrating 102 import sites against an implementation that is still moving mixes two failure sources:
a broken consumer and a broken library. Sequencing them means a failure after migration is
unambiguously a migration failure.

#### Scenario: Migration is gated
- **WHEN** the first consumer import is repointed
- **THEN** the crate's own suite is already green at full coverage

#### Scenario: A post-migration failure is attributable
- **WHEN** a consumer fails to build or its tests fail after migration
- **THEN** the crate's own suite is known to have been passing beforehand

### Requirement: Shared test helpers live in `src`, never in `tests`
Every shared test helper SHALL live under `src/utils_tests/`, and no helper module SHALL be placed inside the `tests/` tree.

Bazel cannot reach a helper file that lives inside `tests/` — only the `src` tree is available to the
test target — so a helper placed there builds under Cargo and fails under Bazel. Placing helpers in
`src` has a second consequence that is not optional: they are library code, so the repository's
coverage requirement applies to them, and they need their own tests.

The workspace spells this directory two ways today — `utils_tests` in `deep_causality_algebra`,
`deep_causality_num_complex`, `deep_causality_physics` and `deep_causality_topology`, and
`utils_test` in `deep_causality` and `deep_causality_ethos`. This crate follows `AGENTS.md` and the
neighbouring math crates: **`utils_tests`**.

#### Scenario: No helper sits under tests
- **WHEN** the `tests/` tree is searched for helper modules
- **THEN** it contains only test files, each named `*_tests.rs`

#### Scenario: Bazel reaches every helper
- **WHEN** `bazel test //deep_causality_linear/...` runs from a clean output base
- **THEN** it resolves every helper without a missing-file error

#### Scenario: Helpers are themselves tested
- **WHEN** a helper `src/utils_tests/h.rs` exists
- **THEN** its tests are at `tests/utils_tests/h_tests.rs`
- **AND** coverage over `src/utils_tests/` meets the crate's coverage requirement

### Requirement: Test layout mirrors the source tree and is registered in both build systems
The suite SHALL mirror the `src` tree file for file, SHALL register every test module upward through its `float_bfloat16` chain, and SHALL declare every test directory in `tests/BUILD.bazel`.

A test file that is not registered in its `float_bfloat16` compiles and never runs, and a directory absent
from `tests/BUILD.bazel` is invisible to the primary gate. Both failures are silent: the suite
appears to pass.

#### Scenario: Structure mirrors the source
- **WHEN** a source file `src/a/b.rs` exists
- **THEN** its tests are at `tests/a/b_tests.rs`

#### Scenario: Every test module is registered
- **WHEN** the suite is run
- **THEN** the executed test count matches the number of test functions in the tree

#### Scenario: Bazel sees every test directory
- **WHEN** `bazel test //deep_causality_linear/...` runs
- **THEN** every test directory is declared in `tests/BUILD.bazel` and executes
- **AND** the count of tests it executes matches the count `cargo test` executes
