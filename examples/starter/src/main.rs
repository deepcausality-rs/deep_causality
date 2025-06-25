/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::prelude::*;

fn main() {
    println!();
    println!("Build new causality graph");
    let g = get_multi_cause_graph();
    println!();

    println!("Verify that the graph is fully inactive");
    let percent_active = g.percent_active();
    assert_eq!(percent_active, 0.0);
    assert_eq!(g.number_active(), 0.0);
    assert!(!g.all_active());
    println!();

    println!("Full reasoning over the entire graph");
    let data = [0.99; 5]; // sample data
    let res = g
        .reason_all_causes(&data, None)
        .expect("Failed to reason over the entire graph");
    assert!(res);
    println!();

    println!("Verify that the graph is fully active");
    assert_eq!(g.percent_active(), 100.0);
    assert!(g.all_active());
    println!();

    println!("Partial reasoning over shortest path through the graph");
    let start_index = 2;
    let stop_index = 3;
    let res = g
        .reason_shortest_path_between_causes(start_index, stop_index, &data, None)
        .unwrap();
    assert!(res);
    println!();

    println!("Explain partial reasoning");
    let expl = g
        .explain_shortest_path_between_causes(start_index, stop_index)
        .unwrap();

    println!("{}", expl);
}

fn get_test_causaloid<'l>(id: IdentificationValue) -> BaseCausaloid<'l> {
    let description = "tests whether data exceeds threshold of 0.55";
    fn causal_fn(obs: NumericalValue) -> Result<bool, CausalityError> {
        if obs.is_sign_negative() {
            return Err(CausalityError("Observation is negative".into()));
        }

        let threshold: NumericalValue = 0.55;
        if !obs.ge(&threshold) {
            Ok(false)
        } else {
            Ok(true)
        }
    }

    Causaloid::new(id, causal_fn, description)
}

fn get_multi_cause_graph<'l>() -> BaseCausalGraph<'l> {
    // Builds a multi cause graph:
    //  root
    //  / \
    //  A B
    //  \ /
    //   C

    // Create a new causaloid graph
    let mut g = CausaloidGraph::new();

    // Add root causaloid
    let root_causaloid = get_test_causaloid(0);
    let root_index = g.add_root_causaloid(root_causaloid);

    // Add causaloid A
    let causaloid = get_test_causaloid(1);
    let idx_a = g.add_causaloid(causaloid);

    // Link causaloid A to root causaloid
    g.add_edge(root_index, idx_a)
        .expect("Failed to add edge between root and A");

    // Add causaloid B
    let causaloid = get_test_causaloid(2);
    let idx_b = g.add_causaloid(causaloid);

    // Link causaloid B to root causaloid
    g.add_edge(root_index, idx_b)
        .expect("Failed to add edge between root and B");

    // Add causaloid C
    let causaloid = get_test_causaloid(3);
    let idx_c = g.add_causaloid(causaloid);

    // Link causaloid C to A
    g.add_edge(idx_a, idx_c)
        .expect("Failed to add edge between A and C");

    // Link causaloid C to B
    g.add_edge(idx_b, idx_c)
        .expect("Failed to add edge between C and B");

    g
}
