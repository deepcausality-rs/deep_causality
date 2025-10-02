# Tasks: Parallelize mRMR algorithm with Rayon

This document outlines the tasks required to parallelize the mRMR feature selection algorithm using the `rayon` crate, guarded by a `parallel` feature flag.

## Setup Tasks

- [ ] **T001**: Verify `rayon` dependency and `parallel` feature flag are correctly configured in `deep_causality_algorithms/Cargo.toml`.

## Test Tasks

- [ ] **T002**: Create a new benchmark in `deep_causality_algorithms/benches/mrmr_benchmark.rs` to measure the performance of `mrmr_features_selector` with and without the `parallel` feature enabled. This benchmark should use a large dataset similar to the sepsis case study.
- [ ] **T003**: Run existing unit tests for `deep_causality_algorithms/src/feature_selection/mrmr/mrmr_algo.rs` with `cargo test -p deep_causality_algorithms` (sequential) and `cargo test -p deep_causality_algorithms --features parallel` (parallel) to ensure functional equivalence.
- [ ] **T004**: Run existing unit tests for `deep_causality_algorithms/src/feature_selection/mrmr/mrmr_algo_cdl.rs` with `cargo test -p deep_causality_algorithms` (sequential) and `cargo test -p deep_causality_algorithms --features parallel` (parallel) to ensure functional equivalence.

## Core Implementation Tasks

- [ ] **T005**: Modify `deep_causality_algorithms/src/feature_selection/mrmr/mrmr_algo.rs` to conditionally import `rayon::prelude::*` and parallelize the initial feature selection loop (finding the `first_feature`).
- [ ] **T006**: Modify `deep_causality_algorithms/src/feature_selection/mrmr/mrmr_algo.rs` to conditionally import `rayon::prelude::*` and parallelize the iterative feature selection loop (finding subsequent `best_feature`s).
- [ ] **T007**: Modify `deep_causality_algorithms/src/feature_selection/mrmr/mrmr_algo_cdl.rs` to conditionally import `rayon::prelude::*` and parallelize the initial feature selection loop (finding the `first_feature`).
- [ ] **T008**: Modify `deep_causality_algorithms/src/feature_selection/mrmr/mrmr_algo_cdl.rs` to conditionally import `rayon::prelude::*` and parallelize the iterative feature selection loop (finding subsequent `best_feature`s).

## Polish Tasks

- [ ] **T009**: Update relevant documentation (e.g., `deep_causality_algorithms/README.md`, function comments in `mrmr_algo.rs` and `mrmr_algo_cdl.rs`) to reflect the parallelization and the `parallel` feature flag.

## Parallel Execution Guidance

Tasks T005 and T006 can be considered as a single logical unit for `mrmr_algo.rs`. Similarly, T007 and T008 for `mrmr_algo_cdl.rs`. These two logical units (modifying `mrmr_algo.rs` and `mrmr_algo_cdl.rs`) can be worked on in parallel if desired, as they operate on separate files.

```bash
# Example of running tests sequentially and in parallel
cargo test -p deep_causality_algorithms
cargo test -p deep_causality_algorithms --features parallel

# Example of running benchmarks
cargo bench -p deep_causality_algorithms --bench mrmr_benchmark
cargo bench -p deep_causality_algorithms --bench mrmr_benchmark --features parallel
```
