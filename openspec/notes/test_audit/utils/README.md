<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Utility crates test-suite audit

Scanned 2026-09-08: **155 test functions across 34 files**, comprising 2,443 test lines against
3,603 source lines in the three crates under `deep_causality_utils/`:

* `deep_causality_ast`
* `deep_causality_file`
* `deep_causality_par`

The scanner is `openspec/notes/test_audit/physics/audit_tests.py`. It was run unchanged except for
substituting its hard-coded `ROOT` value in memory for each crate. The script's output was then
calibrated by reading every site it classified as `no-assertion` or `cherry-picked`, and by checking
the suites for conditional early exits that the scanner cannot detect.

## Finding in one line

**The utility suites do not reproduce the physics crate's widespread tautology, circular-oracle, or
magic-number problem. They do contain three confirmed ineffective-test defects, all in
`deep_causality_file`, while the high single-input count remains a risk signal rather than a defect
count.**

The raw scan found no tautologies and no circular expectations. After calibration, none of its seven
`cherry-picked` findings was a defect. Two of its three `no-assertion` findings are ineffective
laziness tests; the third is a valid compile-time trait-bound test. A separate manual check found one
environment-dependent integration test that passes without exercising the loader.

## Raw scanner results

| Crate | Tests | Files | single-input | cherry-picked | tautology | circular | no-assertion |
|---|---:|---:|---:|---:|---:|---:|---:|
| `deep_causality_ast` | 24 | 12 | 23 (95.8%) | 0 | 0 | 0 | 0 |
| `deep_causality_file` | 122 | 20 | 105 (86.1%) | 6 (4.9%) | 0 | 0 | 2 (1.6%) |
| `deep_causality_par` | 9 | 2 | 7 (77.8%) | 1 (11.1%) | 0 | 0 | 1 (11.1%) |
| **Total** | **155** | **34** | **135 (87.1%)** | **7 (4.5%)** | **0** | **0** | **3 (1.9%)** |

The classes overlap. A `cherry-picked` test is also `single-input` by construction.

For comparison, a fresh run of the same scanner over the current physics tree found 116 tautologies,
52 circular expectations, and 632 cherry-picked tests among 1,766 tests; it found no assertion-free
test. On the same detector, the suspected defect pattern is therefore **not** present in the utility
crates at a comparable rate. The 87.1% single-input rate is close to physics's current 89.8%, but that
detector is too broad to establish a defect on its own. These current physics counts supersede the
older snapshots recorded in the physics audit markdown for purposes of this comparison.

## Confirmed defects

### 1. The real-data integration test can pass without running

`deep_causality_file/tests/load_real_galileo.rs:39-45` returns successfully when either fixture is
absent:

```rust
if !clk.exists() || !sp3.exists() {
    eprintln!("skipping: Galileo fixtures not present at {}", data_dir().display());
    return;
}
```

This is a false-green path, not a test skip visible to the harness. More importantly, the test's own
`data_dir` documentation says that under `bazel test` neither runtime path variable is set and the
fixtures are not declared as data. `deep_causality_file/BUILD.bazel` registers the integration test
but supplies no fixture data. The Bazel test therefore succeeds while exercising none of the loader,
the real files, or the composed action.

The Cargo run performed for this audit did exercise the test because both fixture files are present
in the workspace. That does not remove the CI blind spot.

**Required repair:** declare the two files as Bazel test data and make absence a failure. If the test
is intentionally unsupported under a runner, exclude it explicitly instead of returning success
from the test body.

### 2. The clock laziness test has no behavioural oracle

`deep_causality_file/tests/types/loaders/read_clk_tests.rs:92` only checks that the expression has the
declared return type:

```rust
let _action: ReadClockData<f64> = read_clock_data::<f64>("unread.clk", "E14");
```

Compilation proves the API shape, but the test name and comment claim the stronger property that
construction performs no I/O. The test never observes that property. An implementation that performs
work during construction and still returns `ReadClockData` would satisfy this test whenever that work
does not panic.

**Required repair:** follow the existing `read_table` and `read_sensor_trace` pattern: construct the
action while its path is absent, create a valid file afterwards, run the saved action, and assert the
parsed clock value.

### 3. The orbit laziness test has the same defect

`deep_causality_file/tests/types/loaders/read_sp3_tests.rs:195` likewise only assigns the returned
value to `ReadOrbitData<f64>` and makes no observation.

