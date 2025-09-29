# Trait Contracts for Implement Data Cleaner and update Feature Selector for Option<f64>

## `DataCleaner` Trait

### Purpose
To define the interface for data cleaning operations, specifically converting `CausalTensor<f64>` to `CausalTensor<Option<f64>>`.

### Method Signature
```rust
fn process(
    &self,
    tensor: CausalTensor<f64>,
) -> Result<CausalTensor<Option<f64>>, DataCleaningError>;
```

### Inputs
- `tensor`: A `CausalTensor<f64>` containing the raw floating-point data, potentially with `NaN` values.

### Outputs
- `Result<CausalTensor<Option<f64>>, DataCleaningError>`: On success, a `CausalTensor<Option<f64>>` where `f64` values are wrapped in `Some()` and `NaN` values are replaced with `None`. On failure, a `DataCleaningError`.

## `FeatureSelector` Trait

### Purpose
To define the interface for feature selection algorithms, updated to operate on `CausalTensor<Option<f64>>`.

### Method Signature
```rust
fn select(
    &self,
    tensor: CausalTensor<Option<f64>>,
    config: &FeatureSelectorConfig,
) -> Result<CausalTensor<Option<f64>>, FeatureSelectError>;
```

### Inputs
- `tensor`: A `CausalTensor<Option<f64>>` containing data where missing values are represented by `None`.
- `config`: A `FeatureSelectorConfig` enum containing the specific settings for the selection algorithm.

### Outputs
- `Result<CausalTensor<Option<f64>>, FeatureSelectError>`: On success, a new `CausalTensor<Option<f64>>` with only the selected feature columns. On failure, a `FeatureSelectError`.
