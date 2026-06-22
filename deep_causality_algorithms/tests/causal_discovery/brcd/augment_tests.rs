/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algorithms::brcd::brcd_augment::{
    augmented_graph, f_node_indicator, get_configurations_multi,
};
use deep_causality_algorithms::brcd::brcd_error::{BrcdError, BrcdErrorEnum};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::MixedGraph;

fn graph(n: usize) -> MixedGraph<()> {
    let data = CausalTensor::new(vec![(); n], vec![n]).unwrap();
    MixedGraph::new(n, data, 0).unwrap()
}

/// Every returned configuration is fully validated: acyclic.
fn all_acyclic(configs: &[MixedGraph<()>]) -> bool {
    configs.iter().all(|g| !g.has_cycle())
}

// --- f_node_indicator -------------------------------------------------------

#[test]
fn f_node_indicator_marks_normal_then_anomalous() {
    assert_eq!(f_node_indicator(2, 3), vec![false, false, true, true, true]);
    assert_eq!(f_node_indicator(0, 0), Vec::<bool>::new());
    assert_eq!(f_node_indicator(1, 0), vec![false]);
}

// --- get_configurations_multi ----------------------------------------------

#[test]
fn arcs_only_yields_a_single_configuration() {
    // 0 → 1 → 2, target {1}: no undirected edge incident → one config.
    let mut g = graph(3);
    g.add_arc(0, 1).unwrap();
    g.add_arc(1, 2).unwrap();
    let configs = get_configurations_multi(&g, &[1]).unwrap();
    assert_eq!(configs.len(), 1);
    assert!(all_acyclic(&configs));
    // The compelled arcs are preserved.
    assert!(configs[0].arcs().contains(&(0, 1)));
    assert!(configs[0].arcs().contains(&(1, 2)));
}

#[test]
fn single_incident_edge_yields_two_orientations() {
    // 0 — 1, target {1}: both orientations are collider-free → two configs.
    let mut g = graph(2);
    g.add_undirected(0, 1).unwrap();
    let configs = get_configurations_multi(&g, &[1]).unwrap();
    assert_eq!(configs.len(), 2);
    assert!(all_acyclic(&configs));
}

#[test]
fn new_unshielded_collider_orientation_is_excluded() {
    // 0 — 1 — 2 with 0, 2 non-adjacent; target {1}. Of the 4 orientations, only
    // 0 → 1 ← 2 is a new unshielded collider at 1, so 3 configs survive.
    let mut g = graph(3);
    g.add_undirected(0, 1).unwrap();
    g.add_undirected(1, 2).unwrap();
    let configs = get_configurations_multi(&g, &[1]).unwrap();
    assert_eq!(configs.len(), 3);
    assert!(all_acyclic(&configs));
    // No surviving config has both 0 and 2 as parents of 1 (the excluded collider).
    for c in &configs {
        let parents = c.parents(1);
        assert!(!(parents.contains(&0) && parents.contains(&2)));
    }
}

#[test]
fn shielded_collider_is_allowed_but_cycles_are_pruned() {
    // 0 — 1 — 2 with 0 → 2 present. Of the 4 orientations of {0—1, 1—2}:
    //   0→1,1→2  ✓   1→0,1→2  ✓   0→1,2→1 ✓ (collider 0→1←2 is now *shielded*
    //   by 0→2, so allowed)   1→0,2→1 ✗ (closes the cycle 1→0→2→1).
    // → 3 configs: the shielded collider survives, the cyclic orientation does not.
    let mut g = graph(3);
    g.add_undirected(0, 1).unwrap();
    g.add_undirected(1, 2).unwrap();
    g.add_arc(0, 2).unwrap();
    let configs = get_configurations_multi(&g, &[1]).unwrap();
    assert_eq!(configs.len(), 3);
    assert!(all_acyclic(&configs));
    // The shielded collider (both 0 and 2 as parents of 1) does appear.
    assert!(
        configs
            .iter()
            .any(|c| c.parents(1).contains(&0) && c.parents(1).contains(&2))
    );
}

