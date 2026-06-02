/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Configuration and result types for the BRCD driver.

use crate::brcd::brcd_gate::GateConfig;
use crate::brcd::brcd_gaussian::{RIDGE_DEFAULT, Transform};
use deep_causality_num::{FromPrimitive, RealField};

/// The likelihood family BRCD scores each node with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FamilyKind {
    /// Plug-in ridge-Gaussian continuous family.
    Continuous,
    /// Dirichlet posterior-predictive discrete family.
    Discrete,
}

/// Configuration for a BRCD run.
#[derive(Debug, Clone)]
pub struct BrcdConfig<T> {
    /// Seed for the uniform MEC DAG sampling (reproducible runs).
    pub seed: u64,
    /// Continuous or discrete likelihood family.
    pub family: FamilyKind,
    /// Node transform (continuous family only).
    pub node_transform: Transform,
    /// Apply the node transform to continuous parents (no Jacobian).
    pub transform_parents: bool,
    /// Number of simultaneous root causes per candidate set, `k`.
    pub num_root_causes: usize,
    /// Ridge `λ` for the conditional-mean fits (continuous family).
    pub ridge: T,
    /// Dirichlet concentration `α*` (discrete family).
    pub alpha_star: T,
    /// Logistic-gate configuration (continuous mixture; unused by `brcd_update`,
    /// which only hits the per-regime and single-expert branches).
    pub gate: GateConfig<T>,
}

impl<T: RealField + FromPrimitive> BrcdConfig<T> {
    /// A continuous-family configuration with the given seed and `k = 1`.
    pub fn continuous(seed: u64) -> Self {
        Self {
            seed,
            family: FamilyKind::Continuous,
            node_transform: Transform::None,
            transform_parents: false,
            num_root_causes: 1,
            ridge: from_f64(RIDGE_DEFAULT),
            alpha_star: from_f64(5.0),
            gate: GateConfig::default(),
        }
    }

    /// A discrete-family configuration with the given seed and `k = 1`.
    pub fn discrete(seed: u64) -> Self {
        Self {
            family: FamilyKind::Discrete,
            ..Self::continuous(seed)
        }
    }
}

impl<T: RealField + FromPrimitive> Default for BrcdConfig<T> {
    fn default() -> Self {
        Self::continuous(0)
    }
}

fn from_f64<T: FromPrimitive>(x: f64) -> T {
    <T as FromPrimitive>::from_f64(x).expect("constant is representable in every RealField")
}
