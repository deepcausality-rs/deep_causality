#![allow(clippy::too_many_arguments)]

/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::utils_test::test_utils;
use deep_causality::*;

#[test]
fn test_evaluate_subgraph_from_cause_with_relay_to_simple() {
    // Graph: Root (0) -> A (1) -> B (2) -> C (3)
    // A will relay to C
    let mut g = CausaloidGraph::new(0);

    let root_causaloid = test_utils::get_test_causaloid_deterministic_true();
    let root_index = g.add_root_causaloid(root_causaloid).unwrap();

    // Causaloid A: Relays to C (index 3) with a specific effect
    let causaloid_a_id = 10;
    let causaloid_a_description = "Causaloid A relays to node 3 with Numerical(100.0)";
    let causaloid_a_fn =
        |_effect: &PropagatingEffect| -> Result<PropagatingEffect, CausalityError> {
            Ok(PropagatingEffect::RelayTo(
                3,
                Box::new(PropagatingEffect::Deterministic(false)),
            ))
        };
    let causaloid_a = Causaloid::new(causaloid_a_id, causaloid_a_fn, causaloid_a_description);
    let idx_a = g.add_causaloid(causaloid_a).unwrap();

    // Causaloid B: Standard causaloid, should be skipped
    let causaloid_b = test_utils::get_test_causaloid_deterministic_input_output();
    let idx_b = g.add_causaloid(causaloid_b).unwrap();

    // Causaloid C: Standard causaloid, should be the final evaluated node
    let causaloid_c = test_utils::get_test_causaloid_deterministic_input_output();
    let idx_c = g.add_causaloid(causaloid_c).unwrap();

    // Link the graph: Root -> A -> B -> C
    g.add_edge(root_index, idx_a).unwrap();
    g.add_edge(idx_a, idx_b).unwrap();
    g.add_edge(idx_b, idx_c).unwrap();

    g.freeze();

    let initial_effect = PropagatingEffect::from_deterministic(true);
    let res = g.evaluate_subgraph_from_cause(root_index, &initial_effect);

    dbg!(&res);
    assert!(res.is_ok());
    // Expected: Root (true) -> A (Relay to C with Deterministic(false))
    // C (input  Deterministic(false)) -> Deterministic(true) (C just inverts the input)
    assert_eq!(res.unwrap(), PropagatingEffect::from_deterministic(true));
}


