/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # A hyperbolic metamaterial lens, driven by its metric signature
//!
//! An ordinary lens cannot resolve detail finer than the light it uses. The detail is carried by
//! high spatial frequencies, and in vacuum a spatial frequency above `k₀ = 2π/λ` has an imaginary
//! out-of-plane wavenumber: the wave decays instead of travelling, so the detail never reaches the
//! far field. That is the diffraction limit.
//!
//! A **hyperbolic metamaterial** is built so that one principal permittivity is negative while the
//! others are positive. That single sign flip turns the dispersion surface from a closed sphere,
//! which bounds `k_x`, into an open hyperboloid, which does not. Arbitrarily fine detail
//! propagates.
//!
//! # The metric *is* the material
//!
//! A sign pattern over principal axes is exactly what a [`Metric`] signature carries, so the two
//! materials here are two metrics and nothing else:
//!
//! ```text
//! vacuum                    Euclidean(3)         (+, +, +)
//! Type I metamaterial       Generic{p:2, q:1}    (+, +, −)
//! ```
//!
//! [`model::permittivity`] reads each sign with `Metric::sign_of_sq`, so the optics never names a
//! sign of its own. Swapping the metric swaps the physics, which is the claim this example exists
//! to make.
//!
//! # What the run does
//!
//! Two categorical operations sweep a range of object periods:
//!
//! ```text
//! fmap   period → k_z²               the dispersion relation, one period at a time
//! fold   k_z² over all periods → the finest period that still propagates
//! ```
//!
//! `fmap` applies the dispersion relation pointwise, and `fold` reduces the sweep to the
//! resolution limit each material imposes.

mod model;
mod utils_print;

use deep_causality_haft::{Foldable, Functor};
use deep_causality_num::Float106;
use deep_causality_tensor::{CausalTensor, CausalTensorError, CausalTensorWitness};
use model::{
    SEPARATIONS_NM, ZERO, hyperbolic_metamaterial, squared_out_of_plane_wavenumber, vacuum,
};
use utils_print::{print_header, print_materials, print_sweep, print_verdict};

/// The working scalar. Switch it to `f32`, `f64` or `deep_causality_num::BFloat16`; the
/// wavenumbers, the dispersion relation and the resolution limits all recompute at that precision.
///
/// It sits at [`Float106`] by default on purpose. A hard-coded `f64` anywhere in the program is
/// invisible while the alias *is* `f64`, and shows up here as a compile error the moment the two
/// types differ.
pub type FloatType = Float106;

fn main() -> Result<(), CausalTensorError> {
    print_header();
    print_materials(&vacuum(), &hyperbolic_metamaterial());

    // The periods to probe, coarse to fine, as a rank-1 tensor.
    let periods = CausalTensor::new(SEPARATIONS_NM.to_vec(), vec![SEPARATIONS_NM.len()])?;

    // fmap: the dispersion relation reads one period and returns one k_z². The law is pointwise,
    // so the functor carries it across the sweep without the sweep appearing in the law.
    let in_vacuum = CausalTensorWitness::fmap(periods.clone(), |d| {
        squared_out_of_plane_wavenumber(&vacuum(), d)
    });
    let in_lens = CausalTensorWitness::fmap(periods.clone(), |d| {
        squared_out_of_plane_wavenumber(&hyperbolic_metamaterial(), d)
    });

    print_sweep(&periods, &in_vacuum, &in_lens);

    // fold: the finest period whose wave still propagates. A period contributes only when its
    // k_z² is non-negative, so the reduction reports the limit rather than the last entry.
    let vacuum_limit = finest_propagating(&periods, &in_vacuum);
    let lens_limit = finest_propagating(&periods, &in_lens);

    print_verdict(vacuum_limit, lens_limit);
    Ok(())
}

/// The finest period that still propagates, or `None` when every period decays.
///
/// `fold` walks the paired sweep and keeps the smallest period whose `k_z²` stayed non-negative.
/// Pairing the period with its own result is what lets one reduction answer in nanometres.
fn finest_propagating(
    periods: &CausalTensor<FloatType>,
    squared: &CausalTensor<FloatType>,
) -> Option<FloatType> {
    let paired: Vec<(FloatType, FloatType)> = periods
        .as_slice()
        .iter()
        .zip(squared.as_slice())
        .map(|(&d, &k)| (d, k))
        .collect();

    CausalTensorWitness::fold(
        CausalTensor::from_vec(paired, &[periods.len()]),
        None,
        |finest, (period, k_z_squared)| {
            if k_z_squared < ZERO {
                return finest;
            }
            match finest {
                Some(best) if best <= period => Some(best),
                _ => Some(period),
            }
        },
    )
}
