# Tasks: Causal Discovery DSL

This document outlines the actionable, dependency-ordered tasks for implementing the Causal Discovery DSL feature.

## Feature: Causal Discovery DSL

## Task Generation Strategy:

Tasks are generated from the design documents (`plan.md`, `research.md`, `data-model.md`, `contracts/CQD_DSL.md`, `quickstart.md`) and ordered by dependencies, following a Test-Driven Development (TDD) approach where tests are written before implementation.

## Ordering Strategy:

1.  **Setup**: Project initialization and dependency management.
2.  **Error Handling**: Define core error types.
3.  **Configuration**: Implement configuration structs.
4.  **Traits**: Define and implement core traits.
5.  **Core DSL Structure**: Implement the `CQD` type with its typestate transitions and the `CQDRunner`.
6.  **Algorithm Integration**: Implement wrappers for feature selection and causal discovery algorithms.
7.  **Analysis & Formatting**: Implement the result analysis heuristics and formatting logic.
8.  **Integration Tests**: End-to-end tests based on user stories.
9.  **Unit Tests**: Granular tests for individual components.
10. **Polish**: Documentation, benchmarking, and final review.

## Estimated Output:

Approximately 30-40 tasks, clearly numbered and with file paths.

## Tasks

### Setup

- **T001**: Create new crate `deep_causality_discovery` in the monorepo.
    - **File**: `deep_causality_discovery/Cargo.toml`
    - **Command**: `cargo new --lib deep_causality_discovery` (then move/adjust in monorepo structure)
- **T002**: Add `deep_causality_algorithms`, `deep_causality_tensor`, `csv`, `parquet` as dependencies to `deep_causality_discovery/Cargo.toml`.
    - **File**: `deep_causality_discovery/Cargo.toml`
    - **Depends on**: T001
- **T003**: Configure `deep_causality_discovery/Cargo.toml` for monorepo integration (e.g., workspace settings).
    - **File**: `deep_causality_discovery/Cargo.toml`
    - **Depends on**: T002

### Error Handling

- **T004**: Define `CqdError` enum, encapsulating specific error variants for each stage.
    - **File**: `deep_causality_discovery/src/error.rs`
    - **Depends on**: T003
- **T005 [P]**: Define `DataError` enum (FileNotFound, PermissionDenied, os_error).
    - **File**: `deep_causality_discovery/src/error.rs`
    - **Depends on**: T004
- **T006 [P]**: Define `FeatureSelectError` enum (TooFewFeatures, MRMRError, TensorError).
    - **File**: `deep_causality_discovery/src/error.rs`
    - **Depends on**: T004
- **T007 [P]**: Define `CausalDiscoveryError` enum (TensorError).
    - **File**: `deep_causality_discovery/src/error.rs`
    - **Depends on**: T004
- **T008 [P]**: Define `AnalyzeError` enum (EmptyResult, AnalysisFailed, TensorError).
    - **File**: `deep_causality_discovery/src/error.rs`
    - **Depends on**: T004
- **T009 [P]**: Define `FinalizeError` enum (FormattingError).
    - **File**: `deep_causality_discovery/src/error.rs`
    - **Depends on**: T004

### Configuration Structs

- **T010**: Implement `MrmrConfig` struct (`num_features`, `target_col`).
    - **File**: `deep_causality_discovery/src/config.rs`
    - **Depends on**: T009
- **T011 [P]**: Implement `SurdConfig` struct (`max_order`).
    - **File**: `deep_causality_discovery/src/config.rs`
    - **Depends on**: T009
- **T012 [P]**: Implement `CsvConfig` struct (`has_headers`, `delimiter`, `skip_rows`, `columns`).
    - **File**: `deep_causality_discovery/src/config.rs`
    - **Depends on**: T009
- **T013 [P]**: Implement `ParquetConfig` struct (`columns`, `batch_size`).
    - **File**: `deep_causality_discovery/src/config.rs`
    - **Depends on**: T009

### Traits

- **T014**: Define `ProcessDataLoader` trait.
    - **File**: `deep_causality_discovery/src/traits.rs`
    - **Depends on**: T013
- **T015 [P]**: Define `FeatureSelector` trait.
    - **File**: `deep_causality_discovery/src/traits.rs`
    - **Depends on**: T013
- **T016 [P]**: Define `CausalDiscovery` trait.
    - **File**: `deep_causality_discovery/src/traits.rs`
    - **Depends on**: T013
- **T017 [P]**: Define `ProcessResultAnalyzer` trait.
    - **File**: `deep_causality_discovery/src/traits.rs`
    - **Depends on**: T013
- **T018 [P]**: Define `ProcessResultFormatter` trait.
    - **File**: `deep_causality_discovery/src/traits.rs`
    - **Depends on**: T013

### Core DSL Structure

- **T019**: Implement `CQD<NoData>` and its `new()` and `start()` methods.
    - **File**: `deep_causality_discovery/src/cqd.rs`
    - **Depends on**: T018
- **T020**: Implement `CQD<WithData>` and its `feat_select()` method.
    - **File**: `deep_causality_discovery/src/cqd.rs`
    - **Depends on**: T019
- **T021**: Implement `CQD<WithFeatures>` and its `causal_discovery()` method.
    - **File**: `deep_causality_discovery/src/cqd.rs`
    - **Depends on**: T020
- **T022**: Implement `CQD<WithCausalResults>` and its `analyze()` method.
    - **File**: `deep_causality_discovery/src/cqd.rs`
    - **Depends on**: T021
