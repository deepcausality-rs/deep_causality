/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::UncertainGraph;
use std::collections::HashMap;
use ultragraph::{GraphError, GraphMut, GraphView, UltraGraph};

/// Merges two source graphs into a new destination graph.
///
/// This is a critical utility for combining computation graphs when an operator
/// like `+` or `*` is used. It copies all nodes and edges from the source graphs
/// into the destination graph, keeping track of the mapping from old node indices
/// to new ones.
///
/// # Returns
/// A tuple containing the new node indices of the root nodes from the left and
/// right source graphs, respectively: `(new_lhs_root_idx, new_rhs_root_idx)`.
pub fn merge_graphs<T: Copy + Send + Sync + 'static>(
    dest: &mut UncertainGraph,
    lhs: &UncertainGraph,
    rhs: &UncertainGraph,
) -> Result<(usize, usize), GraphError> {
    let mut lhs_map: HashMap<usize, usize> = HashMap::new();
    let mut rhs_map: HashMap<usize, usize> = HashMap::new();

    // Clone nodes from LHS graph
    for (old_idx, node) in lhs.get_all_nodes().iter().enumerate() {
        let new_idx = dest.add_node(**node)?;
        lhs_map.insert(old_idx, new_idx);
    }

    // Clone edges from LHS graph
    for old_src_idx in 0..lhs.number_nodes() {
        if let Some(edges) = lhs.get_edges(old_src_idx) {
            for (old_target_idx, _) in edges {
                let new_src = lhs_map[&old_src_idx];
                let new_target = lhs_map[&old_target_idx];
                dest.add_edge(new_src, new_target, ())?;
            }
        }
    }

    // Clone nodes from RHS graph
    for (old_idx, node) in rhs.get_all_nodes().iter().enumerate() {
        let new_idx = dest.add_node(**node)?;
        rhs_map.insert(old_idx, new_idx);
    }

    // Clone edges from RHS graph
    for old_src_idx in 0..rhs.number_nodes() {
        if let Some(edges) = rhs.get_edges(old_src_idx) {
            for (old_target_idx, _) in edges {
                let new_src = rhs_map[&old_src_idx];
                let new_target = rhs_map[&old_target_idx];
                dest.add_edge(new_src, new_target, ())?;
            }
        }
    }

    let new_lhs_root = lhs_map[&lhs
        .get_root_index()
        .expect("LHS graph must have a root node")];
    let new_rhs_root = rhs_map[&rhs
        .get_root_index()
        .expect("RHS graph must have a root node")];

    Ok((new_lhs_root, new_rhs_root))
}

/// Copies a source graph into a new UltraGraph and returns the new graph
/// along with the remapped index of the original root node.
pub fn copy_graph_and_get_remapped_root(
    source_graph: &UncertainGraph,
) -> Result<(UncertainGraph, usize), GraphError> {
    let mut new_graph = UltraGraph::new();
    let mut node_map: HashMap<usize, usize> = HashMap::new();

    // Copy nodes and remap indices
    for (old_idx, node_data) in source_graph.get_all_nodes().iter().enumerate() {
        let new_idx = new_graph.add_node(**node_data)?;
        node_map.insert(old_idx, new_idx);
    }

    // Copy edges with remapped indices
    for old_src_idx in 0..source_graph.number_nodes() {
        if let Some(edges) = source_graph.get_edges(old_src_idx) {
            for (old_target_idx, weight) in edges {
                let new_src = node_map[&old_src_idx];
                let new_target = node_map[&old_target_idx];
                new_graph.add_edge(new_src, new_target, *weight)?;
            }
        }
    }

    let remapped_root_idx = node_map[&source_graph
        .get_root_index()
        .expect("Source graph must have a root node")];

    Ok((new_graph, remapped_root_idx))
}
