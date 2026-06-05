/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Stage functions and chain entry point for the aneurysm counterfactual chain.

use crate::model_types::{CRITICAL_WSS, CycleSummary, FloatType, N_CYCLES, RUPTURE_THRESHOLD};

/// Stage 2. Convert systolic BP into peak wall shear stress at the aneurysm dome.
///
/// Higher pressure feeds higher flow velocity, which feeds higher WSS, with
/// a non-linear amplification at the bulge. This is a deliberately simple
/// surrogate, not a CFD model. `CausalFlow::map` carries the value in and out,
/// so the stage reads as a plain `BP -> WSS` transform.
pub fn shear_stress_stage(systolic: FloatType) -> FloatType {
    // Aneurysm-dome amplified relationship. Tuned so a hypertensive patient
    // sits clearly above the critical threshold and a controlled patient
    // sits clearly below it.
    0.22 * (systolic - 80.0).max(0.0)
}

/// Stage 3. Accumulate wall fatigue across `N_CYCLES` cardiac cycles.
///
/// Per-cycle damage scales with how far WSS exceeds the critical
/// threshold. When fatigue reaches the rupture threshold, the cycle loop
/// terminates and the result records a rupture.
pub fn fatigue_stage(wss: FloatType) -> CycleSummary {
    let mut fatigue: FloatType = 0.0;
    let mut cycles_run = 0;
    let mut ruptured = false;

    for cycle in 1..=N_CYCLES {
        cycles_run = cycle;
        if wss > CRITICAL_WSS {
            fatigue += 0.04 * (wss / CRITICAL_WSS);
        } else {
            // Slow healing when stress is below threshold.
            fatigue = (fatigue - 0.005).max(0.0);
        }
        if fatigue >= RUPTURE_THRESHOLD {
            ruptured = true;
            break;
        }
    }

    CycleSummary {
        cycles_run,
        final_fatigue: fatigue.clamp(0.0, 1.0),
        ruptured,
        peak_wss: wss,
    }
}