// #[test]
// fn test_evaluate_subgraph_from_cause_with_relay_to_visited_node() {
//     // Graph: Root (0) -> A (1) -> B (2)
//     // B will relay back to Root (0)
//     let mut g = CausaloidGraph::new(0);
//
//     let root_causaloid = test_utils::get_test_causaloid_deterministic_true();
//     let root_index = g.add_root_causaloid(root_causaloid).unwrap();
//
//     let causaloid_a = test_utils::get_test_causaloid_deterministic_input_output();
//     let idx_a = g.add_causaloid(causaloid_a).unwrap();
//
//     // Causaloid B: Relays back to Root (index 0)
//     let causaloid_b_id = 11;
//     let causaloid_b_description = "Causaloid B relays to node 0 with Numerical(50.0)";
//     let causaloid_b_fn =
//         |_effect: &PropagatingEffect| -> Result<PropagatingEffect, CausalityError> {
//             Ok(PropagatingEffect::RelayTo(
//                 0,
//                 Box::new(PropagatingEffect::Numerical(50.0)),
//             ))
//         };
//     let causaloid_b = Causaloid::new(causaloid_b_id, causaloid_b_fn, causaloid_b_description);
//     let idx_b = g.add_causaloid(causaloid_b).unwrap();
//
//     // Link the graph: Root -> A -> B
//     g.add_edge(root_index, idx_a).unwrap();
//     g.add_edge(idx_a, idx_b).unwrap();
//
//     g.freeze();
//
//     let initial_effect = PropagatingEffect::Deterministic(true);
//     let res = g.evaluate_subgraph_from_cause(root_index, &initial_effect);
//
//     assert!(res.is_ok());
//     // Expected: Root (true) -> A (false) -> B (Relay to Root with Numerical(50.0))
//     // The traversal should jump back to Root, but since Root is already visited, it won't be re-added.
//     // The last propagated effect will be the RelayTo effect from B.
//     assert_eq!(
//         res.unwrap(),
//         PropagatingEffect::RelayTo(0, Box::new(PropagatingEffect::Numerical(50.0)))
//     );
// }
//
// #[test]
// fn test_evaluate_shortest_path_between_causes_with_relay_interrupt() {
//     // Graph: Root (0) -> A (1) -> B (2) -> C (3)
//     // A will relay, interrupting the shortest path from Root to C
//     let mut g = CausaloidGraph::new(0);
//
//     let root_causaloid = test_utils::get_test_causaloid_deterministic_true();
//     let root_index = g.add_root_causaloid(root_causaloid).unwrap();
//
//     // Causaloid A: Relays to C (index 3) with a specific effect
//     let causaloid_a_id = 12;
//     let causaloid_a_description = "Causaloid A relays to node 3 with Numerical(200.0)";
//     let causaloid_a_fn =
//         |_effect: &PropagatingEffect| -> Result<PropagatingEffect, CausalityError> {
//             Ok(PropagatingEffect::RelayTo(
//                 3,
//                 Box::new(PropagatingEffect::Numerical(200.0)),
//             ))
//         };
//     let causaloid_a = Causaloid::new(causaloid_a_id, causaloid_a_fn, causaloid_a_description);
//     let idx_a = g.add_causaloid(causaloid_a).unwrap();
//
//     // Causaloid B: Standard causaloid, should not be reached
//     let causaloid_b = test_utils::get_test_causaloid_deterministic_input_output();
//     let idx_b = g.add_causaloid(causaloid_b).unwrap();
//
//     // Causaloid C: Standard causaloid
//     let causaloid_c = test_utils::get_test_causaloid_deterministic_input_output();
//     let idx_c = g.add_causaloid(causaloid_c).unwrap();
//
//     // Link the graph: Root -> A -> B -> C
//     g.add_edge(root_index, idx_a).unwrap();
//     g.add_edge(idx_a, idx_b).unwrap();
//     g.add_edge(idx_b, idx_c).unwrap();
//
//     g.freeze();
//
//     let initial_effect = PropagatingEffect::Deterministic(true);
//     // Attempt to find shortest path from Root (0) to C (3)
//     let res = g.evaluate_shortest_path_between_causes(root_index, idx_c, &initial_effect);
//
//     assert!(res.is_ok());
//     // Expected: The RelayTo effect from A should interrupt the path and be returned.
//     assert_eq!(
//         res.unwrap(),
//         PropagatingEffect::RelayTo(3, Box::new(PropagatingEffect::Numerical(200.0)))
//     );
// }
//
// #[test]
// fn test_evaluate_shortest_path_between_causes_with_relay_at_end() {
//     // Graph: Root (0) -> A (1) -> B (2)
//     // B will relay, as the last node on the shortest path
//     let mut g = CausaloidGraph::new(0);
//
//     let root_causaloid = test_utils::get_test_causaloid_deterministic_true();
//     let root_index = g.add_root_causaloid(root_causaloid).unwrap();
//
//     let causaloid_a = test_utils::get_test_causaloid_deterministic_input_output();
//     let idx_a = g.add_causaloid(causaloid_a).unwrap();
//
//     // Causaloid B: Relays to Root (index 0)
//     let causaloid_b_id = 13;
//     let causaloid_b_description = "Causaloid B relays to node 0 with Numerical(50.0)";
//     let causaloid_b_fn =
//         |_effect: &PropagatingEffect| -> Result<PropagatingEffect, CausalityError> {
//             Ok(PropagatingEffect::RelayTo(
//                 0,
//                 Box::new(PropagatingEffect::Numerical(50.0)),
//             ))
//         };
//     let causaloid_b = Causaloid::new(causaloid_b_id, causaloid_b_fn, causaloid_b_description);
//     let idx_b = g.add_causaloid(causaloid_b).unwrap();
//
//     // Link the graph: Root -> A -> B
//     g.add_edge(root_index, idx_a).unwrap();
//     g.add_edge(idx_a, idx_b).unwrap();
//
//     g.freeze();
//
//     let initial_effect = PropagatingEffect::Deterministic(true);
//     // Attempt to find shortest path from Root (0) to B (2)
//     let res = g.evaluate_shortest_path_between_causes(root_index, idx_b, &initial_effect);
//
//     assert!(res.is_ok());
//     // Expected: The RelayTo effect from B should be returned.
//     assert_eq!(
//         res.unwrap(),
//         PropagatingEffect::RelayTo(0, Box::new(PropagatingEffect::Numerical(50.0)))
//     );
// }
//
// #[test]
// fn test_evaluate_subgraph_from_cause_relay_to_non_existent_node() {
//     // Graph: Root (0) -> A (1)
//     // A will relay to a non-existent node (e.g., index 99)
//     let mut g = CausaloidGraph::new(0);
//
//     let root_causaloid = test_utils::get_test_causaloid_deterministic_true();
//     let root_index = g.add_root_causaloid(root_causaloid).unwrap();
//
//     // Causaloid A: Relays to a non-existent node (index 99)
//     let non_existent_target_index = 124;
//     let causaloid_a_id = 14;
//     let causaloid_a_description =
//         format!("Causaloid A relays to non-existent node {non_existent_target_index}");
//     let causaloid_a_fn =
//         |_effect: &PropagatingEffect| -> Result<PropagatingEffect, CausalityError> {
//             Ok(PropagatingEffect::RelayTo(
//                 124, // non_existent_target_index
//                 Box::new(PropagatingEffect::Numerical(1.0)),
//             ))
//         };
//
//     let causaloid_a = Causaloid::new(causaloid_a_id, causaloid_a_fn, &causaloid_a_description);
//     let idx_a = g.add_causaloid(causaloid_a).unwrap();
//
//     // Link the graph: Root -> A
//     g.add_edge(root_index, idx_a).unwrap();
//
//     g.freeze();
//
//     let initial_effect = PropagatingEffect::Deterministic(true);
//     let res = g.evaluate_subgraph_from_cause(root_index, &initial_effect);
//
//     assert!(res.is_err());
//     assert_eq!(
//         res.unwrap_err().to_string(),
//         format!(
//             "CausalityError: RelayTo target causaloid with index {non_existent_target_index} not found in graph."
//         )
//     );
// }
