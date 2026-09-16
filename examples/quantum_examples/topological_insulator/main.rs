/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # A topological insulator: the Chern number, two independent ways
//!
//! Some insulators cannot be turned into others without closing their gap. What separates them is
//! not a local property but an integer, the **Chern number**, obtained by integrating the Berry
//! curvature over the whole Brillouin zone:
//!
//! ```text
//! C = (1 / 2π) ∫∫ Ω(kx, ky) dkx dky
//! ```
//!
//! It comes out an integer however the material is deformed, and it changes only when the gap
//! closes. That is what "topological" means here, and it is why the quantised Hall conductance of
//! such a material is insensitive to disorder.
//!
//! The model is Qi-Wu-Zhang, `H(k) = d(k)·σ` with
//!
//! ```text
//! d(k) = (sin kx,  sin ky,  u + cos kx + cos ky)
//! ```
//!
//! whose phase depends on the single mass parameter `u`.
//!
//! # Two routes, sharing nothing but the d-vector
//!
//! ```text
//! quadrature   Ω from exact ∂d/∂k, integrated by nested composite Simpson
//! Wilson loop  Berry flux Im ln W around each plaquette of a k-grid, summed
//! ```
//!
//! The first differentiates and never forms a spinor. The second forms spinors and never
//! differentiates. They can only agree by both being right, which is what makes printing them side
//! by side worth the second calculation.
//!
//! # What the run does
//!
//! ```text
//! fmap       phase → its two Chern numbers
//! sequence   a tensor of fallible phases → one fallible tensor
//! bind       the analysis as a stage that short-circuits on a non-finite integral
//! ```
//!
//! The derivatives come from the tangent functor, so there are no finite differences anywhere and
//! no step size to tune. `DComponent` carries no numbers at all: the mass rides in with the
//! momenta, so every quantity in the model is at the precision the caller asked for rather than
//! the one the model was written down in.

mod model;
mod utils_print;

use deep_causality_core::{CausalFlow, CausalityError, CausalityErrorEnum, PropagatingEffect};
use deep_causality_haft::ResultWitness;
use deep_causality_haft::{Functor, Traversable};
use deep_causality_num::Float106;
use deep_causality_tensor::{CausalTensor, CausalTensorWitness};
use model::{N_QUADRATURE, N_WILSON, ONE, THREE, ZERO, chern_quadrature, chern_wilson, finite};
use utils_print::{Phase, print_header, print_report};

/// The working scalar. Switch it to `f32`, `f64` or `deep_causality_num::BFloat16`; the d-vector,
/// its exact derivatives, both integrals and the mass parameter all recompute at that precision.
///
/// It sits at [`Float106`] by default on purpose. A hard-coded `f64` anywhere in the program is
/// invisible while the alias *is* `f64`, and shows up here as a compile error the moment the two
/// types differ.
pub type FloatType = Float106;

fn main() {
    print_header();

    // The analysis runs as a `CausalFlow` stage, so an integral that leaves the finite range
    // short-circuits the chain rather than printing a number nobody should read.
    let pipeline = CausalFlow::effect().bind(|_, _, _| analyze()).into_effect();

    match pipeline.value_cloned() {
        Some(report) => print_report(&report),
        None => eprintln!("Chern analysis failed: {:?}", pipeline.error()),
    }
}

/// The three phases of the model, by mass parameter.
///
/// `|u| > 2` is trivial and the two windows either side of zero are topological, with opposite
/// signs. The labels say what the phase diagram predicts, and the table says what came out.
fn phases() -> [(FloatType, &'static str); 3] {
    [
        (THREE, "trivial (|u| > 2)"),
        (ONE, "topological (0 < u < 2)"),
        (-ONE, "topological (-2 < u < 0)"),
    ]
}

fn analyze() -> PropagatingEffect<Vec<Phase>> {
    let entries = phases();

    let Ok(tensor) = CausalTensor::new(entries.to_vec(), vec![entries.len()]) else {
        return fail("the phase list could not be formed");
    };

    // fmap: each phase is integrated twice, and either integral may leave the finite range.
    let attempted = CausalTensorWitness::fmap(tensor, |(u, label)| {
        let quadrature = chern_quadrature::<FloatType>(u, N_QUADRATURE);
        if !finite(quadrature) {
            return Err(format!(
                "the Berry-curvature integral at u = {label} is not finite"
            ));
        }

        let wilson = chern_wilson::<FloatType>(u, N_WILSON);
        if !finite(wilson) {
            return Err(format!("the Wilson-loop sum at u = {label} is not finite"));
        }

        Ok(Phase {
            u,
            quadrature,
            wilson,
            label,
        })
    });

    // sequence: one failure anywhere collapses the whole analysis, so the report either holds
    // every phase or the chain carries the reason it holds none.
    match CausalTensorWitness::sequence::<_, ResultWitness<String>>(attempted) {
        Ok(rows) => PropagatingEffect::pure(rows.into_vec()),
        Err(reason) => fail(&reason),
    }
}

/// The agreement between the two routes, as the largest gap across the phases.
pub fn largest_disagreement(rows: &[Phase]) -> FloatType {
    rows.iter().fold(ZERO, |worst, row| {
        let gap = magnitude(row.quadrature - row.wilson);

        if gap > worst { gap } else { worst }
    })
}

/// How far the quadrature result sits from the nearest integer, which is what quantisation means
/// in practice: nothing rounds, and an integer is what the integral returns.
pub fn largest_departure_from_integer(rows: &[Phase]) -> FloatType {
    rows.iter().fold(ZERO, |worst, row| {
        let nearest =
            model::integer_scalar::<FloatType>(model::nearest_chern_number(row.quadrature));
        let gap = magnitude(row.quadrature - nearest);

        if gap > worst { gap } else { worst }
    })
}

fn magnitude(x: FloatType) -> FloatType {
    if x < ZERO { -x } else { x }
}

fn fail<T: Default + Clone + std::fmt::Debug>(msg: &str) -> PropagatingEffect<T> {
    PropagatingEffect::from_error(CausalityError::new(CausalityErrorEnum::Custom(msg.into())))
}
