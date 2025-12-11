# Tasks for Implement Data Cleaner and update Feature Selector for Option<f64>

This document outlines the tasks required to implement the `OptionNoneDataCleaner` and update the `FeatureSelector` trait and `MrmrFeatureSelector` to work with `CausalTensor<Option<f64>>`.

## Setup

- **T001** [P]: Verify `deep_causality_discovery` crate dependencies are correctly configured.
  - **Command**: `cargo check -p deep_causality_discovery`
  - **File**: `deep_causality_discovery/Cargo.toml`

## Data Cleaner Implementation

- **T002** [P]: Create a new test file for `OptionNoneDataCleaner` in `deep_causality_discovery/tests/types/data_cleaner/option_none_data_cleaner_tests.rs`.
  - **Description**: Write a test case that creates a `CausalTensor<f64>` with `NaN` values, applies `OptionNoneDataCleaner`, and asserts that `f64` values are wrapped in `Some()` and `NaN` values are replaced with `None` in the resulting `CausalTensor<Option<f64>>`. This test should initially fail.
  - **File**: `deep_causality_discovery/tests/types/data_cleaner/option_none_data_cleaner_tests.rs`
  - **Command**: `cargo test -p deep_causality_discovery --test option_none_data_cleaner_tests`

- **T003**: Update the `DataCleaner` trait in `deep_causality_discovery/src/traits/data_cleaner.rs` to return `Result<CausalTensor<Option<f64>>, DataCleaningError>`.
  - **Description**: Modify the `process` method signature in the `DataCleaner` trait.
  - **File**: `deep_causality_discovery/src/traits/data_cleaner.rs`

- **T004**: Implement the `DataCleaner` trait for `OptionNoneDataCleaner` in `deep_causality_discovery/src/types/data_cleaner/option_none.rs`.
  - **Description**: Implement the `process` method to convert `CausalTensor<f64>` to `CausalTensor<Option<f64>>`, mapping `f64` to `Some(f64)` and `NaN` to `None`.
  - **File**: `deep_causality_discovery/src/types/data_cleaner/option_none.rs`
  - **Depends on**: T003

- **T005** [P]: Run tests for `OptionNoneDataCleaner` to ensure they pass.
  - **Command**: `cargo test -p deep_causality_discovery --test option_none_data_cleaner_tests`
  - **Depends on**: T002, T004

## Feature Selector Update

- **T006** [P]: Create a new test file for `FeatureSelector` trait update in `deep_causality_discovery/tests/traits/feature_selector_update_tests.rs`.
  - **Description**: Write a test case that attempts to define a mock `FeatureSelector` implementation that takes `CausalTensor<Option<f64>>` as input and returns `CausalTensor<Option<f64>>`. This test should initially fail due to the trait signature mismatch.
  - **File**: `deep_causality_discovery/tests/traits/feature_selector_update_tests.rs`
  - **Command**: `cargo test -p deep_causality_discovery --test feature_selector_update_tests`

- **T007**: Update the `FeatureSelector` trait in `deep_causality_discovery/src/traits/feature_selector.rs` to accept `CausalTensor<Option<f64>>` as input and return `CausalTensor<Option<f64>>` as output.
  - **Description**: Modify the `select` method signature in the `FeatureSelector` trait.
  - **File**: `deep_causality_discovery/src/traits/feature_selector.rs`
  - **Depends on**: T003 (potential conflict, but trait changes are independent)

- **T008** [P]: Run tests for `FeatureSelector` trait update to ensure they pass.
  - **Command**: `cargo test -p deep_causality_discovery --test feature_selector_update_tests`
  - **Depends on**: T006, T007

- **T009** [P]: Create a new test file for `MrmrFeatureSelector` update in `deep_causality_discovery/tests/types/feature_selector/mrmr_feature_selector_update_tests.rs`.
  - **Description**: Write a test case that creates a `CausalTensor<Option<f64>>` with `Some` and `None` values, applies `MrmrFeatureSelector`, and asserts that feature selection is performed correctly using `mrmr_features_selector_cdl` while ignoring `None` values. This test should initially fail.
  - **File**: `deep_causality_discovery/tests/types/feature_selector/mrmr_feature_selector_update_tests.rs`
  - **Command**: `cargo test -p deep_causality_discovery --test mrmr_feature_selector_update_tests`

- **T010**: Update `MrmrFeatureSelector` in `deep_causality_discovery/src/types/feature_selector/mrmr.rs` to use `CausalTensor<Option<f64>>` as input/output and call `mrmr_features_selector_cdl`.
  - **Description**: Modify the `select` method implementation in `MrmrFeatureSelector` to adapt to the new `CausalTensor<Option<f64>>` type and delegate to `mrmr_features_selector_cdl`.
  - **File**: `deep_causality_discovery/src/types/feature_selector/mrmr.rs`
  - **Depends on**: T007

