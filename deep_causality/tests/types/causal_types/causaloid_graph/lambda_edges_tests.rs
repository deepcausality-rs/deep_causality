/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Storage-and-lookup behaviour of [`LambdaEdges`] — the per-edge Λ decoration store keyed by
//! intrinsic `(source, target)` edge identity. An undecorated edge is the identity Λ; a decorated
//! edge applies its stored `fn` to the value flowing along it (before the `∇` merge at a join).

use deep_causality::LambdaEdges;

#[test]
fn test_default_is_empty_and_identity() {
    // `default()` is the empty store: no edge is decorated, so every edge is the identity Λ.
    let edges = LambdaEdges::<i64>::default();
    assert_eq!(edges.len(), 0);
    assert!(edges.is_empty());
    // An undecorated edge passes its value through unchanged.
    assert_eq!(edges.apply(0, 1, 5), 5);
}

#[test]
fn test_with_lambda_decorates_one_edge() {
    // Builder form: decorate edge (0, 1) with `+10`.
    let edges = LambdaEdges::<i64>::new().with_lambda(0, 1, |v| v + 10);
    assert!(!edges.is_empty());
    assert_eq!(edges.len(), 1);

    // `get` lends back the stored transform; applied to 5 it maps to 15.
    let lambda = edges.get(0, 1).expect("edge (0, 1) is decorated");
    assert_eq!(lambda(5), 15);

    // `apply` on the decorated edge runs the transform; on any other edge it is the identity.
    assert_eq!(edges.apply(0, 1, 5), 15);
    assert_eq!(edges.apply(9, 9, 5), 5);
    assert!(edges.get(9, 9).is_none());
}

#[test]
fn test_insert_returns_previous_lambda_on_overwrite() {
    let mut edges = LambdaEdges::<i64>::new();

    // First insert on a fresh slot returns `None`.
    let first = edges.insert(0, 1, |v| v + 10);
    assert!(first.is_none());
    assert_eq!(edges.apply(0, 1, 5), 15);

    // Overwriting the same (source, target) returns the previously stored Λ ...
    let previous = edges.insert(0, 1, |v| v * 2);
    let previous = previous.expect("overwrite returns the previous Λ");
    assert_eq!(previous(5), 15); // the old transform was `+10`.

    // ... and the store now applies the new Λ, without growing (overwrite, not add).
    assert_eq!(edges.apply(0, 1, 5), 10);
    assert_eq!(edges.len(), 1);
}

#[test]
fn test_debug_names_type_and_lists_decorated_keys() {
    let edges = LambdaEdges::<i64>::new().with_lambda(0, 1, |v| v + 10);
    let rendered = format!("{edges:?}");
    assert!(rendered.contains("LambdaEdges"));
    assert!(rendered.contains("decorated_edges"));
    // The intrinsic edge key is listed.
    assert!(rendered.contains("(0, 1)"));
}
