/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Resection-specific display helpers. Shared printing plumbing comes
//! from `causal_intervention_examples::print_utils`.

use crate::model_types::{COUPLING_STRENGTH, Connectome, DT, N_REGIONS, SeizureResult, TIME_STEPS};
use causal_intervention_examples::print_utils;
use deep_causality_core::{EffectValue, PropagatingEffect};

pub fn print_connectome_header() {
    println!(
        "Patient connectome: {} regions, hub-and-spoke (region 0 is the hub / putative focus).",
        N_REGIONS
    );
    println!("Kuramoto: coupling={COUPLING_STRENGTH}, dt={DT}, steps={TIME_STEPS}\n");
}

pub fn print_process(
    label: &str,
    effect: &PropagatingEffect<SeizureResult>,
    resected: Option<usize>,
) {
    let r = match &effect.value {
        EffectValue::Value(r) => r,
        _ => return println!("  {label}: <no value>"),
    };
    let status = if r.seizing { "SEIZURE" } else { "stable" };
    let curative = match resected {
        Some(_) if !r.seizing => "  <- CURATIVE",
        _ => "",
    };
    println!(
        "  {label}: final_sync={:.3}  status={status}{curative}",
        r.final_sync,
    );
}

pub fn print_resection_screening<F>(factual: &Connectome, mut runner: F)
where
    F: FnMut(Connectome, Connectome) -> PropagatingEffect<SeizureResult>,
{
    println!("--- Virtual resection screening ---");
    println!("Same chain, evaluated under do(connectome resected at region R) for each R:\n");
    for r in 0..N_REGIONS {
        let resected = factual.resected(r);
        let cf = runner(factual.clone(), resected);
        print_process(&format!("do(resect region {r:>2})"), &cf, Some(r));
    }
}

pub fn print_audit_trail<F>(factual: &Connectome, mut runner: F)
where
    F: FnMut(Connectome, Connectome) -> PropagatingEffect<SeizureResult>,
{
    println!(
        "\nAudit trail for the resection at region 0 (the seizure focus).\n\
         Each !!ValueAlternation!! entry below records which connectome was\n\
         substituted into the chain at the moment of intervention.\n"
    );
    let audit = runner(factual.clone(), factual.resected(0));
    print_utils::print_effect_log(&audit.logs);
}
