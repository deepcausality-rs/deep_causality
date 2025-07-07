/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality::*;

#[test]
fn test_creation_with_label() {
    let c = ConformalSpacetime::new(1, Some("i+".into()));
    assert_eq!(c.id, 1);
    assert_eq!(c.label.as_deref(), Some("i+"));
    assert!(c.causal_links.is_empty());
}

#[test]
fn test_creation_without_label() {
    let c = ConformalSpacetime::new(2, None);
    assert_eq!(c.id, 2);
    assert!(c.label.is_none());
    assert!(c.causal_links.is_empty());
}

#[test]
fn test_link_to_and_can_affect() {
    let mut n1 = ConformalSpacetime::new(3, Some("scri".into()));
    let target_id = 99;

    assert!(!n1.can_affect(target_id));

    n1.link_to(target_id);
    assert!(n1.can_affect(target_id));
}

#[test]
fn test_multiple_links() {
    let mut c = ConformalSpacetime::new(4, Some("H".into()));
    c.link_to(10);
    c.link_to(20);
    c.link_to(30);

    assert!(c.can_affect(10));
    assert!(c.can_affect(20));
    assert!(c.can_affect(30));
    assert_eq!(c.fanout(), 3);
}

#[test]
fn test_ordered_causal_links() {
    let mut c = ConformalSpacetime::new(5, Some("cone".into()));
    c.link_to(200);
    c.link_to(100);
    c.link_to(300);

    let ids: Vec<_> = c.causal_links.iter().cloned().collect();
    assert_eq!(ids, vec![100, 200, 300]); // BTreeSet ensures ordering
}

#[test]
fn test_display_trait() {
    let c = ConformalSpacetime::new(6, Some("boundary".into()));
    let output = format!("{c}");

    assert!(output.contains("ConformalSpacetime"));
    assert!(output.contains("id: 6"));
    assert!(output.contains("boundary"));
}
