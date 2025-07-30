/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::AggregateLogic;

#[test]
fn test_aggregate_logic_all() {
    let logic = AggregateLogic::All;
    assert_eq!(format!("{logic}"), "All");
}

#[test]
fn test_aggregate_logic_any() {
    let logic = AggregateLogic::Any;
    assert_eq!(format!("{logic}"), "Any");
}

#[test]
fn test_aggregate_logic_none() {
    let logic = AggregateLogic::None;
    assert_eq!(format!("{logic}"), "None");
}

#[test]
fn test_aggregate_logic_some() {
    let logic = AggregateLogic::Some(5);
    assert_eq!(format!("{logic}"), "Some(5)");
}
