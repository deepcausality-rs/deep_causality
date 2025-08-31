/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Uncertain;
use crate::types::computation::{
    ComparisonOperator, ComputationNode, copy_graph_and_get_remapped_root,
};
use ultragraph::GraphMut;

// Note: We do not implement the standard `PartialOrd` and `PartialEq` traits
// because their signatures return `bool`, which is misleading for uncertain values.
// Instead, we provide methods that correctly return a new `Uncertain<bool>`.

impl Uncertain<f64> {
    pub fn greater_than(&self, threshold: f64) -> Uncertain<bool> {
        let (mut new_graph, self_root) =
            copy_graph_and_get_remapped_root(self.graph.as_ref()).expect("Failed to copy graph");

        let op_node = ComputationNode::ComparisonOp {
            op: ComparisonOperator::GreaterThan,
            threshold,
        };
        let op_idx = new_graph
            .add_root_node(op_node)
            .expect("Failed to add root node");
        new_graph
            .add_edge(self_root, op_idx, ())
            .expect("Failed to add edge");

        Uncertain::from_graph(new_graph)
    }

    pub fn less_than(&self, threshold: f64) -> Uncertain<bool> {
        let (mut new_graph, self_root) =
            copy_graph_and_get_remapped_root(self.graph.as_ref()).expect("Failed to copy graph");

        let op_node = ComputationNode::ComparisonOp {
            op: ComparisonOperator::LessThan,
            threshold,
        };
        let op_idx = new_graph
            .add_root_node(op_node)
            .expect("Failed to add root node");
        new_graph
            .add_edge(self_root, op_idx, ())
            .expect("Failed to add edge");

        Uncertain::from_graph(new_graph)
    }

    pub fn equals(&self, threshold: f64) -> Uncertain<bool> {
        let (mut new_graph, self_root) =
            copy_graph_and_get_remapped_root(self.graph.as_ref()).expect("Failed to copy graph");

        let op_node = ComputationNode::ComparisonOp {
            op: ComparisonOperator::EqualTo,
            threshold,
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
