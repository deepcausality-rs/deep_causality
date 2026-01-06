/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_discovery::{
    BinningStrategy, ColumnSelector, DataDiscretizer, DataPreprocessor, PreprocessConfig,
    PreprocessError,
};

use deep_causality_tensor::CausalTensor;

#[test]
fn test_data_discretizer_2d_tensor_check() {
    let discretizer = DataDiscretizer;
    let tensor = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap(); // 1D tensor
    let config = PreprocessConfig::new(BinningStrategy::EqualWidth, 2, ColumnSelector::All);

    let result = discretizer.process(tensor, &config);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        PreprocessError::BinningError("Tensor must be 2-dimensional".to_string())
    );
}

#[test]
fn test_data_discretizer_by_name_not_implemented() {
    let discretizer = DataDiscretizer;
    let tensor = CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();
    let config = PreprocessConfig::new(
        BinningStrategy::EqualWidth,
        2,
        ColumnSelector::ByName(vec!["col1".to_string()]),
    );

    let result = discretizer.process(tensor, &config);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        PreprocessError::ConfigError("ByName column selection is not yet implemented".to_string())
    );
}

#[test]
fn test_data_discretizer_invalid_column_index() {
    let discretizer = DataDiscretizer;
    let tensor = CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap(); // 2 columns
    let config = PreprocessConfig::new(
        BinningStrategy::EqualWidth,
        2,
        ColumnSelector::ByIndex(vec![0, 2]), // Column index 2 is out of bounds
    );

    let result = discretizer.process(tensor, &config);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        PreprocessError::InvalidColumnIdentifier(
            "Column index 2 is out of bounds for tensor with 2 columns".to_string()
        )
    );
}

#[test]
fn test_data_discretizer_equal_width_strategy_all_columns() {
    let discretizer = DataDiscretizer;
    let tensor = CausalTensor::new(
        vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0],
        vec![5, 2],
    )
    .unwrap();
    let config = PreprocessConfig::new(BinningStrategy::EqualWidth, 2, ColumnSelector::All);

    let result = discretizer.process(tensor, &config).unwrap();
    let expected = CausalTensor::new(
        vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0],
        vec![5, 2],
    )
    .unwrap();
    assert_eq!(result.as_slice(), expected.as_slice());
}

#[test]
fn test_data_discretizer_equal_width_strategy_by_index() {
    let discretizer = DataDiscretizer;
    let tensor = CausalTensor::new(
        vec![1.0, 10.0, 2.0, 9.0, 3.0, 8.0, 4.0, 7.0, 5.0, 6.0],
        vec![5, 2],
    )
    .unwrap();
    let config = PreprocessConfig::new(
        BinningStrategy::EqualWidth,
        2,
        ColumnSelector::ByIndex(vec![0]), // Only discretize the first column
    );

    let result = discretizer.process(tensor, &config).unwrap();
    let expected = CausalTensor::new(
        vec![0.0, 10.0, 0.0, 9.0, 1.0, 8.0, 1.0, 7.0, 1.0, 6.0],
        vec![5, 2],
    )
    .unwrap();
    assert_eq!(result.as_slice(), expected.as_slice());
}

#[test]
fn test_data_discretizer_equal_width_strategy_single_value_column() {
    let discretizer = DataDiscretizer;
    let tensor = CausalTensor::new(vec![5.0, 5.0, 5.0, 5.0], vec![2, 2]).unwrap();
    let config = PreprocessConfig::new(BinningStrategy::EqualWidth, 2, ColumnSelector::All);

    let result = discretizer.process(tensor, &config).unwrap();
    let expected = CausalTensor::new(vec![0.0, 0.0, 0.0, 0.0], vec![2, 2]).unwrap();
    assert_eq!(result.as_slice(), expected.as_slice());
}

#[test]
fn test_data_discretizer_equal_width_strategy_less_than_two_bins() {
    let discretizer = DataDiscretizer;
    let tensor = CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();
    let config = PreprocessConfig::new(BinningStrategy::EqualWidth, 1, ColumnSelector::All); // Less than 2 bins

    let result = discretizer.process(tensor, &config);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        PreprocessError::ConfigError("Number of bins must be at least 2".to_string())
    );
}

#[test]
fn test_data_discretizer_equal_frequency_strategy_all_columns() {
    let discretizer = DataDiscretizer;
    let tensor = CausalTensor::new(
        vec![1.0, 10.0, 2.0, 9.0, 3.0, 8.0, 4.0, 7.0, 5.0, 6.0],
        vec![5, 2],
    )
    .unwrap();
    let config = PreprocessConfig::new(BinningStrategy::EqualFrequency, 2, ColumnSelector::All);

    let result = discretizer.process(tensor, &config).unwrap();
    // Expected values for equal frequency binning (2 bins)
    // Column 0: [1.0, 2.0, 3.0, 4.0, 5.0] -> [0.0, 0.0, 1.0, 1.0, 1.0] (approx)
    // Column 1: [10.0, 9.0, 8.0, 7.0, 6.0] -> [1.0, 1.0, 0.0, 0.0, 0.0] (approx)
    let expected = CausalTensor::new(
        vec![0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0],
        vec![5, 2],
    )
    .unwrap();
    assert_eq!(result.as_slice(), expected.as_slice());
}

#[test]
fn test_data_discretizer_equal_frequency_strategy_by_index() {
    let discretizer = DataDiscretizer;
    let tensor = CausalTensor::new(
        vec![1.0, 10.0, 2.0, 9.0, 3.0, 8.0, 4.0, 7.0, 5.0, 6.0],
        vec![5, 2],
    )
    .unwrap();
    let config = PreprocessConfig::new(
        BinningStrategy::EqualFrequency,
        2,
        ColumnSelector::ByIndex(vec![1]), // Only discretize the second column
    );

    let result = discretizer.process(tensor, &config).unwrap();
    // Column 0 remains unchanged: [1.0, 2.0, 3.0, 4.0, 5.0]
    // Column 1: [10.0, 9.0, 8.0, 7.0, 6.0] -> [1.0, 1.0, 0.0, 0.0, 0.0] (approx)
    let expected = CausalTensor::new(
        vec![1.0, 1.0, 2.0, 1.0, 3.0, 0.0, 4.0, 0.0, 5.0, 0.0],
        vec![5, 2],
    )
    .unwrap();
    assert_eq!(result.as_slice(), expected.as_slice());
}

#[test]
fn test_data_discretizer_equal_frequency_strategy_empty_data() {
    let discretizer = DataDiscretizer;
    let tensor = CausalTensor::new(Vec::<f64>::new(), vec![0, 0]).unwrap();
    let config = PreprocessConfig::new(BinningStrategy::EqualFrequency, 2, ColumnSelector::All);

    let result = discretizer.process(tensor, &config).unwrap();
    let expected = CausalTensor::new(Vec::<f64>::new(), vec![0, 0]).unwrap();
    assert_eq!(result.as_slice(), expected.as_slice());
}

#[test]
fn test_data_discretizer_equal_frequency_strategy_less_than_two_bins() {
    let discretizer = DataDiscretizer;
    let tensor = CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();
    let config = PreprocessConfig::new(BinningStrategy::EqualFrequency, 1, ColumnSelector::All); // Less than 2 bins

    let result = discretizer.process(tensor, &config);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        PreprocessError::ConfigError("Number of bins must be at least 2".to_string())
    );
}
