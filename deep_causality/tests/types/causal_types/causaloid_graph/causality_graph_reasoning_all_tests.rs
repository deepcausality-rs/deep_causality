/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality::utils_test::test_utils_graph;
use deep_causality::*;

#[test]
fn test_graph_evaluate() {
    let (g, registry) = test_utils_graph::build_multi_cause_graph();

    let root_index = 0;
    let idx_a = 1;
    let idx_b = 2;

    // Create an initial effect to be applied to the root node
    let effect = PropagatingEffect::from_numerical(0.99); // A value that will activate all nodes
    // Here we evaluate the effect on the root node only,
    let res = g.evaluate_shortest_path_between_causes(&registry, root_index, root_index, &effect);
    // dbg!(&res);
    assert!(res.is_ok());
    // The root node returns Deterministic(true) because its causal function evaluates to true w.r.t. to effect

    // Now, we evaluate the effect propagating from root to A
    // Root evaluates from Numerical to true; and A evaluates from Boolean true to Boolean false
    let res = g.evaluate_shortest_path_between_causes(&registry, root_index, idx_a, &effect);
    dbg!(&res);
    assert!(res.is_ok());

    // Now, we evaluate the effect propagating from  root -> A -> B
    // Root evaluates from Numerical to true;
    // A evaluates from Boolean true to Boolean false;
    // B evaluates from Boolean false to Boolean true
    let res = g.evaluate_shortest_path_between_causes(&registry, root_index, idx_b, &effect);
    dbg!(&res);
    assert!(res.is_ok());
}

// #[test]
// fn test_graph_evaluate_error_conditions() {
//     // Test case 1: Graph has no root
//     let g: BaseCausalGraph = CausaloidGraph::new(0);
//     assert!(g.is_empty());
//     assert!(!g.contains_root_causaloid());
//
//     let effect = PropagatingEffect::Numerical(0.99);
//     let res = g.evaluate(&effect);
//     assert!(res.is_err());
//     assert_eq!(
//         res.unwrap_err().to_string(),
//         "CausalityError: Cannot evaluate graph: Root node not found."
//     );
//
//     // Test case 2: Graph is not frozen
//     let mut g = CausaloidGraph::new(0);
//     let root_causaloid = test_utils::get_test_causaloid_deterministic();
//     g.add_root_causaloid(root_causaloid).unwrap();
//     // DO NOT call g.freeze()
//
//     let res = g.evaluate(&effect);
//     assert!(res.is_err());
//     // The error comes from `evaluate_subgraph_from_cause` which is called by `evaluate`
//     assert_eq!(
//         res.unwrap_err().to_string(),
//         "CausalityError: Graph is not frozen. Call freeze() first"
//     );
// }
