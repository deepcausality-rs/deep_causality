/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Domain types for the cascading failure chain.

#![allow(dead_code)] // Network struct fields retained for readability of the model.

use deep_causality_core::PropagatingProcess;
use std::collections::HashSet;

/// Switch this alias to `f32` for low precision, `f64` for standard precision,
/// or `Float106` for high precision. Literals in this crate would need wrapping
/// in `FloatType::from(…)` to switch away from `f64`.
pub type FloatType = f64;

pub const CASCADE_MAX_ITERATIONS: u32 = 20;

/// A directed flow edge from `from` to `to`, with a maximum capacity (units
/// of flow per second). Real-valued so the solver can produce fractional
/// flows on parallel paths.
#[derive(Debug, Clone)]
pub struct Edge {
    pub id: u32,
    pub from: u32,
    pub to: u32,
    pub capacity: FloatType,
}

/// Network configuration: immutable topology + capacities. Carried in the
/// `Context` channel of the `PropagatingProcess`.
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub n_nodes: u32,
    pub edges: Vec<Edge>,
    pub source: u32,
    pub sink: u32,
    pub source_supply: FloatType,
}

/// State accumulated across cascade iterations. Carried in the `State`
/// channel of the `PropagatingProcess`.
#[derive(Debug, Default, Clone)]
pub struct NetworkState {
    pub failed_edges: HashSet<u32>,
    pub cascade_step: u32,
}

/// Value channel. The result of the most recent flow solve.
#[derive(Debug, Default, Clone)]
pub struct FlowSolution {
    /// flows[edge_id] = current flow on that edge (0.0 if failed)
    pub flows: Vec<FloatType>,
    /// edges whose flow exceeds their capacity in this solve
    pub overloaded: Vec<u32>,
    /// total flow delivered to the sink
    pub delivered: FloatType,
}

pub type NetworkProcess<T> = PropagatingProcess<T, NetworkState, NetworkConfig>;
