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
//!   because of the intervened pressure, not by direct edict; a
//!   beta-blocker reduces pressure, and the model's pressure-to-stress
//!   relationship carries the effect from there.
//! * **Surgical arm.** `intervene(wall_shear_stress = clipped)` fires
//!   between wall shear stress and fatigue accumulation. Pressure upstream
//!   is unchanged. A flow diverter or surgical clip directly attenuates
//!   wall stress, bypassing the pressure-to-stress edge entirely.
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

use deep_causality_core::{CausalFlow, PropagatingEffect};
use model::{fatigue_stage, shear_stress_stage};
use model_types::{CRITICAL_WSS, CycleSummary, FloatType};

fn main() {
    println!("=== Aneurysm Counterfactual Treatment Options ===\n");

    let patient_baseline_bp: FloatType = 175.0; // hypertensive patient
    println!("Patient baseline systolic BP: {patient_baseline_bp:.0} mmHg");
    println!(
        "Model: WSS = 0.22 * (BP - 80); damage/cycle = 0.04 * (WSS / {CRITICAL_WSS}) when WSS > {CRITICAL_WSS} Pa\n"
    );

    let f = run_factual(patient_baseline_bp);
    let m = run_medication_counterfactual(patient_baseline_bp, 120.0);
    let s = run_surgical_counterfactual(patient_baseline_bp, 8.0);

    model_utils::print_process("Factual (untreated)", &f);
    model_utils::print_process("Counterfactual A: do(BP = 120), beta-blocker", &m);
    model_utils::print_process("Counterfactual B: do(WSS = 8), surgical clip", &s);

    model_utils::print_audit_trail(&s);
}

fn run_factual(baseline_bp: FloatType) -> PropagatingEffect<CycleSummary> {
    CausalFlow::value(baseline_bp)
        .map(shear_stress_stage)
        .map(fatigue_stage)
        .into_effect()
}

fn run_medication_counterfactual(
    baseline_bp: FloatType,
    controlled_bp: FloatType,
) -> PropagatingEffect<CycleSummary> {
    CausalFlow::value(baseline_bp)
        .intervene(controlled_bp)
        .map(shear_stress_stage)
        .map(fatigue_stage)
        .into_effect()
}

fn run_surgical_counterfactual(
    baseline_bp: FloatType,
    clipped_wss: FloatType,
) -> PropagatingEffect<CycleSummary> {
    CausalFlow::value(baseline_bp)
        .map(shear_stress_stage)
        .intervene(clipped_wss)
        .map(fatigue_stage)
        .into_effect()
}
