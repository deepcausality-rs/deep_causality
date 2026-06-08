/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Domain types and constants for the aneurysm counterfactual chain.

/// Switch this alias to `f32` for low precision, `f64` for standard precision,
/// or `Float106` for high precision. Literals in this crate would need wrapping
/// in `FloatType::from(…)` to switch away from `f64`.
pub type FloatType = f64;

pub const CRITICAL_WSS: FloatType = 15.0; // Pa
pub const RUPTURE_THRESHOLD: FloatType = 0.75; // fatigue clamped to [0, 1]
pub const N_CYCLES: u32 = 30;

#[derive(Debug, Clone, Default)]
pub struct CycleSummary {
    pub cycles_run: u32,
    pub final_fatigue: FloatType,
    pub ruptured: bool,
    pub peak_wss: FloatType,
}
