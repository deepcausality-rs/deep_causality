/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_discovery::{BinningStrategy, ColumnSelector, PreprocessConfig};

#[test]
fn test_new_and_getters() {
    let config = PreprocessConfig::new(BinningStrategy::EqualWidth, 10, ColumnSelector::All);
    assert!(matches!(config.strategy(), BinningStrategy::EqualWidth));
    assert_eq!(config.num_bins(), 10);
    assert!(matches!(config.columns(), ColumnSelector::All));
}

#[test]
fn test_display() {
    let config_all = PreprocessConfig::new(BinningStrategy::EqualWidth, 10, ColumnSelector::All);
    assert_eq!(
        format!("{}", config_all),
        "PreprocessConfig(strategy: EqualWidth, num_bins: 10, columns: All)"
    );

    let config_by_index = PreprocessConfig::new(
        BinningStrategy::EqualFrequency,
        5,
        ColumnSelector::ByIndex(vec![0, 2]),
    );
    assert_eq!(
        format!("{}", config_by_index),
        "PreprocessConfig(strategy: EqualFrequency, num_bins: 5, columns: ByIndex([0, 2]))"
    );

    let config_by_name = PreprocessConfig::new(
        BinningStrategy::EqualWidth,
        8,
        ColumnSelector::ByName(vec!["a".into(), "c".into()]),
    );
    assert_eq!(
        format!("{}", config_by_name),
        "PreprocessConfig(strategy: EqualWidth, num_bins: 8, columns: ByName([\"a\", \"c\"]))"
    );
}

#[test]
fn test_clone() {
    let config1 = PreprocessConfig::new(
        BinningStrategy::EqualWidth,
        10,
        ColumnSelector::ByIndex(vec![1]),
    );
    let config2 = config1.clone();

    assert!(matches!(config2.strategy(), BinningStrategy::EqualWidth));
    assert_eq!(config1.num_bins(), config2.num_bins());
    if let (ColumnSelector::ByIndex(v1), ColumnSelector::ByIndex(v2)) =
        (config1.columns(), config2.columns())
    {
        assert_eq!(v1, v2);
    } else {
        panic!("Clone failed for columns");
    }
}

#[test]
fn test_debug() {
    let config = PreprocessConfig::new(
        BinningStrategy::EqualFrequency,
        5,
        ColumnSelector::ByIndex(vec![0, 2]),
    );
    let debug = format!("{:?}", config);
    assert!(debug.starts_with("PreprocessConfig"));
    assert!(debug.contains("strategy: EqualFrequency"));
    assert!(debug.contains("num_bins: 5"));
    assert!(debug.contains("columns: ByIndex([0, 2])"));
}

#[test]
fn test_enum_clones() {
    let bs1 = BinningStrategy::EqualWidth;
    let bs2 = bs1.clone();
    assert!(matches!(bs2, BinningStrategy::EqualWidth));

    let cs1 = ColumnSelector::ByName(vec!["a".into()]);
    let cs2 = cs1.clone();
    assert!(matches!(cs2, ColumnSelector::ByName(_)));
    if let ColumnSelector::ByName(v) = cs2 {
        assert_eq!(v, vec!["a".to_string()]);
    }
}
