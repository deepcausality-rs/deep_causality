/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `Data<T>` requires `Clone` of its payload, not `Copy`, so a context node can carry a sequence.
//!
//! `Copy` is asked for only by the `Adjustable` impl, where `ArrayGrid`'s fixed-size array backing
//! genuinely needs it. Nothing about being a context node does.

use deep_causality_context::utils_test::test_utils::get_context;
use deep_causality_context::{
    Context, Contextoid, ContextoidType, ContextuableGraph, Data, Datable, EuclideanSpace,
    EuclideanSpacetime, EuclideanTime, Identifiable,
};

/// A time series is not `Copy`. Before the bound was relaxed this line did not compile.
type Series = Data<Vec<f64>>;

#[test]
fn test_sequence_payload_round_trips() {
    let series = vec![50.0, 51.5, 49.25];
    let node = Series::new(1, series.clone());

    assert_eq!(node.id(), 1);
    assert_eq!(node.get_data(), series);
}

#[test]
fn test_sequence_payload_is_settable() {
    let mut node = Series::new(2, vec![1.0, 2.0]);
    node.set_data(vec![3.0, 4.0, 5.0]);

    assert_eq!(node.get_data(), vec![3.0, 4.0, 5.0]);
}

#[test]
fn test_sequence_node_lives_in_a_context() {
    // `Context` asks only for `D: Datable + Clone`, so a sequence node is a valid `D`.
    let mut context: Context<Series, EuclideanSpace, EuclideanTime, EuclideanSpacetime> =
        Context::with_capacity(1, "series", 4);

    let oil = Series::new(10, vec![50.0, 52.0, 53.5]);
    let idx = context
        .add_node(Contextoid::new(10, ContextoidType::Datoid(oil.clone())))
        .expect("a sequence-valued contextoid is accepted");

    let stored = context.get_node(idx).expect("node is retrievable");
    assert_eq!(stored.id(), 10);
    assert_eq!(context.size(), 1);

    // The empty-series counterfactual world the Granger example alternates to.
    let empty = Series::new(11, Vec::new());
    assert!(empty.get_data().is_empty());

    // The default context built by the shared helper is unaffected by the relaxation.
    assert_eq!(get_context().size(), 0);
}
