/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::{ParentEffects, PropagatingEffect};
use std::collections::BTreeMap;

fn make(entries: &[(usize, f64)]) -> ParentEffects<f64> {
    let mut m: BTreeMap<usize, PropagatingEffect<f64>> = BTreeMap::new();
    for (k, v) in entries {
        m.insert(*k, PropagatingEffect::from_value(*v));
    }
    ParentEffects::new(m)
}

#[test]
fn test_get_returns_fired_parent() {
    let pe = make(&[(1, 10.0), (4, 40.0)]);
    assert_eq!(pe.get(1).and_then(|e| e.value()), Some(&10.0));
    assert_eq!(pe.get(4).and_then(|e| e.value()), Some(&40.0));
    assert!(pe.get(2).is_none());
}

#[test]
fn test_iter_is_ascending_key_order() {
    // Insert out of order; iteration must be ascending by parent index.
    let pe = make(&[(4, 40.0), (1, 10.0), (2, 20.0)]);
    let keys: Vec<usize> = pe.iter().map(|(k, _)| k).collect();
    assert_eq!(keys, vec![1, 2, 4]);
}

#[test]
fn test_parent_indices() {
    let pe = make(&[(3, 3.0), (1, 1.0)]);
    let idx: Vec<usize> = pe.parent_indices().collect();
    assert_eq!(idx, vec![1, 3]);
}

#[test]
fn test_len_and_is_empty() {
    let pe = make(&[(1, 1.0), (2, 2.0)]);
    assert_eq!(pe.len(), 2);
    assert!(!pe.is_empty());

    let empty: ParentEffects<f64> = ParentEffects::new(BTreeMap::new());
    assert_eq!(empty.len(), 0);
    assert!(empty.is_empty());
}
