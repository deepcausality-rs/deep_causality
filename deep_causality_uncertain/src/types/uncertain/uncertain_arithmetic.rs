/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::computation::ArithmeticOperator;
use crate::{ComputationNode, Uncertain, merge_graphs};
use std::ops::{Add, Div, Mul, Sub};
use ultragraph::{GraphMut, UltraGraph};

// Operator overloading is only implemented for f64 for now.
impl Add for Uncertain<f64> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut new_graph = UltraGraph::new();
        let (lhs_root, rhs_root) = merge_graphs::<f64>(&mut new_graph, &self.graph, &rhs.graph)
            .expect("Failed to merge graphs");

        let op_node = ComputationNode::ArithmeticOp {
            op: ArithmeticOperator::Add,
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

        Self::from_graph(new_graph)
    }
}

impl Sub for Uncertain<f64> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let mut new_graph = UltraGraph::new();
        let (lhs_root, rhs_root) = merge_graphs::<f64>(&mut new_graph, &self.graph, &rhs.graph)
            .expect("Failed to merge graphs");

        let op_node = ComputationNode::ArithmeticOp {
            op: ArithmeticOperator::Sub,
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

        Self::from_graph(new_graph)
    }
}

impl Mul for Uncertain<f64> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let mut new_graph = UltraGraph::new();
        let (lhs_root, rhs_root) = merge_graphs::<f64>(&mut new_graph, &self.graph, &rhs.graph)
            .expect("Failed to merge graphs");

        let op_node = ComputationNode::ArithmeticOp {
            op: ArithmeticOperator::Mul,
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

        Self::from_graph(new_graph)
    }
}

impl Div for Uncertain<f64> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        let mut new_graph = UltraGraph::new();
        let (lhs_root, rhs_root) = merge_graphs::<f64>(&mut new_graph, &self.graph, &rhs.graph)
            .expect("Failed to merge graphs");

        let op_node = ComputationNode::ArithmeticOp {
            op: ArithmeticOperator::Div,
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

        Self::from_graph(new_graph)
    }
}
