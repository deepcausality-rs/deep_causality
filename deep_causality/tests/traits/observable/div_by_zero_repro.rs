/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::{Identifiable, NumericalValue, Observable, ObservableReasoning};

#[derive(Debug, Clone)]
struct TestObservable {
    id: u64,
    observation: f64,
    effect: f64,
}

impl Identifiable for TestObservable {
    fn id(&self) -> u64 {
        self.id
    }
}

impl Observable for TestObservable {
    fn observation(&self) -> NumericalValue {
        self.observation
    }
    fn observed_effect(&self) -> NumericalValue {
        self.effect
    }
}

struct TestCollection {
    items: Vec<TestObservable>,
}

impl ObservableReasoning<TestObservable> for TestCollection {
    fn len(&self) -> usize {
        self.items.len()
    }
    fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
    fn get_all_items(&self) -> Vec<&TestObservable> {
        self.items.iter().collect()
    }
}

#[test]
fn test_observable_div_by_zero() {
    let empty = TestCollection { items: vec![] };
    let result = empty.percent_observation(0.5, 1.0);

    // Fix: returns 0.0 on empty collection
    assert_eq!(result, 0.0, "Should return 0.0 on empty collection");
    assert!(!result.is_nan());
}