**Required repair:** construct the action before its SP3 file exists, write a valid epoch and position
record, run the action, and assert the independently expected timestamp and metre-converted
coordinates.

## Calibration of the raw findings

### `no-assertion`: three raw, two defects

| Site | Calibration |
|---|---|
| `read_clk_tests.rs:92` | **Confirmed ineffective.** Type-checks the return value but does not test the claimed laziness. |
| `read_sp3_tests.rs:195` | **Confirmed ineffective.** Same defect for the SP3 loader. |
| `maybe_parallel_tests.rs:27` | **False positive.** The property is compile-time satisfiability of `T: MaybeParallel + ?Sized` for `str` and slices. Successful compilation is the oracle. |

The scanner's definition of `no-assertion` is appropriate for runtime behaviour but cannot distinguish
a missing runtime oracle from a compile-pass contract.

### `cherry-picked`: seven raw, no confirmed defects

The seven flags fall into three legitimate test forms:

| Sites | Why the flag is not a defect |
|---|---|
| `orbit_types_tests.rs:19,35`; `clock_types_tests.rs:18` | Constructor/accessor round trips use exact values. The orbit radius uses a documented 3-4-12 triple with an independently known result of 13. |
| `read_clk_tests.rs:25`; `read_trace_tests.rs:21`; `read_gnss_tests.rs:61` | Parser fixtures state the input text and assert the decoded representation. These are direct format or delegation oracles, not values copied from a numeric implementation. |
| `maybe_parallel_tests.rs:14` | Numeric literals merely instantiate several scalar types for a compile-time blanket-trait contract. There is no numeric algorithm or external oracle. |

The physics detector deliberately treats an unexplained floating literal as suspicious. That
heuristic is useful for scientific kernels, but it overreaches on constructors, parsers, and marker
traits.

## The single-input result

`single-input` is the dominant raw class in every crate, but most instances are not broken tests.
Error paths such as a malformed SP3 month, an absent table column, or a truncated snapshot each pin a
specific branch and should not be converted into a range merely to satisfy the detector.

There is nevertheless one concentration worth further testing: **23 of 24 `deep_causality_ast`
tests are single-input.** Traversal, mapping, joining, search, size, and depth are each tested against
one principal non-trivial tree shape. The assertions are meaningful, so this is neither tautology nor
circular testing, but correlated indexing or traversal defects may survive when all expectations are
drawn from a small set of shallow trees. Mutation testing should decide whether this is a real gap;
the static detector cannot.

`deep_causality_par` is stronger than its 77.8% figure suggests. Its main `scoped_map` test compares
all 1,000 results with an independently executed sequential map, and the suite covers empty, singleton,
borrowed-closure, fallible, and over-core-count inputs. The same nine tests pass with and without the
`parallel` feature.

`deep_causality_file` similarly uses tables or loops where variation matters: all satellite enum
variants, awkward floating-point bit patterns, multi-row parser data, real GNSS series, and table
round trips. Its many single-input parser rejection tests represent distinct grammar or error branches,
not repeated sampling of one numeric formula.

## Scanner limitations exposed by this audit

The raw counts must not be converted directly into a remediation backlog:

1. The scanner recognizes assertions syntactically, not whether a test can return before reaching
   them. It missed the real-data test's false-green branch.
2. It treats successful compilation as assertion-free, even when compilation is exactly the trait
   contract being tested.
3. Its `cherry-picked` heuristic assumes numeric literals need scientific provenance. That assumption
   does not hold for data-format fixtures and constructor/accessor contracts.
4. `single-input` means only “no loop or table was detected.” It does not distinguish a singular
   boundary or error branch from an under-sampled algorithm.
5. The scan covers `#[test]` functions under `tests/`; it does not include doctests. Cargo additionally
   ran 19 doctests for `deep_causality_ast` and one for `deep_causality_file`.

## Verification

The audit script was run separately against all three test roots. The following suites were also run
to confirm the measured tests compile and pass:

```text
cargo test -p deep_causality_ast                         24 tests + 19 doctests passed
cargo test -p deep_causality_file                       122 tests + 1 doctest passed
cargo test -p deep_causality_par                         9 tests passed
cargo test -p deep_causality_par --features parallel     9 tests passed
```

No test implementation was changed as part of this audit.
