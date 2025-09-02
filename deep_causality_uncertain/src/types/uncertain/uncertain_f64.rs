/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{
    ComputationNode, DistributionEnum, NodeId, NormalDistributionParams, Uncertain,
    UniformDistributionParams,
};
use std::sync::Arc;

// Constructors
impl Uncertain<f64> {
    pub fn point(value: f64) -> Self {
        Self::from_root_node(ComputationNode::LeafF64 {
            node_id: NodeId::new(), // Added node_id
            dist: DistributionEnum::Point(value),
        })
    }

    pub fn normal(mean: f64, std_dev: f64) -> Self {
        let params = NormalDistributionParams { mean, std_dev };
        Self::from_root_node(ComputationNode::LeafF64 {
            node_id: NodeId::new(), // Added node_id
            dist: DistributionEnum::Normal(params),
        })
    }

    pub fn uniform(low: f64, high: f64) -> Self {
        let params = UniformDistributionParams { low, high };
        Self::from_root_node(ComputationNode::LeafF64 {
            node_id: NodeId::new(), // Added node_id
            dist: DistributionEnum::Uniform(params),
        })
    }

    pub fn map<F>(&self, func: F) -> Self
    where
        F: Fn(f64) -> f64 + Send + Sync + 'static,
    {
        Self::from_root_node(ComputationNode::FunctionOp {
            node_id: NodeId::new(), // Added node_id
            func: Arc::new(func),
            operand: Box::new((*self.root_node).clone()),
        })
    }

    pub fn map_to_bool<F>(&self, func: F) -> Uncertain<bool>
    where
        F: Fn(f64) -> bool + Send + Sync + 'static,
    {
        Uncertain::from_root_node(ComputationNode::FunctionOpBool {
            node_id: NodeId::new(), // Added node_id
            func: Arc::new(func),
            operand: Box::new((*self.root_node).clone()),
        })
    }
}
