/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::utils_test::test_utils_shared;
use crate::{BaseCausaloid, CausableGraph, CausaloidGraph, IdentificationValue};

pub fn build_linear_graph(k: usize) -> CausaloidGraph<BaseCausaloid<f64, bool>> {
    let mut g = CausaloidGraph::<BaseCausaloid<f64, bool>>::new(0 as IdentificationValue);

    let root_causaloid = test_utils_shared::get_test_causaloid(0);
    let root_index = g
        .add_root_causaloid(root_causaloid)
        .expect("Failed to add root causaloid");

    let mut previous_idx = root_index;

    for i in 1..k {
        let causaloid = test_utils_shared::get_test_causaloid(i as IdentificationValue);
        let current_idx = g.add_causaloid(causaloid).expect("Failed to add causaloid");

        g.add_edge(previous_idx, current_idx)
            .expect("Failed to add edge");

        previous_idx = current_idx;
    }

    g.freeze();

    g
}

pub fn get_small_multi_cause_graph_and_data() -> (CausaloidGraph<BaseCausaloid<f64, bool>>, [f64; 4])
{
    let g = build_multi_cause_graph();
    (g, test_utils_shared::generate_sample_data())
}

pub fn build_multi_cause_graph() -> CausaloidGraph<BaseCausaloid<f64, bool>> {
    let mut g = CausaloidGraph::<BaseCausaloid<f64, bool>>::new(0 as IdentificationValue);

    // Add root causaloid
    let root_causaloid = test_utils_shared::get_test_causaloid(0);
    let root_index = g
        .add_root_causaloid(root_causaloid)
        .expect("Failed to add root causaloid");

    // Add causaloid A
    let causaloid = test_utils_shared::get_test_causaloid(1);
    let idx_a = g.add_causaloid(causaloid).expect("Failed to add causaloid");

    // Link causaloid root to A causaloid
    g.add_edge(root_index, idx_a)
        .expect("Failed to add edge between root and A");

    // Add causaloid B
    let causaloid = test_utils_shared::get_test_causaloid(2);
    let idx_b = g.add_causaloid(causaloid).expect("Failed to add causaloid");

    // Link causaloid B to root causaloid
    g.add_edge(root_index, idx_b)
        .expect("Failed to add edge between root and B");

    // Add causaloid C
    let causaloid = test_utils_shared::get_test_causaloid(3);
    let idx_c = g.add_causaloid(causaloid).expect("Failed to add causaloid");

    // Link causaloid A to C
    g.add_edge(idx_a, idx_c)
        .expect("Failed to add edge between A and C");

    // Link causaloid B to C
    g.add_edge(idx_b, idx_c)
        .expect("Failed to add edge between C and B");

    // Now, we have a graph like this:
    // root -> A -> B  -> C
    g.freeze();

    g
}
