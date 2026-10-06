/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_context::*;

#[test]
fn test_creation_with_label() {
    let e = CausalSetSpacetime::new(1, Some("Init".to_string()));
    assert_eq!(e.id(), 1);
    assert_eq!(e.label(), Some("Init"));
    assert!(e.predecessors().is_empty());
}

#[test]
fn test_creation_without_label() {
    let e = CausalSetSpacetime::new(2, None);
    assert_eq!(e.id(), 2);
    assert!(e.label().is_none());
    assert!(e.predecessors().is_empty());
}

#[test]
fn test_add_predecessor() {
    let mut e = CausalSetSpacetime::new(3, Some("C".into()));
    e.add_predecessor(1);
    e.add_predecessor(2);

    assert!(e.predecessors().contains(&1));
    assert!(e.predecessors().contains(&2));
    assert_eq!(e.predecessor_count(), 2);
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
fn test_predecessor_count() {
    let mut e = CausalSetSpacetime::new(5, Some("CountTest".into()));
    assert_eq!(e.predecessor_count(), 0);

    e.add_predecessor(100);
    assert_eq!(e.predecessor_count(), 1);

    e.add_predecessor(200);
    assert_eq!(e.predecessor_count(), 2);
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

    let preds: Vec<_> = e.predecessors().iter().cloned().collect();
    assert_eq!(preds, vec![13, 42, 99]); // BTreeSet guarantees ordering
}

#[test]
fn test_add_predecessor_reports_whether_it_added() {
    let mut e = CausalSetSpacetime::new(8, None);
    assert!(e.add_predecessor(1));
    assert!(!e.add_predecessor(1)); // already recorded
    assert_eq!(e.predecessor_count(), 1);
}

#[test]
fn test_an_element_does_not_precede_itself() {
    // Irreflexivity (Sorkin 2003, p. 5): x ≺ x never holds, so the element's own id is refused.
    let mut e = CausalSetSpacetime::new(9, None);
    assert!(!e.add_predecessor(9));
    assert!(!e.is_after(9));
    assert_eq!(e.predecessor_count(), 0);
}

#[test]
fn test_predecessors_is_the_past_without_the_element_itself() {
    // For any sequence of insertions, `predecessors()` equals the inserted ids minus the element's
    // own id, and every other accessor agrees with it. The candidates repeat the own id and other
    // ids, on both sides of it, at the ends of the id range.
    for own in [0, 9, u64::MAX] {
        let mut e = CausalSetSpacetime::new(own, Some("Past".into()));
        let mut expected = std::collections::BTreeSet::new();
        for candidate in [own, 4, own, u64::MAX, 4, 0, 17, own, 1] {
            let added = candidate != own && expected.insert(candidate);
            assert_eq!(e.add_predecessor(candidate), added);
            assert_eq!(e.predecessors(), &expected);
            assert!(!e.predecessors().contains(&own));
            assert!(!e.is_after(own));
            assert_eq!(e.is_after(candidate), expected.contains(&candidate));
            assert_eq!(e.predecessor_count(), e.predecessors().len());
            assert_eq!(e.id(), own);
            assert_eq!(e.label(), Some("Past"));
        }
        assert!(!expected.is_empty());
    }
}