- **T011** [P]: Run tests for `MrmrFeatureSelector` update to ensure they pass.
  - **Command**: `cargo test -p deep_causality_discovery --test mrmr_feature_selector_update_tests`
  - **Depends on**: T009, T010

## Causal Discovery Update

- **T012** [P]: Create a new test file for `CausalDiscovery` trait update in `deep_causality_discovery/tests/traits/causal_discovery_update_tests.rs`.
  - **Description**: Write a test case that attempts to define a mock `CausalDiscovery` implementation that takes `CausalTensor<Option<f64>>` as input. This test should initially fail due to the trait signature mismatch.
  - **File**: `deep_causality_discovery/tests/traits/causal_discovery_update_tests.rs`
  - **Command**: `cargo test -p deep_causality_discovery --test causal_discovery_update_tests`

- **T013**: Update the `CausalDiscovery` trait in `deep_causality_discovery/src/traits/causal_discovery.rs` to accept `CausalTensor<Option<f64>>` as input.
  - **Description**: Modify the `discover` method signature in the `CausalDiscovery` trait.
  - **File**: `deep_causality_discovery/src/traits/causal_discovery.rs`
  - **Depends on**: T012 (test should now compile)

- **T014** [P]: Run tests for `CausalDiscovery` trait update to ensure they pass.
  - **Command**: `cargo test -p deep_causality_discovery --test causal_discovery_update_tests`
  - **Depends on**: T012, T013

- **T015**: Implement `surd_states_cdl` in `deep_causality_algorithms/src/surd/surd_algo_cdl.rs` to handle `CausalTensor<Option<f64>>`.
  - **Description**: This function should perform SURD analysis on `CausalTensor<Option<f64>>` using pairwise deletion for `None` values, similar to `mrmr_features_selector_cdl`. This is a conceptual task as the actual implementation is outside `deep_causality_discovery`.
  - **File**: `deep_causality_algorithms/src/surd/surd_algo_cdl.rs` (new file)
  - **Depends on**: T014 (conceptual)

- **T016**: Update `SurdCausalDiscovery` in `deep_causality_discovery/src/types/causal_discovery/surd.rs` to use `CausalTensor<Option<f64>>` as input and call `surd_states_cdl`.
  - **Description**: Modify the `discover` method implementation in `SurdCausalDiscovery` to adapt to the new `CausalTensor<Option<f64>>` type and delegate to `surd_states_cdl`.
  - **File**: `deep_causality_discovery/src/types/causal_discovery/surd.rs`
  - **Depends on**: T013, T015 (conceptual)

- **T017**: Update `CDL<WithFeatures>::causal_discovery` in `deep_causality_discovery/src/types/cdl/mod.rs` to pass `CausalTensor<Option<f64>>` to `discovery.discover`.
  - **Description**: Modify the `causal_discovery` method in the `CDL` pipeline to pass the `CausalTensor<Option<f64>>` from `WithFeatures` state to the `discover` method.
  - **File**: `deep_causality_discovery/src/types/cdl/mod.rs`
  - **Depends on**: T016

## Integration and Polish

- **T018** [P]: Create an integration test `deep_causality_discovery/tests/integration/quickstart_integration_tests.rs` that mirrors the `quickstart.md` flow.
  - **Description**: This test should cover the full flow from raw `CausalTensor<f64>` to `OptionNoneDataCleaner` to `MrmrFeatureSelector` with `CausalTensor<Option<f64>>` and then to `CausalDiscovery` with `CausalTensor<Option<f64>>`.
  - **File**: `deep_causality_discovery/tests/integration/quickstart_integration_tests.rs`
  - **Command**: `cargo test -p deep_causality_discovery --test quickstart_integration_tests`
  - **Depends on**: T005, T011, T017

- **T019**: Update documentation for `OptionNoneDataCleaner`, `DataCleaner` trait, `FeatureSelector` trait, `MrmrFeatureSelector`, `CausalDiscovery` trait, and `SurdCausalDiscovery`.
  - **Description**: Ensure all relevant public APIs have up-to-date documentation reflecting the changes.
  - **Files**:
    - `deep_causality_discovery/src/types/data_cleaner/option_none.rs`
    - `deep_causality_discovery/src/traits/data_cleaner.rs`
    - `deep_causality_discovery/src/traits/feature_selector.rs`
    - `deep_causality_discovery/src/types/feature_selector/mrmr.rs`
    - `deep_causality_discovery/src/traits/causal_discovery.rs`
    - `deep_causality_discovery/src/types/causal_discovery/surd.rs`

- **T020** [P]: Run `make format` and `make fix` to ensure code style and linting.
  - **Command**: `make format && make fix`

- **T021** [P]: Run all tests for `deep_causality_discovery` crate.
  - **Command**: `cargo test -p deep_causality_discovery`

- **T022** [P]: Run all tests for the entire mono-repo.
  - **Command**: `make test`
