# Data Model: Causal Discovery DSL

This document outlines the key entities and their relationships for the Causal Discovery DSL feature, as extracted from the feature specification.

## Key Entities

### CQD (Causal Qualities Discovery)
- **Description**: The main type for orchestrating causal discovery workflows.
- **Fields**: Internally manages the state of the discovery process, transitioning through different types based on the Typestate Pattern.
- **Relationships**: Composes various trait objects (`ProcessDataLoader`, `FeatureSelector`, `CausalDiscovery`, `ProcessResultAnalyzer`, `ProcessResultFormatter`).

### ProcessDataLoader
- **Description**: Trait for loading data (e.g., from CSV, Parquet) into a `CausalTensor`.
- **Methods**: `load_data(config: DataLoaderConfig) -> Result<CausalTensor, DataError>` (conceptual)

### FeatureSelector
- **Description**: Trait for applying feature selection algorithms (e.g., mRMR) to a `CausalTensor`.
- **Methods**: `select_features(tensor: CausalTensor, config: FeatureSelectorConfig) -> Result<CausalTensor, FeatureSelectError>` (conceptual)

### CausalDiscovery
- **Description**: Trait for applying causal discovery algorithms (e.g., SURD) to a `CausalTensor`.
- **Methods**: `discover_causality(tensor: CausalTensor, config: CausalDiscoveryConfig) -> Result<SurdResult, CausalDiscoveryError>` (conceptual)

### ProcessResultAnalyzer
- **Description**: Trait for analyzing causal discovery results and recommending `Causaloid` structures.
- **Methods**: `analyze_results(surd_result: SurdResult) -> Result<ProcessAnalysis, AnalyzeError>` (conceptual)

### ProcessAnalysis
- **Description**: The output of the `ProcessResultAnalyzer`, containing recommendations for `Causaloid` structures.
- **Fields**: Contains information to construct `CausaloidGraph`, `Causaloid`, and `Causaloid Collection`.

### ProcessResultFormatter
- **Description**: Trait for formatting `ProcessAnalysis` into a presentable result.
- **Methods**: `format_results(analysis: ProcessAnalysis) -> Result<ProcessFormattedResult, FinalizeError>` (conceptual)

### ProcessFormattedResult
- **Description**: The final formatted output of the causal discovery process.

### CausalTensor
- **Description**: A multi-dimensional array for numerical data, used throughout the process.

### CausaloidGraph
- **Description**: A graph structure representing causal links.

### Causaloid
- **Description**: A self-contained unit of causality with internal logic.

### Causaloid Collection
- **Description**: A container for multiple `Causaloids` with aggregate logic.

### MrmrConfig
- **Description**: Configuration for the mRMR algorithm.
- **Fields**: `num_features` (usize), `target_col` (column index).

### SurdConfig
- **Description**: Configuration for the SURD algorithm.
- **Fields**: `max_order` (`MaxOrder` enum).

### AnalyzeConfig
- **Description**: Configuration for the analysis heuristics.
- **Fields**: `synergy_threshold` (f64), `unique_threshold` (f64), `redundancy_threshold` (f64).

### AnalyzeConfig
- **Description**: Configuration for the analysis heuristics.
- **Fields**: `synergy_threshold` (f64), `unique_threshold` (f64), `redundancy_threshold` (f64).

### CsvConfig
- **Description**: Configuration for CSV data loading.
- **Fields**: `has_headers` (boolean), `delimiter` (byte), `skip_rows` (usize), optional `columns` (vector of strings).

### ParquetConfig
- **Description**: Configuration for Parquet data loading.
- **Fields**: optional `columns` (vector of strings), `batch_size` (usize).

## Error Types (as defined in spec.md)
- `CqdError` (main error enum)
- `DataError`
- `FeatureSelectError`
- `CausalDiscoveryError`
- `AnalyzeError`
- `FinalizeError`
