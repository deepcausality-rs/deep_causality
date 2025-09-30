# Tasks: MRMR Feature Score

**Input**: Design documents from `/specs/006-mrmr-feature-score/`

## Phase 3.1: Core Types & Errors
- [ ] T001 Add `FeatureScoreError(String)` variant to `deep_causality_algorithms/src/feature_selection/mrmr/mrmr_error.rs`.

## Phase 3.2: Tests First (TDD) ⚠️ MUST COMPLETE BEFORE 3.3
- [ ] T002 [P] Add a basic test for the new `FeatureScoreError` variant in `deep_causality_algorithms/tests/feature_selection/mrmr/mrmr_error_tests.rs`.
- [ ] T003 [P] Update tests in `deep_causality_algorithms/tests/feature_selection/mrmr/mrmr_algo_tests.rs` to expect `Result<Vec<(usize, f64)>, MrmrError>` and assert that scores are returned.
- [ ] T004 [P] Update tests in `deep_causality_algorithms/tests/feature_selection/mrmr/mrmr_algo_cdl_tests.rs` to expect `Result<Vec<(usize, f64)>, MrmrError>` and assert that scores are returned.
- [ ] T005 [P] In `mrmr_algo_tests.rs`, add a new test case that causes a `NaN` or `Infinity` score and asserts that `MrmrError::FeatureScoreError` is returned.

## Phase 3.3: Core Implementation (ONLY after tests are failing)
- [ ] T006 Modify the `mrmr_features_selector` function in `deep_causality_algorithms/src/feature_selection/mrmr/mrmr_algo.rs` to:
    - Capture the relevance score for the first feature.
    - Capture the mRMR score for subsequent features.
    - Return `Ok(Vec<(usize, f64)>)`.
    - Add a check for `NaN` or `Infinity` scores and return `MrmrError::FeatureScoreError` if found.
- [ ] T007 [P] Modify the `mrmr_features_selector_cdl` function in `deep_causality_algorithms/src/feature_selection/mrmr/mrmr_algo_cdl.rs` with the same changes as T006.

## Phase 3.4: Polish
- [ ] T008 [P] Update the rustdoc comments for `mrmr_features_selector` in `mrmr_algo.rs` to reflect the new return type and behavior.
- [ ] T009 [P] Update the rustdoc comments for `mrmr_features_selector_cdl` in `mrmr_algo_cdl.rs` to reflect the new return type and behavior.
- [ ] T010 Create a new benchmark file `deep_causality_algorithms/benches/mrmr_benchmark.rs` and add a benchmark test to compare the performance of the old vs. new implementation to ensure it stays within the 10% degradation budget.

## Dependencies
- `T001` must be done before all other tasks.
- `T002`, `T003`, `T004`, `T005` must be done before `T006` and `T007`.
- `T006` and `T007` can be done in parallel.
- `T008`, `T009`, `T010` can be done after `T006` and `T007` are complete.

## Parallel Example
```bash
# You can run these test-related tasks in parallel
# Task: "[T003] Update tests in deep_causality_algorithms/tests/feature_selection/mrmr/mrmr_algo_tests.rs..."
# Task: "[T004] Update tests in deep_causality_algorithms/tests/feature_selection/mrmr/mrmr_algo_cdl_tests.rs..."
# Task: "[T005] In mrmr_algo_tests.rs, add a new test case that causes a NaN or Infinity score..."
```
