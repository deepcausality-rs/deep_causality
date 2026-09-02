/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Shots from the Born rule, on the `DensityMatrix` carrier, in the default build.
//!
//! `QpuSampler::sample` takes a `QuantumCircuit`, so a plant carried as a density matrix reaches
//! no shipped sampler. This is the draw from a probability to a count: the probability comes from
//! `born_projective_probability`, once, at the pipeline's scalar, and the Born rule is not
//! restated here.

use crate::QuantumError;
use crate::types::density_matrix::DensityMatrix;
use crate::types::qpu::histogram::CountHistogram;
use crate::types::qpu::prng::SplitMix64;
use crate::types::verdict::born::born_projective_probability;
use crate::types::verdict::projection::Projection;
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;

/// `shots` Bernoulli draws at `Tr(Pρ)`, seeded, as a one-bit histogram: outcome `1` is the
/// projector accepting, `0` rejecting.
///
/// The comparison `u < p` happens at `R`, with the uniform lifted through `R::from_f64`, so the
/// probability is never narrowed to `f64` on the way to a count. A fixed seed reproduces the
/// histogram exactly.
///
/// # Errors
///
/// [`QuantumError::DimensionMismatch`] from the Born call when the state's dimension differs from
/// `D`, in which case nothing is drawn; [`QuantumError::CalculationError`] if `R` cannot represent
/// a uniform sample, which no shipped real scalar triggers.
pub fn sample_projector<R, const D: usize>(
    rho: &DensityMatrix<R>,
    projection: &Projection<R, D>,
    shots: u64,
    seed: u64,
) -> Result<CountHistogram, QuantumError>
where
    R: RealField + FromPrimitive + Default + core::fmt::Debug,
{
    let p = born_projective_probability(rho, projection)?;
    let mut rng = SplitMix64::new(seed);
    let mut hist = CountHistogram::new(1);
    for _ in 0..shots {
        let u = R::from_f64(rng.next_f64()).ok_or_else(|| {
            QuantumError::CalculationError("scalar cannot represent a uniform sample".into())
        })?;
        hist.record(usize::from(u < p));
    }
    Ok(hist)
}
