/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{ComputationNode, Operator, Uncertain, merge_graphs};
use std::ops::{Add, Mul};
use ultragraph::{GraphMut, UltraGraph};

// Operator Overloading
impl Add for Uncertain {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut new_graph = UltraGraph::new();
        let (lhs_root, rhs_root) = merge_graphs(&mut new_graph, &self.graph, &rhs.graph).expect("Failed to merge graphs");

        let op_node = ComputationNode::BinaryOp { op: Operator::Add };
        // Create the new operator node and set it as the root in one step.
        let op_idx = new_graph.add_root_node(op_node).expect("Failed to add root node");

        // The old roots are now children of the new root.
        new_graph.add_edge(lhs_root, op_idx, ()).expect("Failed to add edge");
        new_graph.add_edge(rhs_root, op_idx, ()).expect("Failed to add edge");

        Self::from_graph(new_graph)
    }
}

impl Mul for Uncertain {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let mut new_graph = UltraGraph::new();
        let (lhs_root, rhs_root) =
            merge_graphs(&mut new_graph, &self.graph, &rhs.graph).expect("Failed to merge graphs");

        let op_node = ComputationNode::BinaryOp { op: Operator::Mul };
        // Create the new operator node and set it as the root in one step.
        let op_idx = new_graph
            .add_root_node(op_node)
            .expect("Failed to add root node");

        // The old roots are now children of the new root.
        new_graph
            .add_edge(lhs_root, op_idx, ())
            .expect("Failed to add edge");
        new_graph
            .add_edge(rhs_root, op_idx, ())
            .expect("Failed to add edge");

        Self::from_graph(new_graph)
    }
}
