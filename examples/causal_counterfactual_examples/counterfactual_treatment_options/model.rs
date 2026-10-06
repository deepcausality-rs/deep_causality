/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Stage functions and chain entry point for the aneurysm counterfactual chain.

use crate::model_types::{
    CRITICAL_WSS, CycleSummary, DAMAGE_RATE, FloatType, HEALING_RATE, N_CYCLES, PatientContext,
    RUPTURE_THRESHOLD, WSS_GAIN, WSS_ONSET_BP, read,
};
use deep_causality_core::CausalityError;

/// Stage 2. Convert systolic BP into peak wall shear stress at the aneurysm dome.
///
/// Higher pressure feeds higher flow velocity, which feeds higher WSS, with
/// a non-linear amplification at the bulge. This is a deliberately simple
/// surrogate, not a CFD model. `CausalFlow::try_step_with` carries the value in
/// and out and hands the stage the patient context, so the stage reads as a
/// `BP -> WSS` transform over the patient's surrogate parameters.
pub fn shear_stress_stage(
    systolic: FloatType,
    _state: &(),
    patient: Option<&PatientContext>,
) -> Result<FloatType, CausalityError> {
    let patient = patient.ok_or(CausalityError::MissingContext())?;
    // Aneurysm-dome amplified relationship.
    Ok(read(patient, WSS_GAIN)? * (systolic - read(patient, WSS_ONSET_BP)?).max(0.0))
}

/// Stage 3. Accumulate wall fatigue across `N_CYCLES` cardiac cycles.
///
/// Per-cycle damage scales with how far WSS exceeds the critical
/// threshold. When fatigue reaches the rupture threshold, the cycle loop
/// terminates and the result records a rupture. The thresholds and rates are
/// read from the patient context.
pub fn fatigue_stage(
    wss: FloatType,
    _state: &(),
    patient: Option<&PatientContext>,
) -> Result<CycleSummary, CausalityError> {
    let patient = patient.ok_or(CausalityError::MissingContext())?;
    let critical_wss = read(patient, CRITICAL_WSS)?;
    let damage_rate = read(patient, DAMAGE_RATE)?;
    let healing_rate = read(patient, HEALING_RATE)?;
    let rupture_threshold = read(patient, RUPTURE_THRESHOLD)?;

    let mut fatigue: FloatType = 0.0;
    let mut cycles_run = 0;
    let mut ruptured = false;

    for cycle in 1..=N_CYCLES {
        cycles_run = cycle;
        if wss > critical_wss {
            fatigue += damage_rate * (wss / critical_wss);
        } else {
            // Slow healing when stress is below threshold.
            fatigue = (fatigue - healing_rate).max(0.0);
        }
        if fatigue >= rupture_threshold {
            ruptured = true;
            break;
        }
    }

    Ok(CycleSummary {
        cycles_run,
        final_fatigue: fatigue.clamp(0.0, 1.0),
        ruptured,
        peak_wss: wss,
    })
}
