/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::computation::{
    ComparisonOperator, ComputationNode, copy_graph_and_get_remapped_root,
};
use crate::{Uncertain, merge_graphs};
use ultragraph::{GraphMut, UltraGraph};

// Note: We do not implement the standard `PartialOrd` and `PartialEq` traits
// because their signatures return `bool`, which is misleading for uncertain values.
// Instead, we provide methods that correctly return a new `Uncertain<bool>`.

impl Uncertain<f64> {
    pub fn gt(&self, threshold: f64) -> Uncertain<bool> {
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

    pub fn lt(&self, threshold: f64) -> Uncertain<bool> {
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

    pub fn eq(&self, threshold: f64) -> Uncertain<bool> {
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

    pub fn gt_uncertain(&self, other: &Self) -> Uncertain<bool> {
        let mut new_graph = UltraGraph::new();
        let (lhs_root, rhs_root) = merge_graphs::<f64>(&mut new_graph, &self.graph, &other.graph)
            .expect("Failed to merge graphs");

        let op_node = ComputationNode::ComparisonOp {
            op: ComparisonOperator::GreaterThan,
            threshold: 0.0, // Threshold is not used for uncertain vs uncertain comparison
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

    pub fn lt_uncertain(&self, other: &Self) -> Uncertain<bool> {
        let mut new_graph = UltraGraph::new();
        let (lhs_root, rhs_root) = merge_graphs::<f64>(&mut new_graph, &self.graph, &other.graph)
            .expect("Failed to merge graphs");

        let op_node = ComputationNode::ComparisonOp {
            op: ComparisonOperator::LessThan,
            threshold: 0.0, // Threshold is not used for uncertain vs uncertain comparison
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

    pub fn eq_uncertain(&self, other: &Self) -> Uncertain<bool> {
        let mut new_graph = UltraGraph::new();
        let (lhs_root, rhs_root) = merge_graphs::<f64>(&mut new_graph, &self.graph, &other.graph)
            .expect("Failed to merge graphs");

        let op_node = ComputationNode::ComparisonOp {
            op: ComparisonOperator::EqualTo,
            threshold: 0.0, // Threshold is not used for uncertain vs uncertain comparison
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

    /// Check if value is approximately equal within tolerance
    pub fn approx_eq(&self, target: f64, tolerance: f64) -> Uncertain<bool> {
        self.map_to_bool(move |x| (x - target).abs() <= tolerance)
    }

    /// Check if value is within a range
    pub fn within_range(&self, min: f64, max: f64) -> Uncertain<bool> {
        self.map_to_bool(move |x| x >= min && x <= max)
    }
}
