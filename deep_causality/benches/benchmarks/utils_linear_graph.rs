/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::{
    CausableGraph, CausaloidGraph, CausaloidId, CausaloidRegistry, IdentificationValue,
};

use crate::benchmarks::utils_shared;

pub fn build_linear_graph(k: usize) -> (CausaloidGraph<CausaloidId>, CausaloidRegistry) {
    let mut g = CausaloidGraph::<CausaloidId>::new(0 as IdentificationValue);
    let mut registry = CausaloidRegistry::new();

    let root_causaloid = utils_shared::get_test_causaloid();
    let root_causaloid_id = registry.register(root_causaloid);
    let root_index = g
        .add_root_causaloid(root_causaloid_id)
        .expect("Failed to add root causaloid");

    let mut previous_idx = root_index;

    for _ in 0..k {
        let causaloid = utils_shared::get_test_causaloid();
        let causaloid_id = registry.register(causaloid);
        let current_idx = g
            .add_causaloid(causaloid_id)
            .expect("Failed to add causaloid");

        g.add_edge(previous_idx, current_idx)
            .expect("Failed to add edge");

        previous_idx = current_idx;
    }

    g.freeze();

    (g, registry)
}

pub fn get_small_multi_cause_graph_and_data()
-> (CausaloidGraph<CausaloidId>, CausaloidRegistry, [f64; 4]) {
    let (g, registry) = build_multi_cause_graph();
    (g, registry, utils_shared::generate_sample_data())
}

fn build_multi_cause_graph() -> (CausaloidGraph<CausaloidId>, CausaloidRegistry) {
    let mut g = CausaloidGraph::<CausaloidId>::new(0 as IdentificationValue);
    let mut registry = CausaloidRegistry::new();

    // Add root causaloid
    let root_causaloid = utils_shared::get_test_causaloid();
    let root_causaloid_id = registry.register(root_causaloid);
    let root_index = g
        .add_root_causaloid(root_causaloid_id)
        .expect("Failed to add root causaloid");

    // Add causaloid A
    let causaloid = utils_shared::get_test_causaloid();
    let idx_a_id = registry.register(causaloid);
    let idx_a = g.add_causaloid(idx_a_id).expect("Failed to add causaloid");

    // Link causaloid A to root causaloid
    g.add_edge(root_index, idx_a)
        .expect("Failed to add edge between root and A");

    // Add causaloid B
    let causaloid = utils_shared::get_test_causaloid();
    let idx_b_id = registry.register(causaloid);
    let idx_b = g.add_causaloid(idx_b_id).expect("Failed to add causaloid");

    // Link causaloid B to root causaloid
    g.add_edge(root_index, idx_b)
        .expect("Failed to add edge between root and B");

    // Add causaloid C
    let causaloid = utils_shared::get_test_causaloid();
    let idx_c_id = registry.register(causaloid);
    let idx_c = g.add_causaloid(idx_c_id).expect("Failed to add causaloid");

    // Link causaloid C to A
    g.add_edge(idx_a, idx_c)
        .expect("Failed to add edge between A and C");

    // Link causaloid C to B
    g.add_edge(idx_b, idx_c)
        .expect("Failed to add edge between C and B");

    g.freeze();

    (g, registry)
}
