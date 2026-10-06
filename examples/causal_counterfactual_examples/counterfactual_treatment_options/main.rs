/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Counterfactual Treatment Options (Aneurysm Hemodynamics)
//!
//! Two interventions, two different sites, one causal chain. The factual
//! chain is an untreated patient over 30 cardiac cycles. Each
//! counterfactual reuses the same chain with a treatment effect injected
//! at a different upstream variable:
//!
//! * **Medication arm.** `intervene(systolic_bp = 120)` fires between
//!   blood pressure and wall shear stress. The downstream stress falls
//!   because of the alternate pressure, not by direct edict; a
//!   beta-blocker reduces pressure, and the model's pressure-to-stress
//!   relationship carries the effect from there.
//! * **Surgical arm.** `intervene(wall_shear_stress = clipped)` fires
//!   between wall shear stress and fatigue accumulation. Pressure upstream
//!   is unchanged. A flow diverter or surgical clip directly attenuates
//!   wall stress, bypassing the pressure-to-stress edge entirely.
//!
//! The patient is the chain's context: baseline blood pressure, the wall
//! shear stress surrogate's parameters, and the wall's fatigue thresholds and
//! rates. Every run carries the same patient; the interventions act on the
//! value channel only.
//!
//! ## The lesson: intervention site is a causal claim
//!
//! Both arms reduce rupture probability. They reduce it for different
//! reasons, and the reasons are recoverable from the chain. Where you
//! intervene says what is upstream of the manipulated quantity:
//!
//! * The medication intervention encodes the claim *reducing BP causes
//!   reduced wall stress causes reduced fatigue*.
//! * The surgical intervention encodes the claim *we modified wall stress
//!   directly; upstream pressure is irrelevant to the surgical effect*.
//!
//! Two clinical recommendations, one model, two intervention sites. Chain
//! identity guarantees that the only difference between the runs is where
//! the intervention happened.

mod model;
pub mod model_types;
mod model_utils;

use deep_causality_core::{CausalFlow, CausalityError};
use model::{fatigue_stage, shear_stress_stage};
use model_types::{
    BASELINE_BP, CRITICAL_WSS, CycleSummary, DAMAGE_RATE, FloatType, PatientContext,
    PatientProcess, WSS_GAIN, WSS_ONSET_BP, patient_context, read,
};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("=== Aneurysm Counterfactual Treatment Options ===\n");

    let patient = patient_context()?;
    let critical_wss = read(&patient, CRITICAL_WSS)?;
    println!(
        "Patient baseline systolic BP: {:.0} mmHg",
        read(&patient, BASELINE_BP)?
    );
    println!(
        "Model: WSS = {} * (BP - {}); damage/cycle = {} * (WSS / {critical_wss}) when WSS > {critical_wss} Pa\n",
        read(&patient, WSS_GAIN)?,
        read(&patient, WSS_ONSET_BP)?,
        read(&patient, DAMAGE_RATE)?,
    );

    let f = run_factual(patient.clone())?;
    let m = run_medication_counterfactual(patient.clone(), 120.0)?;
    let s = run_surgical_counterfactual(patient, 8.0)?;

    model_utils::print_process("Factual (untreated)", &f);
    model_utils::print_process("Counterfactual A: do(BP = 120), beta-blocker", &m);
    model_utils::print_process("Counterfactual B: do(WSS = 8), surgical clip", &s);

    model_utils::print_audit_trail(&s);
    Ok(())
}

/// Seed the chain with the patient's baseline systolic BP and carry the
/// patient as its context.
fn start(
    patient: PatientContext,
) -> Result<CausalFlow<FloatType, (), PatientContext>, CausalityError> {
    Ok(CausalFlow::value(read(&patient, BASELINE_BP)?).context(patient))
}

fn run_factual(patient: PatientContext) -> Result<PatientProcess<CycleSummary>, CausalityError> {
    Ok(start(patient)?
        .try_step_with(shear_stress_stage)
        .try_step_with(fatigue_stage)
        .into_process())
}

fn run_medication_counterfactual(
    patient: PatientContext,
    controlled_bp: FloatType,
) -> Result<PatientProcess<CycleSummary>, CausalityError> {
    Ok(start(patient)?
        .alternate_value(controlled_bp)
        .try_step_with(shear_stress_stage)
        .try_step_with(fatigue_stage)
        .into_process())
}

fn run_surgical_counterfactual(
    patient: PatientContext,
    clipped_wss: FloatType,
) -> Result<PatientProcess<CycleSummary>, CausalityError> {
    Ok(start(patient)?
        .try_step_with(shear_stress_stage)
        .alternate_value(clipped_wss)
        .try_step_with(fatigue_stage)
        .into_process())
}
