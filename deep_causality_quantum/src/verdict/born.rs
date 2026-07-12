/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Verdict extraction at the measurement boundary (spec
//! quantum-verdict-orthomodular, scenario "Verdicts are extracted at the
//! measurement boundary"). A quantum causaloid does **not** carry a `Verdict`
//! over its operators; instead a verdict is read out from a state and a
//! measurement projection — the Born rule `Tr(Pρ)` for a probability, or the
//! projection itself for a proposition in the orthomodular lattice.

use crate::QuantumError;
use crate::types::density_matrix::DensityMatrix;
use crate::verdict::projection::Projection;
use deep_causality_algebra::{Prob, RealField};
use deep_causality_num::FromPrimitive;

/// The Born-rule probability of the measurement outcome `projection` on state
/// `rho`: `Tr(P ρ)`, clamped to `[0, 1]`. Real for a Hermitian `P` and `ρ`; a
/// non-negligible imaginary part signals a malformed input and errors.
pub fn born_projective_probability<R, const D: usize>(
    rho: &DensityMatrix<R>,
    projection: &Projection<R, D>,
) -> Result<R, QuantumError>
where
    R: RealField + FromPrimitive + Default + core::fmt::Debug,
{
    if rho.dim() != D {
        return Err(QuantumError::DimensionMismatch(format!(
            "state dimension {} ≠ projection dimension {}",
            rho.dim(),
            D
        )));
    }
    let p = projection.matrix().as_slice();
    let r = rho.matrix().as_slice();

    // Tr(P ρ) = Σ_i (P ρ)_ii = Σ_i Σ_k P_ik ρ_ki.
    let mut tr_re = R::zero();
    let mut tr_im = R::zero();
    for i in 0..D {
        for k in 0..D {
            let pik = p[i * D + k];
            let rki = r[k * D + i];
            tr_re += pik.re * rki.re - pik.im * rki.im;
            tr_im += pik.re * rki.im + pik.im * rki.re;
        }
    }

    let tol = R::epsilon().sqrt();
    if tr_im.abs() > tol {
        return Err(QuantumError::NonFiniteValue(format!(
            "Born probability has a non-negligible imaginary part ({:?})",
            tr_im
        )));
    }
    if !tr_re.is_finite() {
        return Err(QuantumError::NonFiniteValue(
            "Born probability is not finite".into(),
        ));
    }
    Ok(tr_re.clamp(R::zero(), R::one()))
}

/// The Born-rule probability as the `Prob` MV-algebra verdict — the boundary
/// where a quantum measurement becomes a classical (fuzzy) verdict.
pub fn born_projective_prob<R, const D: usize>(
    rho: &DensityMatrix<R>,
    projection: &Projection<R, D>,
) -> Result<Prob, QuantumError>
where
    R: RealField + FromPrimitive + Default + core::fmt::Debug + Into<f64>,
{
    let p: R = born_projective_probability(rho, projection)?;
    Ok(Prob(p.into()))
}
