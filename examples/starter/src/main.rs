/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::*;

fn main() {
    println!();
    println!("Build new causality graph");
    // The graph must be mutable at first to build it.
    let mut g = get_multi_cause_graph();
    // Freeze the graph before reasoning or explaining. This is a requirement for
    // high-performance traversal algorithms.
    g.freeze();
    println!();

    println!("Full reasoning over the entire graph");
    // The new reasoning API uses a unified `Evidence` type.
    let evidence = Evidence::Numerical(0.99);
    let root_index = g.get_root_index().expect("Graph has no root");

    // Call the new reasoning method from the `CausableGraphReasoning` trait.
    let res = g
        .evaluate_subgraph_from_cause(root_index, &evidence)
        .expect("Failed to reason over the entire graph");

    // The result is now a `PropagatingEffect`, not a simple bool.
    assert_eq!(res, PropagatingEffect::Deterministic(true));
    println!();

    println!("Partial reasoning over shortest path through the graph");
    let start_index = 2;
    let stop_index = 3;

    // Call the new shortest path reasoning method.
    let res = g
        .evaluate_shortest_path_between_causes(start_index, stop_index, &evidence)
        .unwrap();
    assert_eq!(res, PropagatingEffect::Deterministic(true));
    println!();

    println!("Explain partial reasoning");
    let expl = g
        .explain_shortest_path_between_causes(start_index, stop_index)
        .unwrap();

    println!("{expl}");
}

fn get_test_causaloid(id: IdentificationValue) -> BaseCausaloid {
    let description = "tests whether data exceeds threshold of 0.55";

    // The causal function must now match the `CausalFn` type alias:
    // - It takes `&Evidence` as input.
    // - It returns `Result<PropagatingEffect, CausalityError>`.
    fn causal_fn(evidence: &Evidence) -> Result<PropagatingEffect, CausalityError> {
        // Safely extract the numerical value from the generic Evidence enum.
        let obs = match evidence {
            Evidence::Numerical(val) => *val,
            _ => return Err(CausalityError("Expected Numerical evidence.".into())),
        };

        if obs.is_sign_negative() {
            return Err(CausalityError("Observation is negative".into()));
        }

        let threshold: NumericalValue = 0.55;
        let is_active = obs.ge(&threshold);

        // Return the result wrapped in the `PropagatingEffect` enum.
        Ok(PropagatingEffect::Deterministic(is_active))
    }

    // The Causaloid constructor now takes the updated causal_fn.
    // Note: Your Causaloid::new function might need `Some(causal_fn)` if the field is an Option.
    // Based on the error messages, it seems to be a direct fn pointer.
    Causaloid::new(id, causal_fn, description)
}

fn get_multi_cause_graph() -> BaseCausalGraph {
    // Builds a multi cause graph:
    //  root
    //  / \
    //  A B
    //  \ /
    //   C

    // The CausaloidGraph constructor now requires a unique ID.
    let mut g = CausaloidGraph::new(0);

    // Add root causaloid
    let root_causaloid = get_test_causaloid(0);
    let root_index = g
        .add_root_causaloid(root_causaloid)
        .expect("Failed to add root");

    // Add causaloid A
    let causaloid = get_test_causaloid(1);
    let idx_a = g
        .add_causaloid(causaloid)
        .expect("Failed to add causaloid A");

    // Link causaloid A to root causaloid
    g.add_edge(root_index, idx_a)
        .expect("Failed to add edge between root and A");

    // Add causaloid B
    let causaloid = get_test_causaloid(2);
    let idx_b = g
        .add_causaloid(causaloid)
        .expect("Failed to add causaloid B");

    // Link causaloid B to root causaloid
    g.add_edge(root_index, idx_b)
        .expect("Failed to add edge between root and B");

    // Add causaloid C
    let causaloid = get_test_causaloid(3);
    let idx_c = g
        .add_causaloid(causaloid)
        .expect("Failed to add causaloid C");

    // Link causaloid C to A
    g.add_edge(idx_a, idx_c)
        .expect("Failed to add edge between A and C");

    // Link causaloid C to B
    g.add_edge(idx_b, idx_c)
        .expect("Failed to add edge between C and B");

    g
}
