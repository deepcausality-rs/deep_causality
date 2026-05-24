/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Domain types and constants for the virtual-resection chain.

/// Switch this alias to `f32` for low precision, `f64` for standard precision,
/// or `Float106` for high precision. Literals in this crate would need wrapping
/// in `FloatType::from(…)` to switch away from `f64`.
pub type FloatType = f64;

pub const N_REGIONS: usize = 10;
pub const COUPLING_STRENGTH: FloatType = 3.5;
pub const TIME_STEPS: usize = 500;
pub const DT: FloatType = 0.1;
pub const SEIZURE_THRESHOLD: FloatType = 0.8;

/// A patient connectome: undirected adjacency list, one entry per region.
#[derive(Debug, Default, Clone)]
pub struct Connectome {
    pub adj: Vec<Vec<usize>>,
    pub intrinsic_freq: Vec<FloatType>,
    pub initial_phase: Vec<FloatType>,
}

#[derive(Debug, Default, Clone)]
pub struct SeizureResult {
    pub final_sync: FloatType,
    pub seizing: bool,
}
