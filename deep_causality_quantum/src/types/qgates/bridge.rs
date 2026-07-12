/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Ket-bridge helpers: the metric-correct Dirac inner product on
//! minimal-left-ideal kets.
//!
//! The `HilbertState::to_ket`/`from_ket` bridge (in
//! `deep_causality_multivector`) identifies a minimal-left-ideal ket with a
//! `D`-dimensional complex column, normalized so the raw column inner product
//! `k(φ)ᴴ·k(ψ)` equals the Dirac product `⟨φ|ψ⟩`. Which multivector involution
//! realizes that Dirac product depends on the metric signature — established
//! numerically in the `add-quantum-crate` Phase-2 gate:
//!
//! - **Positive (Euclidean) signature:** reversion + coefficient conjugation
//!   (`QuantumOps::dag`) — the classic Dirac adjoint.
//! - **Negative (`Cl(0,n)`) signature:** the reversion form is *degenerate*
//!   (identically zero) on the minimal left ideal; the **Clifford
//!   conjugation** (grade involution ∘ reversion) + coefficient conjugation
//!   is the metric-correct adjoint.

use crate::{QuantumError, QuantumOps};
use deep_causality_algebra::RealField;
use deep_causality_multivector::{CausalMultiVector, HilbertState, MultiVector};
use deep_causality_num_complex::Complex;

/// The Clifford conjugation with coefficient conjugation: the grade-`k` blade
/// picks up the sign `(−1)^(k(k+1)/2)` (grade involution ∘ reversion) and
/// every coefficient is complex-conjugated. On a negative-signature metric
/// this is the adjoint whose minimal-left-ideal restriction matches the
/// matrix-column Hermitian adjoint.
pub fn clifford_conjugation<R>(mv: &CausalMultiVector<Complex<R>>) -> CausalMultiVector<Complex<R>>
where
    R: RealField,
{
    let data: Vec<Complex<R>> = mv
        .data()
        .iter()
        .enumerate()
        .map(|(idx, c)| {
            let k = (idx as u32).count_ones() as usize;
            if (k * (k + 1) / 2).is_multiple_of(2) {
                Complex::new(c.re, -c.im)
            } else {
                Complex::new(-c.re, c.im)
            }
        })
        .collect();
    CausalMultiVector::new(data, mv.metric()).expect("blade count is preserved")
}

/// The metric-correct Dirac *adjoint* (bra) involution on a ket's multivector:
/// [`QuantumOps::dag`] (reversion + conjugation) for a positive-signature
/// metric, [`clifford_conjugation`] for a negative-signature metric. Unlike
/// [`dirac_bracket_kernel`], this selects only the involution and is therefore
/// defined for any dimension (not just the even-dimensional column bridge), so
/// it is the shared source of truth for the Born/expectation kernels.
///
/// # Errors
/// `UnsupportedMetric` for a mixed-signature metric, where neither involution
/// realizes the Dirac adjoint on the minimal left ideal.
pub(crate) fn metric_adjoint<R>(
    mv: &CausalMultiVector<Complex<R>>,
) -> Result<CausalMultiVector<Complex<R>>, QuantumError>
where
    R: RealField + core::iter::Sum,
{
    let metric = mv.metric();
    let n = metric.dimension();
    let uniform_positive = (0..n).all(|i| metric.sign_of_sq(i) == 1);
    let uniform_negative = (0..n).all(|i| metric.sign_of_sq(i) == -1);
    if n == 0 || uniform_positive {
        Ok(mv.dag())
    } else if uniform_negative {
        Ok(clifford_conjugation(mv))
    } else {
        Err(QuantumError::UnsupportedMetric(format!(
            "the Dirac adjoint requires a uniform-signature metric, got {:?}",
            metric
        )))
    }
}

/// The metric-correct Dirac inner product `⟨φ|ψ⟩` on kets: the scalar part of
/// `adj(φ)·ψ`, where `adj` is `QuantumOps::dag` (reversion + conjugation) for
/// a positive-signature metric and [`clifford_conjugation`] for a
/// negative-signature metric. On the minimal left ideal this equals the raw
/// column inner product of `to_ket(φ)` and `to_ket(ψ)`.
///
/// # Errors
/// Returns `MetricMismatch` if the operands carry different metrics, and
/// `UnsupportedMetric` for a mixed-signature metric (Minkowski/Lorentzian),
/// where neither involution realizes the Dirac product on the ideal.
pub fn dirac_bracket_kernel<R>(
    phi: &HilbertState<R>,
    psi: &HilbertState<R>,
) -> Result<Complex<R>, QuantumError>
where
    R: RealField + core::iter::Sum,
{
    let metric = phi.mv().metric();
    if metric != psi.mv().metric() {
        return Err(QuantumError::MetricMismatch(format!(
            "Metric mismatch in Dirac bracket: {:?} vs {:?}",
            metric,
            psi.mv().metric()
        )));
    }

    // The column bridge (to_ket/from_ket) is defined only for even dimensions
    // (D = 2^(n/2)); the promised column-inner-product equivalence is otherwise
    // unverifiable, so reject odd dimensions rather than return a bracket that
    // cannot be a column inner product.
    let n = metric.dimension();
    if n != 0 && !n.is_multiple_of(2) {
        return Err(QuantumError::UnsupportedMetric(format!(
            "the Dirac ket bridge is defined only for even-dimensional metrics, got dimension {}",
            n
        )));
    }

    let adj = metric_adjoint(phi.mv())?;
    let prod = adj.geometric_product(psi.mv());
    Ok(prod
        .get(0)
        .cloned()
        .unwrap_or(Complex::new(R::zero(), R::zero())))
}
