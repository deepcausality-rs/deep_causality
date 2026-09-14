/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Laws and corner cases for [`CochainWitness`].
//!
//! Corner rows are enumerated in
//! `openspec/changes/add-hkt-traversable-cochain/corner-cases.md`; the ids (K1..K5) in the test
//! names below refer to that table.

use deep_causality_haft::{Foldable, Functor};
use deep_causality_topology::{Cochain, CochainWitness};

/// K3 — the row that matters most: the degree differs from the value count, so a witness that
/// derived the degree from the length would be caught here and nowhere else.
#[test]
fn test_cochain_fmap_degree_differs_from_len() {
    let c = Cochain::new(vec![10, 20, 30, 40, 50], 2);
    let mapped = CochainWitness::fmap(c, |v| v + 1);
    assert_eq!(mapped.degree(), 2, "degree is carried, not derived");
    assert_eq!(mapped.len(), 5, "value count is unchanged");
    assert_eq!(mapped.values(), &[11, 21, 31, 41, 51]);
}

/// K2 — degree 0 survives. It is the degree a defect most plausibly substitutes for a missing one.
#[test]
fn test_cochain_fmap_degree_zero_survives() {
    let c = Cochain::new(vec![1, 2, 3], 0);
    let mapped = CochainWitness::fmap(c, |v| v * 2);
    assert_eq!(mapped.degree(), 0);
    assert_eq!(mapped.values(), &[2, 4, 6]);
}

/// The degree survives a map that changes the element type.
#[test]
fn test_cochain_fmap_changes_element_type() {
    let c = Cochain::new(vec![1i32, 2, 3], 3);
    let mapped: Cochain<String> = CochainWitness::fmap(c, |v| format!("v{v}"));
    assert_eq!(mapped.degree(), 3);
    assert_eq!(
        mapped.values(),
        &["v1".to_string(), "v2".to_string(), "v3".to_string()]
    );
}

/// K4 — index order. Each output position holds the function applied to the input at that
/// position, so a reversing or rotating `fmap` fails.
#[test]
fn test_cochain_fmap_index_order() {
    let c = Cochain::new(vec![1, 2, 3, 4], 1);
    let mapped = CochainWitness::fmap(c, |v| v * 10);
    assert_eq!(mapped.values(), &[10, 20, 30, 40]);
}

/// An empty cochain maps to an empty cochain of the same degree.
#[test]
fn test_cochain_fmap_empty_preserves_degree() {
    let c: Cochain<i32> = Cochain::new(Vec::new(), 4);
    let mapped = CochainWitness::fmap(c, |v| v + 1);
    assert_eq!(mapped.degree(), 4);
    assert!(mapped.is_empty());
}

/// K4, K5 — a non-commutative fold reveals the traversal order. A commutative operation such as
/// addition could not distinguish a reversed fold from a correct one.
#[test]
fn test_cochain_fold_index_order() {
    let c = Cochain::new(vec![1, 2, 3], 1);
    // Left-to-right string building: order-sensitive by construction.
    let acc = CochainWitness::fold(c, String::new(), |mut s, v| {
        s.push_str(&v.to_string());
        s
    });
    assert_eq!(acc, "123");
}

/// K1 — an empty cochain folds to the initial accumulator and never calls the folding function.
#[test]
fn test_cochain_fold_empty_returns_init() {
    let c: Cochain<i32> = Cochain::new(Vec::new(), 2);
    let mut calls = 0;
    let acc = CochainWitness::fold(c, 99, |a, v| {
        calls += 1;
        a + v
    });
    assert_eq!(acc, 99, "the initial accumulator is returned unchanged");
    assert_eq!(calls, 0, "the folding function is never called");
}

/// The fold visits every value exactly once.
#[test]
fn test_cochain_fold_visits_every_value_once() {
    let c = Cochain::new(vec![5, 5, 5, 5], 1);
    let n = CochainWitness::fold(c, 0, |a, _| a + 1);
    assert_eq!(n, 4);
}

/// Functor identity: `fmap(c, id) == c`, degree included.
#[test]
fn test_cochain_functor_identity_law() {
    for (values, degree) in [(vec![1, 2, 3], 2usize), (Vec::new(), 0)] {
        let c = Cochain::new(values, degree);
        let mapped = CochainWitness::fmap(c.clone(), |v| v);
        assert_eq!(mapped, c);
    }
}

/// Functor composition: `fmap(fmap(c, f), g) == fmap(c, g . f)`.
#[test]
fn test_cochain_functor_composition_law() {
    let c = Cochain::new(vec![1, 2, 3, 4], 3);
    let f = |v: i32| v + 7;
    let g = |v: i32| v * 3;

    let stepwise = CochainWitness::fmap(CochainWitness::fmap(c.clone(), f), g);
    let composed = CochainWitness::fmap(c, move |v| g(f(v)));
    assert_eq!(stepwise, composed);
}

/// `fold` agrees with a fold over the values obtained through `fmap`.
#[test]
fn test_cochain_fold_fmap_consistency() {
    let c = Cochain::new(vec![1, 2, 3], 2);
    let direct = CochainWitness::fold(c.clone(), 0, |a, v| a + v * 2);
    let via_fmap = CochainWitness::fold(CochainWitness::fmap(c, |v| v * 2), 0, |a, v| a + v);
    assert_eq!(direct, via_fmap);
}
