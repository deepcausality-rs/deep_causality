# Quickstart: Implement Data Cleaner and update Feature Selector for Option<f64>

This quickstart demonstrates the intended usage of the new `OptionNoneDataCleaner` and the updated `FeatureSelector` trait within the CDL pipeline.

## 1. Prepare Data with Missing Values

First, let's create a `CausalTensor<f64>` that includes some `NaN` (Not a Number) values to simulate missing data.

```rust
use deep_causality_tensor::CausalTensor;

let raw_data = vec![
    1.0, 2.0, f64::NAN,
    4.0, f64::NAN, 6.0,
    7.0, 8.0, 9.0,
];
let raw_tensor = CausalTensor::new(raw_data, vec![3, 3]).unwrap();

println!("Original CausalTensor<f64>:\n{:?}", raw_tensor);
```

## 2. Clean Data with `OptionNoneDataCleaner`

Now, we apply the `OptionNoneDataCleaner` to convert `f64` values to `Option<f64>`, where `NaN` values become `None`.

```rust
use deep_causality_discovery::{DataCleaner, OptionNoneDataCleaner};

let cleaner = OptionNoneDataCleaner;
let cleaned_tensor = cleaner.process(raw_tensor).unwrap();

println!("\nCleaned CausalTensor<Option<f64>>:\n{:?}", cleaned_tensor);
```

## 3. (Conceptual) Use with Updated `MrmrFeatureSelector`

The `cleaned_tensor` (of type `CausalTensor<Option<f64>>`) can then be passed to the updated `MrmrFeatureSelector`. This conceptual step illustrates how the feature selection will now operate on data where missing values are explicitly marked as `None`, preventing bias from imputation.

```rust
// This part is conceptual as the MrmrFeatureSelector implementation is not yet updated.
// It demonstrates the intended flow.

use deep_causality_discovery::{FeatureSelector, MrmrConfig, FeatureSelectorConfig, MrmrFeatureSelector};

// Assuming MrmrFeatureSelector has been updated to accept CausalTensor<Option<f64>>
let mrmr_config = MrmrConfig::new(2, 2); // Select 2 features, target column 2
let feature_selector_config = FeatureSelectorConfig::Mrmr(mrmr_config);

let selector = MrmrFeatureSelector;
// let selected_features_tensor = selector.select(cleaned_tensor, &feature_selector_config).unwrap();

// println!("\nSelected Features CausalTensor<Option<f64>>:\n{:?}", selected_features_tensor);
```
