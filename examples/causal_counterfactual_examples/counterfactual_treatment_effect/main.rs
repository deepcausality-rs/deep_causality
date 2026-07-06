/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # CATE as `do(T=1) − do(T=0)` on the Same Chain
//!
//! The Conditional Average Treatment Effect is, by definition, a
//! difference between two interventional worlds. This example writes that
//! definition out in code.
//!
//! ## The estimand
//!
//! For a subgroup `S`:
//!
//! ```text
//! CATE(S) = E[ Y(do(T=1)) − Y(do(T=0)) | X ∈ S ]
//! ```
//!
//! Each patient's two potential outcomes come from the same data-generating
//! process under two different `do`-operators. One `PropagatingEffect`
//! chain models the blood-pressure response to an (unknown) treatment
//! indicator; `.alternate_value(1.0)` and `.alternate_value(0.0)` produce the two
//! potential outcomes. Per-patient differences are individual treatment
//! effects; their subgroup mean is the CATE.
//!
//! ## What `intervene` adds over `bind`
//!
//! Without it, you would build two separate computations, one per arm, and
//! trust by reading them that they are the same model on different inputs.
//! With it, one chain runs twice. Any difference in the output is
//! attributable to the alternate value alone, because nothing else
//! changed between the two runs.

mod model;
mod model_utils;

use model::synthetic_cohort;

fn main() {
    println!("=== CATE as `do(T=1) − do(T=0)` on a Single Causal Chain ===\n");

    let cohort = synthetic_cohort();
    println!("Cohort: {} patients\n", cohort.len());

    let (all, over_65, under_65) = model_utils::evaluate_and_print_cohort(&cohort);
    model_utils::print_cate_summary(&all, &over_65, &under_65);
    model_utils::print_audit_trail(&cohort[0]);
}
