# Quickstart: Using MRMR with Feature Scores

This guide demonstrates how to use the updated MRMR feature selection algorithm to get both the feature indices and their corresponding importance scores.

## Example

The function signature has been changed to return a `Vec<(usize, f64)>`.

```rust
use deep_causality_tensor::CausalTensor;
use deep_causality_algorithms::mrmr::mrmr_features_selector;

// 1. Prepare your data
let data = vec![
    1.0, 2.0, 3.0, 1.6,
    2.0, 4.1, 6.0, 3.5,
    3.0, 6.2, 9.0, 5.5,
    4.0, 8.1, 12.0, 7.5,
];
let mut tensor = CausalTensor::new(data, vec![4, 4]).unwrap();

// 2. Run the feature selector
// Select 2 features, with the target variable in column 3.
let selected_features_with_scores = mrmr_features_selector(&mut tensor, 2, 3).unwrap();

// 3. Interpret the results
println!("Selected features and their scores:");
for (index, score) in selected_features_with_scores {
    println!("- Feature Index: {}, Importance Score: {:.4}", index, score);
}

// Example Output:
// Selected features and their scores:
// - Feature Index: 2, Importance Score: 27.3378
// - Feature Index: 0, Importance Score: 1.5663
```

## Understanding the Scores

- **First Feature**: The score is its F-statistic (relevance). A higher value means it's more correlated with the target.
- **Subsequent Features**: The score is the mRMR score (Relevance / Redundancy). A higher value indicates a better trade-off between being relevant to the target and being different from already selected features.