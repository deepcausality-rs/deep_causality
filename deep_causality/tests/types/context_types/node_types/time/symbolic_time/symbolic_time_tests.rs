// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use deep_causality::prelude::{Identifiable, SymbolicTime, SymbolicTimeUnit, Temporal, TimeScale};

#[test]
fn test_construction_before() {
    let t = SymbolicTime::new(1, SymbolicTimeUnit::Before("SensorReading".into(), -10));
    assert_eq!(t.id(), 1);
    assert_eq!(t.time_unit(), -10);
    assert_eq!(t.time_scale(), TimeScale::Symbolic);
    assert_eq!(
        t.time(),
        &SymbolicTimeUnit::Before("SensorReading".to_string(), -10)
    );
}

#[test]
fn test_construction_named() {
    let t = SymbolicTime::new(2, SymbolicTimeUnit::Named("Init".into(), 0));
    assert_eq!(t.time_unit(), 0);
    assert_eq!(t.time(), &SymbolicTimeUnit::Named("Init".to_string(), 0));
}

#[test]
fn test_construction_after() {
    let t = SymbolicTime::new(3, SymbolicTimeUnit::After("Decision".into(), 42));
    assert_eq!(t.id(), 3);
    assert_eq!(t.time_unit(), 42);
    assert_eq!(
        t.time(),
        &SymbolicTimeUnit::After("Decision".to_string(), 42)
    );
}

#[test]
fn test_construction_simultaneous() {
    let labels = vec!["A".into(), "B".into(), "C".into()];
    let t = SymbolicTime::new(4, SymbolicTimeUnit::Simultaneous(labels.clone(), 100));
    assert_eq!(t.id(), 4);
    assert_eq!(t.time_unit(), 100);
    assert_eq!(t.time(), &SymbolicTimeUnit::Simultaneous(labels, 100));
}

#[test]
fn test_display_output() {
    let cases = vec![
        SymbolicTime::new(1, SymbolicTimeUnit::Before("X".into(), -5)),
        SymbolicTime::new(2, SymbolicTimeUnit::Named("Init".into(), 0)),
        SymbolicTime::new(3, SymbolicTimeUnit::After("End".into(), 10)),
        SymbolicTime::new(
            4,
            SymbolicTimeUnit::Simultaneous(vec!["A".into(), "B".into()], 20),
        ),
    ];

    for t in cases {
        let output = format!("{}", t);
        assert!(output.contains(&format!("#{}", t.id())));
        assert!(output.contains(&format!("@ {}", t.time_unit())));
    }
}

#[test]
fn test_equality_and_hash() {
    use std::collections::HashSet;

    let t1 = SymbolicTime::new(1, SymbolicTimeUnit::Named("Event".into(), 123));
    let t2 = t1.clone();
    let t3 = SymbolicTime::new(2, SymbolicTimeUnit::Named("Event".into(), 123));
    assert_eq!(t1, t2);
    assert_ne!(t1, t3);

    let mut set = HashSet::new();
    set.insert(t1);
    assert!(set.contains(&t2));
    assert!(!set.contains(&t3));
}
