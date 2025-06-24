// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use deep_causality::prelude::*;

#[test]
fn test_creation_with_label() {
    let e = CausalSetSpacetime::new(1, Some("Init".to_string()));
    assert_eq!(e.id, 1);
    assert_eq!(e.label.as_deref(), Some("Init"));
    assert!(e.predecessors.is_empty());
}

#[test]
fn test_creation_without_label() {
    let e = CausalSetSpacetime::new(2, None);
    assert_eq!(e.id, 2);
    assert!(e.label.is_none());
    assert!(e.predecessors.is_empty());
}

#[test]
fn test_add_predecessor() {
    let mut e = CausalSetSpacetime::new(3, Some("C".into()));
    e.add_predecessor(1);
    e.add_predecessor(2);

    assert!(e.predecessors.contains(&1));
    assert!(e.predecessors.contains(&2));
    assert_eq!(e.causal_depth(), 2);
}

#[test]
fn test_is_after() {
    let mut e = CausalSetSpacetime::new(4, Some("D".into()));
    e.add_predecessor(10);
    e.add_predecessor(20);

    assert!(e.is_after(10));
    assert!(e.is_after(20));
    assert!(!e.is_after(30));
}

#[test]
fn test_causal_depth() {
    let mut e = CausalSetSpacetime::new(5, Some("DepthTest".into()));
    assert_eq!(e.causal_depth(), 0);

    e.add_predecessor(100);
    assert_eq!(e.causal_depth(), 1);

    e.add_predecessor(200);
    assert_eq!(e.causal_depth(), 2);
}

#[test]
fn test_display_trait_output() {
    let mut e = CausalSetSpacetime::new(6, Some("Labelled".into()));
    e.add_predecessor(7);
    let output = format!("{e}");

    assert!(output.contains("CausalSetSpacetime"));
    assert!(output.contains("id: 6"));
    assert!(output.contains("Labelled"));
    assert!(output.contains("7"));
}

#[test]
fn test_ordered_predecessors() {
    let mut e = CausalSetSpacetime::new(7, Some("Ordered".into()));
    e.add_predecessor(42);
    e.add_predecessor(13);
    e.add_predecessor(99);

    let preds: Vec<_> = e.predecessors.iter().cloned().collect();
    assert_eq!(preds, vec![13, 42, 99]); // BTreeSet guarantees ordering
}
