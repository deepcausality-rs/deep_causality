/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{GraphTopology, MixedGraph, TopologyError, TopologyErrorEnum};

fn graph(n: usize) -> MixedGraph<()> {
    let data = CausalTensor::new(vec![(); n], vec![n]).unwrap();
    MixedGraph::new(n, data, 0).unwrap()
}

// `get_neighbors` filters every stored canonical key `(a, b)`. The high-side
// branch (`b == node_id` => Some(a)) and the no-match branch (`None`) are only
// reached when the queried node sits on the high end of some keys and is
// entirely absent from others. Edge keys are stored canonically with `lo < hi`,
// so querying the largest node id forces it onto the `b` side, while an
// unrelated edge supplies the `None` filter case.
#[test]
fn get_neighbors_covers_high_side_and_skip_branches() {
    let mut g = graph(4);
    g.add_undirected(0, 3).unwrap(); // key (0, 3): 3 is on the high (b) side
    g.add_arc(1, 3).unwrap(); // key (1, 3): 3 is on the high (b) side
    g.add_undirected(0, 1).unwrap(); // key (0, 1): does NOT contain node 3

    let neighbors = g.get_neighbors(3).unwrap();
    assert_eq!(neighbors, vec![0, 1]);
}

// The `orient` None branch: orienting a pair with no edge between them must be
// rejected (distinct from the "not undirected" rejection).
#[test]
fn orient_missing_edge_is_rejected() {
    let mut g = graph(3);
    let err = g
        .orient(0, 1)
        .expect_err("orienting a pair with no edge must be rejected");
    assert!(matches!(
        err,
        TopologyError(TopologyErrorEnum::GraphError(_))
    ));
}
