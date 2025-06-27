/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::prelude::CausaloidType;

#[test]
fn test_display_trait() {
    assert_eq!(CausaloidType::Singleton.to_string(), "Singleton");
    assert_eq!(CausaloidType::Collection.to_string(), "Collection");
    assert_eq!(CausaloidType::Graph.to_string(), "Graph");
}

#[test]
fn test_partial_eq() {
    assert_eq!(CausaloidType::Singleton, CausaloidType::Singleton);
    assert_ne!(CausaloidType::Singleton, CausaloidType::Graph);
}

#[test]
fn test_partial_ord() {
    assert!(CausaloidType::Singleton < CausaloidType::Collection);
    assert!(CausaloidType::Collection < CausaloidType::Graph);
    assert!(CausaloidType::Graph > CausaloidType::Singleton);
}

#[test]
fn test_copy() {
    let original = CausaloidType::Graph;
    let copied = original;
    let cloned = original;

    assert_eq!(original, copied);
    assert_eq!(original, cloned);
}

#[test]
fn test_debug_output() {
    let value = CausaloidType::Collection;
    let debug_str = format!("{value:?}");
    assert_eq!(debug_str, "Collection");
}
