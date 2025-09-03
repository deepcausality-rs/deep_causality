/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{ComputationNode, NodeId};

use std::marker::PhantomData;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

mod uncertain_bool;
mod uncertain_bool_default;
mod uncertain_f64;
mod uncertain_f64_default;
mod uncertain_op_arithmetic;
mod uncertain_op_comparison;
mod uncertain_op_logic;
mod uncertain_sampling;
mod uncertain_statistics;

// A single static counter for all Uncertain instances to generate unique IDs.
static NEXT_UNCERTAIN_ID: AtomicUsize = AtomicUsize::new(0);

/// A type representing a value with inherent uncertainty, modeled as a probability distribution.
#[derive(Clone, Debug, PartialEq)]
pub struct Uncertain<T> {
    id: usize,
    root_node: Arc<ComputationNode>,
    _phantom: PhantomData<T>,
}

impl<T> Uncertain<T> {
    /// Creates a new `Uncertain` value from a computation graph represented by a root node.
    fn from_root_node(root_node: ComputationNode) -> Self {
        Self {
            id: NEXT_UNCERTAIN_ID.fetch_add(1, Ordering::Relaxed),
            root_node: Arc::new(root_node),
            _phantom: PhantomData,
        }
    }

    pub fn conditional(condition: Uncertain<bool>, if_true: Self, if_false: Self) -> Self {
        Self::from_root_node(ComputationNode::ConditionalOp {
            node_id: NodeId::new(), // Added node_id
            condition: Box::new((*condition.root_node).clone()),
            if_true: Box::new((*if_true.root_node).clone()),
            if_false: Box::new((*if_false.root_node).clone()),
        })
    }
}

impl<T: Copy> Uncertain<T> {
    pub fn id(&self) -> usize {
        self.id
    }
}