#[test]
fn is_deterministic() {
    let mut g = graph(3);
    g.add_undirected(0, 1).unwrap();
    g.add_undirected(1, 2).unwrap();
    let a = get_configurations_multi(&g, &[1]).unwrap();
    let b = get_configurations_multi(&g, &[1]).unwrap();
    let edges = |cs: &[MixedGraph<()>]| cs.iter().map(|g| g.arcs()).collect::<Vec<_>>();
    assert_eq!(edges(&a), edges(&b));
}

#[test]
fn out_of_bounds_target_is_rejected() {
    let g = graph(2);
    assert_eq!(
        get_configurations_multi(&g, &[5]).err(),
        Some(BrcdError(BrcdErrorEnum::NodeOutOfBounds))
    );
}

#[test]
fn too_many_incident_edges_are_refused() {
    // A star: center 0 with 17 undirected leaves → 2^17 configurations, refused.
    let leaves = 17;
    let mut g = graph(leaves + 1);
    for leaf in 1..=leaves {
        g.add_undirected(0, leaf).unwrap();
    }
    assert_eq!(
        get_configurations_multi(&g, &[0]).err(),
        Some(BrcdError(BrcdErrorEnum::ConfigSpaceTooLarge { edges: 17 }))
    );
}

#[test]
fn candidate_with_no_valid_configuration_yields_empty() {
    // CPDAG: arcs 1→2, 2→0, 3→0, plus undirected 0—1. Acyclic by construction.
    // For candidate {0} the single incident undirected edge (0—1) is invalid in
    // both orientations:
    //   0→1 closes the cycle 0→1→2→0 (cyclic), and
    //   1→0 makes 1 a parent of 0 non-adjacent to the existing parent 3, a new
    //       unshielded collider 1→0←3.
    // So neither orientation survives → get_configurations_multi is empty. This
    // is the structural precondition for the −∞ / None-plan branch in brcd_run.
    let mut g = graph(4);
    g.add_arc(1, 2).unwrap();
    g.add_arc(2, 0).unwrap();
    g.add_arc(3, 0).unwrap();
    g.add_undirected(0, 1).unwrap();
    assert!(!g.has_cycle());
    let configs = get_configurations_multi(&g, &[0]).unwrap();
    assert!(configs.is_empty(), "expected no valid configuration");
}

// --- augmented_graph --------------------------------------------------------

#[test]
fn augmentation_adds_the_fnode_and_root_arcs() {
    // Config 0 → 1, 1 — 2; augment for root {1}.
    let mut g = graph(3);
    g.add_arc(0, 1).unwrap();
    g.add_undirected(1, 2).unwrap();
    let aug = augmented_graph(&g, &[1]).unwrap();

    let fnode = 3; // == original num_vertices
    assert_eq!(aug.num_vertices(), 4);
    // Original edges preserved.
    assert!(aug.arcs().contains(&(0, 1)));
    assert_eq!(aug.edge_kind(1, 2), g.edge_kind(1, 2));
    // FNODE → root arc added.
    assert!(aug.arcs().contains(&(fnode, 1)));
}

#[test]
fn augmentation_with_multiple_roots() {
    let mut g = graph(3);
    g.add_arc(0, 1).unwrap();
    let aug = augmented_graph(&g, &[1, 2]).unwrap();
    let fnode = 3;
    assert!(aug.arcs().contains(&(fnode, 1)));
    assert!(aug.arcs().contains(&(fnode, 2)));
}

#[test]
fn augmentation_rejects_out_of_bounds_root() {
    let g = graph(2);
    assert_eq!(
        augmented_graph(&g, &[9]).err(),
        Some(BrcdError(BrcdErrorEnum::NodeOutOfBounds))
    );
}
