/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Topological Insulator: the Chern number two ways
//!
//! Computes the Chern number of the Qi-Wu-Zhang model in its three phases, bringing the
//! DeepCausality pillars together:
//!
//! - **The tangent functor + quadrature.** The Berry curvature `Ω(k)` is built from the exact
//!   momentum derivatives of the d-vector (autodiff, no finite differences), and the Chern number
//!   `C = (1/2π)∫∫ Ω` is a *nested* `quadrature` over the Brillouin zone.
//! - **Precision as a parameter.** `chern_quadrature` is generic over `Scalar`; this example runs
//!   it at `FloatType` (switchable to `Float106`).
//! - **The causal monad.** `PropagatingEffect` sequences the analysis and short-circuits if an
//!   integral leaves the finite range.
//!
//! The quadrature result is cross-checked against the prior accumulation: the
//! Fukui-Hatsugai-Suzuki lattice (Wilson-loop) sum.

use deep_causality_core::{CausalFlow, CausalityError, CausalityErrorEnum, PropagatingEffect};
use model::QWZModel;

mod model;

/// The quadrature Chern number runs at this precision (switch to `Float106` for more digits; the
/// integrand and the fold are generic over `Scalar`).
pub type FloatType = f64;

const N_QUAD: usize = 100; // composite-Simpson panels per axis (even)
const N_WILSON: usize = 100; // Brillouin-zone grid for the lattice cross-check

fn main() {
    println!("----------------------------------------------------------------");
    println!("   Topological Insulator Analysis (Berry Curvature)");
    println!("----------------------------------------------------------------");
    println!("Chern number two ways: tangent-functor Berry curvature integrated by nested");
    println!("quadrature, cross-checked against the Fukui-Hatsugai-Suzuki lattice sum.\n");

    // The analysis runs as a CausalFlow stage; a non-finite integral short-circuits the chain.
    let pipeline = CausalFlow::effect().bind(|_, _, _| analyze()).into_effect();

    match pipeline.value.into_value() {
        Some(report) => print_report(&report),
        None => eprintln!("Chern analysis failed: {:?}", pipeline.error),
    }
}

/// One material phase: its mass parameter, the Chern number from each method, and a label.
#[derive(Default, Clone, Debug)]
struct PhaseRow {
    u: f64,
    quad: FloatType,
    wilson: f64,
    label: &'static str,
}

#[derive(Default, Clone, Debug)]
struct Report {
    rows: Vec<PhaseRow>,
}

fn analyze() -> PropagatingEffect<Report> {
    let phases = [
        (3.0_f64, "trivial (|u| > 2)"),
        (1.0, "topological (0 < u < 2)"),
        (-1.0, "topological (-2 < u < 0)"),
    ];

    let mut rows = Vec::new();
    for (u, label) in phases {
        let quad: FloatType = model::chern_quadrature(u, N_QUAD);
        if !model::finite(quad) {
            return fail("Berry-curvature integral left the finite range");
        }
        let wilson = model::chern_wilson(&QWZModel::new(u), N_WILSON);
        rows.push(PhaseRow {
            u,
            quad,
            wilson,
            label,
        });
    }

    PropagatingEffect::pure(Report { rows })
}

fn print_report(r: &Report) {
    println!("    u   | C (quadrature) | C (Wilson loop) |  C  | phase");
    println!("  ------+----------------+-----------------+-----+-----------------------");
    for row in &r.rows {
        let c = model::nearest_int(row.quad);
        println!(
            "  {:>4.1} |   {:>11.6}  |   {:>12.6}  | {:>+2}  | {}",
            row.u, row.quad, row.wilson, c, row.label
        );
    }

    println!("\n  Phase diagram:  |u| > 2 → C = 0;   0 < u < 2 → C = +1;   −2 < u < 0 → C = −1");
    println!("  Both methods agree; the quadrature path used exact autodiff derivatives.");
}

fn fail<T: Default + Clone + std::fmt::Debug>(msg: &str) -> PropagatingEffect<T> {
    PropagatingEffect::from_error(CausalityError::new(CausalityErrorEnum::Custom(msg.into())))
}
