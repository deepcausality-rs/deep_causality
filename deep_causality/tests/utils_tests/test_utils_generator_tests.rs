/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::prelude::{Datable, Identifiable};
use deep_causality::utils_test::test_utils_generator::MockData;

#[test]
fn test_mock_data_id() {
    let mock = MockData { id: 1, data: 10 };
    assert_eq!(mock.id(), 1);
}

#[test]
fn test_mock_data_get_data() {
    let mock = MockData { id: 1, data: 10 };
    assert_eq!(mock.get_data(), 10);
}

#[test]
fn test_mock_data_set_data() {
    let mut mock = MockData { id: 1, data: 10 };
    mock.set_data(20);
    assert_eq!(mock.get_data(), 20);
}

#[test]
fn test_mock_data_default() {
    let mock = MockData::default();
    assert_eq!(mock.id(), 0);
    assert_eq!(mock.get_data(), 0);
}
