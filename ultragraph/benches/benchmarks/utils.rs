/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use core::ops::Range;
use deep_causality_rand::{Rng, Xoshiro256};
use ultragraph::*;

use crate::benchmarks::data::Data;
use crate::benchmarks::fields::{INDEX_COUNT, SEED};

/// A root and `k` nodes linked in a line: `k + 1` nodes, `k` edges, every node of degree at most 1.
/// The node tables are reserved for exactly `k + 1` nodes, so building never reallocates them.
pub fn build_linear_graph(k: usize) -> UltraGraph<Data> {
    build_linear_graph_with_capacity(k, k + 1)
}

/// The chain of `build_linear_graph(k)`, with the node tables reserved for `capacity` nodes.
pub fn build_linear_graph_with_capacity(k: usize, capacity: usize) -> UltraGraph<Data> {
    let mut g: UltraGraphContainer<Data, _> = UltraGraph::with_capacity(capacity, None);

    let d = Data::default();

    let root_index = g.add_root_node(d).expect("Failed to add node");
    let mut previous_idx = root_index;

    for _ in 0..k {
        // add a new causaloid and set current idx to it
        let data = Data::default();
        let current_idx = g.add_node(data).expect("Failed to add node");

        // link current causaloid to previos causaloid
        g.add_edge(previous_idx, current_idx, ())
            .expect("Failed to add edge");

        previous_idx = current_idx;
    }

    g
}

/// A hub at index 0 with an edge to each of the `k` leaves at indices `1..=k`, so the hub's degree
/// is `k`. Built by pushes, so each edge list carries the capacity growth of real use.
pub fn build_star_graph(k: usize) -> UltraGraph<Data> {
    let mut g: UltraGraphContainer<Data, _> = UltraGraph::with_capacity(k + 1, None);

    let hub = g.add_node(Data::default()).expect("Failed to add node");
    for _ in 0..k {
        let leaf = g.add_node(Data::default()).expect("Failed to add node");
        g.add_edge(hub, leaf, ()).expect("Failed to add edge");
    }

    g
}

/// A chain of `k + 1` nodes whose order is a seeded random permutation of the positions, so that
/// consecutive steps of a walk land at unpredictable addresses. Returns the graph, the first node
/// and the last node of the chain.
pub fn build_shuffled_chain(k: usize) -> (UltraGraph<Data>, usize, usize) {
    let mut g: UltraGraphContainer<Data, _> = UltraGraph::with_capacity(k + 1, None);
    for _ in 0..=k {
        g.add_node(Data::default()).expect("Failed to add node");
    }

    let mut order: Vec<usize> = (0..=k).collect();
    let mut rng = seeded_rng();
    for i in (1..order.len()).rev() {
        let j = rng.random_range(0..i + 1);
        order.swap(i, j);
    }

    for pair in order.windows(2) {
        g.add_edge(pair[0], pair[1], ())
            .expect("Failed to add edge");
    }

    (g, order[0], order[k])
}

/// The generator every benchmark draws from, seeded with `SEED`.
pub fn seeded_rng() -> Xoshiro256 {
    Xoshiro256::from_seed(SEED)
}

/// `INDEX_COUNT` indices drawn uniformly from `range` by the seeded generator. Read benchmarks
/// cycle through them, so the generator's cost stays out of the measurement.
pub fn random_indices(range: Range<usize>) -> Vec<usize> {
    let mut rng = seeded_rng();
    (0..INDEX_COUNT)
        .map(|_| rng.random_range(range.clone()))
        .collect()
}
