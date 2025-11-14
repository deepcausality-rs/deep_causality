/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality::utils_test::test_utils::get_base_context;
use deep_causality::utils_test::test_utils_graph;
use deep_causality::{BaseCausaloid, Causable, Causaloid, IdentificationValue};
use std::sync::{Arc, RwLock};

#[test]
fn test_from_causal_graph() {
    let (g, registry) = test_utils_graph::build_multi_cause_graph();

    let causaloid: BaseCausaloid<f64, bool> = Causaloid::from_causal_graph_with_registry(
        0,
        "Test Causality graph ",
        Arc::new(g),
        Arc::new(registry),
    );
    assert!(!causaloid.is_singleton());
}

#[test]
fn test_from_causal_graph_with_context() {
    let id: IdentificationValue = 1;
    let description = "tests whether data exceeds threshold of 0.55";
    let (causal_graph, registry) = test_utils_graph::build_multi_cause_graph();

    let context = get_base_context();

    let causaloid: BaseCausaloid<f64, bool> =
        Causaloid::from_causal_graph_with_context_and_registry(
            id,
            description,
            Arc::new(causal_graph),
            Arc::new(RwLock::new(context)),
            Arc::new(registry),
        );
    assert!(!causaloid.is_singleton());
}
