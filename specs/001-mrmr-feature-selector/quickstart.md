# Quickstart: mRMR Feature Selection

This guide demonstrates how to use the mRMR feature selection function.

## 1. Setup

First, ensure you have a `CausalTensor` with your dataset.

```rust
use deep_causality_data_structures::CausalTensor;
use deep_causality_algorithms::feature_selection::mrmr::select_features;

// Create a sample 2D CausalTensor (e.g., 5 samples, 4 features + 1 target)
// Columns: [feature1, feature2, feature3, feature4, target]
let data = vec![
    1.0, 2.0, 3.0, 4.0, 1.0,
    2.0, 3.0, 4.0, 5.0, 0.0,
    3.0, 4.0, 5.0, 6.0, 1.0,
    4.0, 5.0, 6.0, 7.0, 0.0,
    5.0, 6.0, 7.0, 8.0, 1.0,
];
let shape = vec![5, 5];
let tensor = CausalTensor::new(data, shape).unwrap();
```

## 2. Run Feature Selection

Call the `select_features` function with your tensor, the target column index, and the number of features you want to select.

```rust
let target_column_index = 4;
let num_features_to_select = 2;

match select_features(&tensor, target_column_index, num_features_to_select) {
    Ok(selected_indices) => {
        println!("Selected feature indices: {:?}", selected_indices);
        // Expected output might be something like: [3, 0]
        // (The actual output will depend on the algorithm's calculations)
    }
    Err(e) => {
        eprintln!("Error during feature selection: {}", e);
    }
}
```

## 3. Test Scenario (for validation)

This test will verify that the `select_features` function correctly identifies the most relevant and least redundant features.

```rust
// Test setup:
// feature0 is highly correlated with the target.
// feature1 is also correlated with the target, but redundant with feature0.
// feature2 is independent and also correlated with the target.
let target = vec![1.0, 2.0, 3.0, 4.0, 5.0];
let feature0 = vec![1.1, 2.1, 3.1, 4.1, 5.1]; // High correlation with target
let feature1 = vec![1.2, 2.2, 3.2, 4.2, 5.2]; // Redundant with feature0
let feature2 = vec![5.0, 4.0, 3.0, 2.0, 1.0]; // Independent, correlated with target
let feature3 = vec![0.1, 0.9, 0.2, 0.8, 0.3]; // Irrelevant

let mut data_vec = Vec::new();
for i in 0..5 {
    data_vec.push(feature0[i]);
    data_vec.push(feature1[i]);
    data_vec.push(feature2[i]);
    data_vec.push(feature3[i]);
    data_vec.push(target[i]);
}

let tensor = CausalTensor::new(data_vec, vec![5, 5]).unwrap();

// Execute: Select top 2 features
let selected = select_features(&tensor, 4, 2).unwrap();

// Assert:
// The first feature selected should be feature0 due to highest relevance.
// The second feature should be feature2, as it's relevant but not redundant with feature0.
// feature1 should be skipped due to its high redundancy with feature0.
assert_eq!(selected, vec![0, 2]);
```
