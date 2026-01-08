/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality::utils_test::test_utils;
use deep_causality::utils_test::test_utils_graph;
use deep_causality::{BaseCausaloid, Causable, Causaloid, IdentificationValue};
use std::sync::{Arc, RwLock};

#[test]
fn test_from_causal_graph() {
    let g = test_utils_graph::build_multi_cause_graph();

    let causaloid: BaseCausaloid<f64, f64> =
        Causaloid::from_causal_graph(0, "Test Causality graph ", Arc::new(g));
    assert!(!causaloid.is_singleton());
    assert!(causaloid.context().is_none());
}

#[test]
fn test_from_causal_graph_with_context() {
    let id: IdentificationValue = 1;
    let description = "tests whether data exceeds threshold of 0.55";
    let causal_graph = test_utils_graph::build_multi_cause_graph();
    let context = test_utils::get_base_context();

    let causaloid: BaseCausaloid<f64, f64> = Causaloid::from_causal_graph_with_context(
        id,
        description,
        Arc::new(causal_graph),
        Arc::new(RwLock::new(context)),
    );
    assert!(!causaloid.is_singleton());
    assert!(causaloid.context().is_some());
}
