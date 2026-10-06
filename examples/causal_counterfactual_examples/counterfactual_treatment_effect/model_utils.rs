/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! CATE-specific helpers. Shared printing and arithmetic plumbing
//! come from `causal_counterfactual_examples::{print_utils, math_utils}`.

use crate::model::{
    AGE, BASELINE_BP, FloatType, PatientContext, evaluate_under, potential_outcomes, read,
};
use causal_counterfactual_examples::{math_utils, print_utils};
use deep_causality_context::Identifiable;
use deep_causality_core::CausalityError;

/// The ITE vectors of the whole cohort, the over-65 stratum and the 65-and-under stratum.
type StratifiedIte = (Vec<FloatType>, Vec<FloatType>, Vec<FloatType>);

/// Iterate the cohort, compute potential outcomes per patient, print
/// the per-patient row, and return the ITE vectors split by age stratum.
pub fn evaluate_and_print_cohort(
    cohort: &[PatientContext],
) -> Result<StratifiedIte, CausalityError> {
    let mut all_ites: Vec<FloatType> = Vec::new();
    let mut over_65_ites: Vec<FloatType> = Vec::new();
    let mut under_65_ites: Vec<FloatType> = Vec::new();

    println!("Per-patient potential outcomes (post-treatment systolic BP):");
    println!("  id |  age | baseline |  Y(do=1)  |  Y(do=0)  |    ITE");
    println!("  ---+------+----------+-----------+-----------+--------");
    for patient in cohort {
        let (y1, y0) = potential_outcomes(patient)?;
        let ite = y1 - y0;
        let age = read(patient, AGE)?;

        all_ites.push(ite);
        if age > 65.0 {
            over_65_ites.push(ite);
        } else {
            under_65_ites.push(ite);
        }

        println!(
            "   {:>2} | {:>4.0} |  {:>6.1}  |  {:>6.1}   |  {:>6.1}   |  {:>+5.2}",
            patient.id(),
            age,
            read(patient, BASELINE_BP)?,
            y1,
            y0,
            ite,
        );
    }

    Ok((all_ites, over_65_ites, under_65_ites))
}

pub fn print_cate_summary(all: &[FloatType], over_65: &[FloatType], under_65: &[FloatType]) {
    let cate_all = math_utils::mean(all);
    let cate_over = math_utils::mean(over_65);
    let cate_under = math_utils::mean(under_65);

    println!("\n=== CATE results ===");
    println!("  Overall ATE                    : {cate_all:>+6.2} mmHg");
    println!(
        "  CATE | age >  65 ({:>2} patients) : {cate_over:>+6.2} mmHg",
        over_65.len()
    );
    println!(
        "  CATE | age <= 65 ({:>2} patients) : {cate_under:>+6.2} mmHg",
        under_65.len()
    );
    println!(
        "  Treatment-effect heterogeneity : {:>+6.2} mmHg",
        cate_over - cate_under
    );
    println!(
        "\nThe non-zero heterogeneity comes from the model itself: older\n\
         patients have a stronger drug response. Because both arms ran on\n\
         the same chain with only the intervened treatment value differing,\n\
         that difference is attributable to the data-generating process and\n\
         not to any divergence between two separately-built computations.\n"
    );
}

pub fn print_audit_trail(patient: &PatientContext) {
    println!("--- Audit trail for patient #1 (under intervention do(T=1)) ---");
    let one = evaluate_under(patient, 1.0);
    print_utils::print_effect_log(one.logs());
}
