/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::*;
use std::collections::HashMap;
use std::sync::Arc;
use ultragraph::UltraGraph;

#[test]
fn test_default() {
    let evidence: Evidence = Default::default();
    assert!(matches!(evidence, Evidence::None));
}

#[test]
fn test_new_map() {
    let evidence = Evidence::new_map();
    if let Evidence::Map(map) = evidence {
        assert!(map.is_empty());
    } else {
        panic!("Expected Evidence::Map, but got a different variant.");
    }
}

#[test]
fn test_insert_success() {
    let mut evidence = Evidence::new_map();
    evidence.insert(1, Evidence::Numerical(42.0));

    if let Evidence::Map(map) = evidence {
        assert_eq!(map.len(), 1);
        let val = map.get(&1).expect("Failed to get inserted value");
        assert!(matches!(val, Evidence::Numerical(num) if *num == 42.0));
    } else {
        panic!("Expected Evidence::Map after insertion.");
    }
}

#[test]
#[should_panic(expected = "Cannot insert into Evidence that is not a Map variant")]
fn test_insert_panic_on_non_map() {
    let mut evidence = Evidence::Numerical(10.0);
    // This should panic because `evidence` is not an `Evidence::Map`.
    evidence.insert(1, Evidence::Numerical(42.0));
}

#[test]
fn test_get_numerical_from_map() {
    let mut evidence = Evidence::new_map();
    evidence.insert(1, Evidence::Numerical(123.45));
    evidence.insert(2, Evidence::Deterministic(true));

    // Success case
    let val = evidence.get_numerical_from_map(1).unwrap();
    assert_eq!(val, 123.45);

    // Error: Wrong type
    let err = evidence.get_numerical_from_map(2).unwrap_err();
    assert_eq!(
        err.to_string(),
        "CausalityError: Evidence for key '2' is not of type Numerical"
    );

    // Error: Key not found
    let err = evidence.get_numerical_from_map(99).unwrap_err();
    assert_eq!(
        err.to_string(),
        "CausalityError: No evidence found for key '99'"
    );

    // Error: Not a map
    let non_map_evidence = Evidence::None;
    let err = non_map_evidence.get_numerical_from_map(1).unwrap_err();
    assert_eq!(
        err.to_string(),
        "CausalityError: Cannot get value by key from Evidence that is not a Map variant"
    );
}

#[test]
fn test_get_deterministic_from_map() {
    let mut evidence = Evidence::new_map();
    evidence.insert(1, Evidence::Deterministic(true));
    evidence.insert(2, Evidence::Numerical(123.45));

    // Success case
    let val = evidence.get_deterministic_from_map(1).unwrap();
    assert!(val);

    // Error: Wrong type
    let err = evidence.get_deterministic_from_map(2).unwrap_err();
    assert_eq!(
        err.to_string(),
        "CausalityError: Evidence for key '2' is not of type Deterministic"
    );

    // Error: Key not found
    let err = evidence.get_deterministic_from_map(99).unwrap_err();
    assert_eq!(
        err.to_string(),
        "CausalityError: No evidence found for key '99'"
    );

    // Error: Not a map
    let non_map_evidence = Evidence::None;
    let err = non_map_evidence.get_deterministic_from_map(1).unwrap_err();
    assert_eq!(
        err.to_string(),
        "CausalityError: Cannot get value by key from Evidence that is not a Map variant"
    );
}

#[test]
fn test_debug_format() {
    assert_eq!(format!("{:?}", Evidence::None), "Evidence::None");
    assert_eq!(
        format!("{:?}", Evidence::Deterministic(true)),
        "Evidence::Deterministic(true)"
    );
    assert_eq!(
        format!("{:?}", Evidence::Numerical(42.0)),
        "Evidence::Numerical(42.0)"
    );
    assert_eq!(
        format!("{:?}", Evidence::Probability(0.5)),
        "Evidence::Probability(0.5)"
    );
    assert_eq!(
        format!("{:?}", Evidence::ContextualLink(1, 2)),
        "Evidence::ContextualLink(1, 2)"
    );

    let mut map = HashMap::new();
    map.insert(1, Evidence::Numerical(1.0));
    assert_eq!(
        format!("{:?}", Evidence::Map(map)),
        "Evidence::Map({1: Evidence::Numerical(1.0)})"
    );

    let graph = Arc::new(UltraGraph::<Evidence>::new());
    assert_eq!(
        format!("{:?}", Evidence::Graph(graph)),
        "Evidence::Graph(nodes: 0, edges: 0)"
    );
}

#[test]
fn test_display_format() {
    // The Display impl currently delegates to Debug, so the outputs should be identical.
    let evidence = Evidence::Numerical(99.9);
    let display_str = format!("{evidence}");
    let debug_str = format!("{evidence:?}");
    assert_eq!(display_str, debug_str);
}
