/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::types::computation::{
    ComputationNode, LogicalOperator, copy_graph_and_get_remapped_root,
};
use crate::{Uncertain, merge_graphs};
use std::ops::{BitAnd, BitOr, Not};
use ultragraph::{GraphMut, UltraGraph};

impl BitAnd for Uncertain<bool> {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        let mut new_graph = UltraGraph::new();
        let (lhs_root, rhs_root) = merge_graphs::<bool>(&mut new_graph, &self.graph, &rhs.graph)
            .expect("Failed to merge graphs");
        let op_node = ComputationNode::LogicalOp {
            op: LogicalOperator::And,
        };
        let op_idx = new_graph
            .add_root_node(op_node)
            .expect("Failed to add root node");
        new_graph
            .add_edge(lhs_root, op_idx, ())
            .expect("Failed to add edge");
        new_graph
            .add_edge(rhs_root, op_idx, ())
            .expect("Failed to add edge");
        Uncertain::from_graph(new_graph)
    }
}

impl BitOr for Uncertain<bool> {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        let mut new_graph = UltraGraph::new();
        let (lhs_root, rhs_root) = merge_graphs::<bool>(&mut new_graph, &self.graph, &rhs.graph)
            .expect("Failed to merge graphs");
        let op_node = ComputationNode::LogicalOp {
            op: LogicalOperator::Or,
        };
        let op_idx = new_graph
            .add_root_node(op_node)
            .expect("Failed to add root node");
        new_graph
            .add_edge(lhs_root, op_idx, ())
            .expect("Failed to add edge");
        new_graph
            .add_edge(rhs_root, op_idx, ())
            .expect("Failed to add edge");
        Uncertain::from_graph(new_graph)
    }
}

impl Not for Uncertain<bool> {
    type Output = Self;
    fn not(self) -> Self::Output {
        let (mut new_graph, self_root) =
            copy_graph_and_get_remapped_root(self.graph.as_ref()).expect("Failed to copy graph");
        let op_node = ComputationNode::LogicalOp {
            op: LogicalOperator::Not,
        };
        let op_idx = new_graph
            .add_root_node(op_node)
            .expect("Failed to add root node");
        new_graph
            .add_edge(self_root, op_idx, ())
            .expect("Failed to add edge");
        Uncertain::from_graph(new_graph)
    }
}
