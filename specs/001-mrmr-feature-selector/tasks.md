# Task Plan: mRMR Feature Selection

This document breaks down the implementation of the mRMR (FCQ variant) feature selection algorithm into a sequence of actionable tasks. Each task is derived from the requirements in `spec.md` and the technical decisions in `research.md`.

## Phase 1: Setup and Core Utilities

**[P]** marks tasks that can be worked on in parallel.

1.  **Create Module Structure**
    - **Description**: Create the necessary file and module structure for the new feature.
    - **Action**: Create the file `deep_causality_algorithms/src/feature_selection/mrmr.rs` and declare it in `deep_causality_algorithms/src/feature_selection/mod.rs`.
    - **Reference**: `spec.md` (FR-002)

2.  **Define Error Enum** **[P]**
    - **Description**: Define the custom `MrmrError` enum to handle all feature-specific errors.
    - **Action**: In `mrmr.rs`, define the `MrmrError` enum with variants `InvalidInput(String)`, `CalculationError(String)`, and `NotEnoughFeatures`.
    - **Reference**: `data-model.md` (Section 2), `spec.md` (FR-006)

3.  **Implement Pearson Correlation Helper**
    - **Description**: Create a helper function to calculate the Pearson correlation between two columns of a `CausalTensor`. This is a core component of the redundancy calculation.
    - **Action**:
        1.  Write a failing unit test for the `pearson_correlation` function.
        2.  Implement the `pearson_correlation` function, including checks for zero variance to prevent division-by-zero errors.
    - **Reference**: `research.md` (Decision: Redundancy)

4.  **Implement F-Statistic Helper**
    - **Description**: Create a helper function to calculate the F-statistic between a feature and a target column. This is the core of the relevance calculation.
    - **Action**:
        1.  Write a failing unit test for the `f_statistic` function.
        2.  Implement the `f_statistic` function using the formula `F = (n-2) * r^2 / (1 - r^2)`, where `r` is the Pearson correlation.
    - **Reference**: `research.md` (Decision: Relevance)

## Phase 2: Main Algorithm Implementation (TDD)

5.  **Write Failing Integration Test**
    - **Description**: Create a high-level integration test for the main `select_features` function. This test will fail until the full implementation is complete.
    - **Action**: Add a new test file in the `deep_causality_algorithms/tests` directory. Use the scenario from `quickstart.md` to create a test that calls `select_features` and asserts the final ranked list of features is correct (`vec![0, 2]`).
    - **Reference**: `quickstart.md` (Section 3), `contracts/mrmr.rs`

6.  **Implement `select_features` Function Stub**
    - **Description**: Create the public-facing `select_features` function signature and initial validation logic.
    - **Action**:
        1.  Add the function signature from `contracts/mrmr.rs` to `mrmr.rs`.
        2.  Implement input validation: check that the tensor is 2D, that `num_features` is not greater than the available features, and that the `target_column` is valid. Return `MrmrError::InvalidInput` or `MrmrError::NotEnoughFeatures` on failure.
    - **Reference**: `spec.md` (FR-003, FR-005), `contracts/mrmr.rs`

7.  **Implement Data Preprocessing**
    - **Description**: Add the logic to handle missing data as specified.
    - **Action**: Before the main loop, iterate through each column of the input `CausalTensor`. Calculate the mean for any column containing NaN values and replace them.
    - **Reference**: `spec.md` (FR-008)

8.  **Implement Core Iterative Selection Logic**
    - **Description**: Implement the main loop of the FCQ algorithm that iteratively selects the best feature.
    - **Action**:
        1.  Initialize `selected_features` and `remaining_features` sets.
        2.  Loop `num_features` times.
        3.  Inside the loop, iterate through all `remaining_features`.
        4.  For each candidate feature, calculate the **relevance** (using the `f_statistic` helper) and **redundancy** (using the `pearson_correlation` helper against already `selected_features`). Handle the initial case where redundancy is 0 by using a small epsilon, as noted in `plan.md` (Risk R-04).
        5.  Calculate the FCQ score (`relevance / redundancy`).
        6.  Identify the feature with the max score, move it from `remaining_features` to `selected_features`.
    - **Reference**: `research.md` (Decision: Algorithm Variant), `plan.md` (Risk R-04)

## Phase 3: Finalization

9.  **Ensure All Tests Pass**
    - **Description**: Run the entire test suite to ensure the new implementation is correct and has not caused any regressions.
    - **Action**: Run `cargo test --all`.

10. **Code Quality and Linting**
    - **Description**: Format and lint the code to adhere to project standards.
    - **Action**: Run `cargo fmt` and `cargo clippy -- -D warnings`.
    - **Reference**: `spec.md` (NFR-005)
