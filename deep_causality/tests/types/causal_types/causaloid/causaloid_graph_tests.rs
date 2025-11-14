/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality::utils_test::test_utils::get_base_context;
use deep_causality::utils_test::test_utils_graph;
use deep_causality::{BaseCausaloid, Causable, Causaloid, EffectValue, IdentificationValue};
use std::sync::{Arc, RwLock};

#[test]
fn test_from_causal_graph() {
    let (g, registry) = test_utils_graph::build_multi_cause_graph();

    // Here we're using the EffectValue container type to ensure many different causaloids can be used.
    let causaloid: BaseCausaloid<EffectValue, EffectValue> =
        Causaloid::from_causal_graph_with_registry(
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

    // Here we limit the BaseCausaloid stored in the graph to those that take in an f64 and return a bool
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
