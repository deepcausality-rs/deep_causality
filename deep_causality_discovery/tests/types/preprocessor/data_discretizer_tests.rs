/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
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
    // The variant is the contract; the wording now comes from `deep_causality_stats`, so pinning
    // the exact string would assert which crate phrased the refusal rather than what it refuses.
    assert!(
        matches!(result, Err(PreprocessError::ConfigError(_))),
        "a bin count below two is a configuration error, got {result:?}"
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
    // The variant is the contract; the wording now comes from `deep_causality_stats`, so pinning
    // the exact string would assert which crate phrased the refusal rather than what it refuses.
    assert!(
        matches!(result, Err(PreprocessError::ConfigError(_))),
        "a bin count below two is a configuration error, got {result:?}"
    );
}

#[test]
fn test_data_discretizer_equal_width_more_bins_than_observations() {
    let discretizer = DataDiscretizer;
    let tensor = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3, 1]).unwrap();
    let config = PreprocessConfig::new(BinningStrategy::EqualWidth, 5, ColumnSelector::All);

    // The refusal is shared with equal frequency, and it reaches the equal-width path too. Only
    // the equal-frequency side of it carried a test, so the equal-width side read as untested
    // rather than as decided.
    let result = discretizer.process(tensor, &config);
    assert!(
        matches!(result, Err(PreprocessError::ConfigError(_))),
        "more bins than observations is a configuration error, got {result:?}"
    );
}

#[test]
fn test_data_discretizer_equal_width_bin_edge_follows_the_scaled_quotient() {
    let discretizer = DataDiscretizer;
    let tensor = CausalTensor::new(vec![0.0, 0.7, 2.1], vec![3, 1]).unwrap();
    let config = PreprocessConfig::new(BinningStrategy::EqualWidth, 3, ColumnSelector::All);

    // `0.7` sits on the first edge of three bins over `[0, 2.1]`, and the answer is decided by
    // rounding, not by the partition. Hand-evaluated from IEEE 754 round-to-nearest-even, not
    // from the shipped code:
    //
    //   fl(0.7) = 3152519739159347 / 2^52,  fl(2.1) = 4728779608739021 / 2^51
    //   fl(fl(0.7) / fl(2.1)) = 0x3FD5555555555555 = 6004799503160661 / 2^54
    //   that × 3 = 18014398509481983 / 2^54 = 1 − 2⁻⁵⁴ exactly — the midpoint below 1, whose
    //   even neighbour is 1.0, so the multiplication rounds *up* to 1.0 and the floor is 1.
    //
    // In exact arithmetic on the same two doubles the quotient times three is strictly below 1,
    // so bin 0 is the mathematically correct answer and this is a rounding artefact of the
    // scaled form. The replaced implementation divided by a pre-rounded `width = fl(2.1/3) =
    // 0x3FE6666666666667` instead, giving `fl(0.7/width) = 0.9999999999999999` and bin 0 — which
    // is why the two forms are not value-identical.
    let result = discretizer.process(tensor, &config).unwrap();
    assert_eq!(result.as_slice(), &[0.0, 1.0, 2.0]);
}

#[test]
fn test_data_discretizer_equal_width_range_below_epsilon_is_not_a_constant_column() {
    let discretizer = DataDiscretizer;
    let tensor = CausalTensor::new(vec![0.0, 1e-17], vec![2, 1]).unwrap();
    let config = PreprocessConfig::new(BinningStrategy::EqualWidth, 2, ColumnSelector::All);

    // The column is not constant, so the two observations occupy different bins: the minimum
    // opens bin 0 and the maximum closes the last one, which holds for any strictly increasing
    // pair regardless of how small the range is. The replaced implementation collapsed a range
    // below `f64::EPSILON` to a single bin, so it answered `[0, 0]` here.
    let result = discretizer.process(tensor, &config).unwrap();
    assert_eq!(result.as_slice(), &[0.0, 1.0]);
}