- **T023**: Implement `CQD<WithAnalysis>` and its `finalize()` method.
    - **File**: `deep_causality_discovery/src/cqd.rs`
    - **Depends on**: T022
- **T024**: Implement `CQD<Finalized>` and its `build()` method.
    - **File**: `deep_causality_discovery/src/cqd.rs`
    - **Depends on**: T023
- **T025**: Implement `CQDRunner` and its `run()` method.
    - **File**: `deep_causality_discovery/src/cqd.rs`
    - **Depends on**: T024

### Algorithm Integration

- **T026**: Implement `ProcessDataLoader` for CSV files.
    - **File**: `deep_causality_discovery/src/data_loader/csv.rs`
    - **Depends on**: T025
- **T027 [P]**: Implement `ProcessDataLoader` for Parquet files.
    - **File**: `deep_causality_discovery/src/data_loader/parquet.rs`
    - **Depends on**: T025
- **T028**: Implement `FeatureSelector` for mRMR algorithm.
    - **File**: `deep_causality_discovery/src/feature_selector/mrmr.rs`
    - **Depends on**: T027
- **T029**: Implement `CausalDiscovery` for SURD algorithm.
    - **File**: `deep_causality_discovery/src/causal_discovery/surd.rs`
    - **Depends on**: T028

### Analysis & Formatting

- **T030**: Implement `ProcessAnalysis` struct.
    - **File**: `deep_causality_discovery/src/analysis.rs`
    - **Depends on**: T029
- **T031**: Implement `ProcessResultAnalyzer` with SURD to Causaloid heuristics.
    - **File**: `deep_causality_discovery/src/analysis.rs`
    - **Depends on**: T030
- **T032**: Implement `ProcessFormattedResult` struct.
    - **File**: `deep_causality_discovery/src/formatter.rs`
    - **Depends on**: T031
- **T033**: Implement `ProcessResultFormatter` for console output.
    - **File**: `deep_causality_discovery/src/formatter.rs`
    - **Depends on**: T032

### Integration Tests

- **T034**: Write integration test for CSV data loading, full DSL process, and formatted output (Acceptance Scenario 1).
    - **File**: `deep_causality_discovery/tests/integration_tests.rs`
    - **Depends on**: T033
- **T035 [P]**: Write integration test for Parquet data loading, full DSL process, and formatted output (Acceptance Scenario 2).
    - **File**: `deep_causality_discovery/tests/integration_tests.rs`
    - **Depends on**: T033
- **T036 [P]**: Write integration test for mRMR feature selection (Acceptance Scenario 3).
    - **File**: `deep_causality_discovery/tests/integration_tests.rs`
    - **Depends on**: T033
- **T037 [P]**: Write integration test for SURD causal discovery (Acceptance Scenario 4).
    - **File**: `deep_causality_discovery/tests/integration_tests.rs`
    - **Depends on**: T033
- **T038 [P]**: Write integration test for SURD result analysis (Acceptance Scenario 5).
    - **File**: `deep_causality_discovery/tests/integration_tests.rs`
    - **Depends on**: T033
- **T039 [P]**: Write integration test for console output formatting (Acceptance Scenario 6).
    - **File**: `deep_causality_discovery/tests/integration_tests.rs`
    - **Depends on**: T033

### Unit Tests

- **T040 [P]**: Write unit tests for `CqdError` and nested error enums.
    - **File**: `deep_causality_discovery/src/error.rs`
    - **Depends on**: T009
- **T041 [P]**: Write unit tests for configuration structs.
    - **File**: `deep_causality_discovery/src/config.rs`
    - **Depends on**: T013
- **T042 [P]**: Write unit tests for traits (if applicable, e.g., default implementations).
    - **File**: `deep_causality_discovery/src/traits.rs`
    - **Depends on**: T018
- **T043 [P]**: Write unit tests for `CQD` methods and typestate transitions.
    - **File**: `deep_causality_discovery/src/cqd.rs`
    - **Depends on**: T025
- **T044 [P]**: Write unit tests for data loaders (CSV, Parquet).
    - **File**: `deep_causality_discovery/src/data_loader/mod.rs`
    - **Depends on**: T027
- **T045 [P]**: Write unit tests for feature selectors (mRMR).
    - **File**: `deep_causality_discovery/src/feature_selector/mod.rs`
    - **Depends on**: T028
- **T046 [P]**: Write unit tests for causal discovery algorithms (SURD).
    - **File**: `deep_causality_discovery/src/causal_discovery/mod.rs`
    - **Depends on**: T029
- **T047 [P]**: Write unit tests for result analysis and heuristics.
    - **File**: `deep_causality_discovery/src/analysis.rs`
    - **Depends on**: T031
- **T048 [P]**: Write unit tests for result formatting.
    - **File**: `deep_causality_discovery/src/formatter.rs`
    - **Depends on**: T033

### Polish

- **T049**: Add comprehensive documentation for the `deep_causality_discovery` crate and its public API.
    - **File**: `deep_causality_discovery/src/lib.rs`, `deep_causality_discovery/src/**/*.rs`
    - **Depends on**: T048
- **T050**: Implement benchmarks for key performance-critical sections (e.g., data loading, feature selection, causal discovery).
    - **File**: `deep_causality_discovery/benches/benchmarks.rs`
    - **Depends on**: T049
- **T051**: Review and refactor code for adherence to Rust best practices and project conventions.
    - **File**: `deep_causality_discovery/src/**/*.rs`
    - **Depends on**: T050
