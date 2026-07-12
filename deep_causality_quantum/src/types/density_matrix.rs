/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The mixed-state density matrix with enforced invariants (L1 of the
//! operator build ladder, design B4).

use crate::QuantumError;
use crate::types::qgates::operator_linalg::{
    frobenius_norm, hermiticity_defect, matrix_trace, square_dim,
};
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;
use deep_causality_num_complex::Complex;
use deep_causality_tensor::CausalTensor;

/// A validated density matrix: Hermitian, positive semidefinite (checked via
/// the dense Hermitian eigendecomposition), and unit trace — every
/// constructor enforces the invariants and returns a typed [`QuantumError`]
/// on violation.
///
/// Reference: R. Lorenz, "Quantum causal models: the merits of the spirit of
/// Reichenbach's principle" (2022), §2.
#[derive(Debug, Clone, PartialEq)]
pub struct DensityMatrix<R: RealField> {
    matrix: CausalTensor<Complex<R>>,
    dim: usize,
}

impl<R> DensityMatrix<R>
where
    R: RealField + FromPrimitive + Default + core::fmt::Debug,
{
    /// The default validation tolerance: `√ε` of the real carrier, scaled by
    /// the operator's Frobenius norm during checks.
    pub fn default_tolerance() -> R {
        R::epsilon().sqrt()
    }

    /// Validates `matrix` as a density matrix with the default tolerance.
    pub fn new(matrix: CausalTensor<Complex<R>>) -> Result<Self, QuantumError> {
        Self::with_tolerance(matrix, Self::default_tolerance())
    }

    /// Validates `matrix` as a density matrix: square, finite, Hermitian,
    /// PSD (spectrum ≥ −tol·scale), and unit trace (|Tr − 1| ≤ tol·scale),
    /// with `scale = max(1, ‖M‖_F)`.
    pub fn with_tolerance(matrix: CausalTensor<Complex<R>>, tol: R) -> Result<Self, QuantumError> {
        let dim = square_dim(&matrix)?;

        if matrix
            .as_slice()
            .iter()
            .any(|c| !c.re.is_finite() || !c.im.is_finite())
        {
            return Err(QuantumError::NonFiniteValue(
                "density matrix contains a non-finite entry".into(),
            ));
        }

        let norm = frobenius_norm(&matrix);
        let scale = if norm > R::one() { norm } else { R::one() };
        let eps = tol * scale;

        let defect = hermiticity_defect(&matrix)?;
        if defect > eps {
            return Err(QuantumError::NonPositiveOperator(format!(
                "not Hermitian: defect {:?} > {:?}",
                defect, eps
            )));
        }

        let (vals, _) = matrix
            .eigen_hermitian()
            .map_err(|e| QuantumError::CalculationError(format!("eigen: {:?}", e)))?;
        for lam in &vals {
            if lam.re < -eps {
                return Err(QuantumError::NonPositiveOperator(format!(
                    "negative eigenvalue {:?}",
                    lam.re
                )));
            }
        }

        let tr = matrix_trace(&matrix)?;
        let tr_defect = ((tr.re - R::one()) * (tr.re - R::one()) + tr.im * tr.im).sqrt();
        if tr_defect > eps {
            return Err(QuantumError::NonUnitTrace(format!(
                "trace = ({:?}, {:?})",
                tr.re, tr.im
            )));
        }

        Ok(Self { matrix, dim })
    }

    /// The pure state `ρ = |ψ⟩⟨ψ| / ⟨ψ|ψ⟩` of a (non-zero) ket column —
    /// kets are rays, so the normalization is gauge, not data.
    pub fn from_ket(ket: &CausalTensor<Complex<R>>) -> Result<Self, QuantumError> {
        let shape = ket.shape();
        let d = match shape {
            [d] | [d, 1] => *d,
            _ => {
                return Err(QuantumError::DimensionMismatch(format!(
                    "expected a ket column, got shape {:?}",
                    shape
                )));
            }
        };
        if d == 0 {
            return Err(QuantumError::DimensionMismatch("empty ket".into()));
        }
        let ks = ket.as_slice();
        if ks.iter().any(|c| !c.re.is_finite() || !c.im.is_finite()) {
            return Err(QuantumError::NonFiniteValue(
                "ket contains a non-finite entry".into(),
            ));
        }
        let norm_sq = ks
            .iter()
            .fold(R::zero(), |acc, c| acc + c.re * c.re + c.im * c.im);
        if norm_sq <= R::epsilon() {
            return Err(QuantumError::NormalizationError(
                "cannot form a state from a (near-)zero ket".into(),
            ));
        }

        let inv = R::one() / norm_sq;
        let mut data = vec![Complex::new(R::zero(), R::zero()); d * d];
        for i in 0..d {
            for j in 0..d {
                let a = ks[i];
                let b = ks[j];
                // a · conj(b) / ⟨ψ|ψ⟩
                data[i * d + j] = Complex::new(
                    (a.re * b.re + a.im * b.im) * inv,
                    (a.im * b.re - a.re * b.im) * inv,
                );
            }
        }
        Self::new(CausalTensor::from_slice(&data, &[d, d]))
    }

    /// The mixed state `ρ = Σ p_κ |ψ_κ⟩⟨ψ_κ|` of an ensemble of weighted kets;
    /// weights must be non-negative and sum to one (within the default
    /// tolerance). Each ket is ray-normalized.
    pub fn from_ensemble(ensemble: &[(R, CausalTensor<Complex<R>>)]) -> Result<Self, QuantumError> {
        if ensemble.is_empty() {
            return Err(QuantumError::NormalizationError("empty ensemble".into()));
        }
        let tol = Self::default_tolerance();
        let mut weight_sum = R::zero();
        for (p, _) in ensemble {
            if *p < -tol {
                return Err(QuantumError::NormalizationError(format!(
                    "negative ensemble weight {:?}",
                    p
                )));
            }
            weight_sum += *p;
        }
        let one_defect = if weight_sum > R::one() {
            weight_sum - R::one()
        } else {
            R::one() - weight_sum
        };
        if one_defect > tol {
            return Err(QuantumError::NormalizationError(format!(
                "ensemble weights sum to {:?}, not 1",
                weight_sum
            )));
        }

        let mut acc: Option<Vec<Complex<R>>> = None;
        let mut dim = 0usize;
        for (p, ket) in ensemble {
            let pure = Self::from_ket(ket)?;
            if let Some(a) = &acc {
                if pure.dim != dim {
                    return Err(QuantumError::DimensionMismatch(format!(
                        "ensemble kets disagree on dimension: {} vs {}",
                        dim, pure.dim
                    )));
                }
                let _ = a;
            } else {
                dim = pure.dim;
                acc = Some(vec![Complex::new(R::zero(), R::zero()); dim * dim]);
            }
            let a = acc.as_mut().expect("initialized above");
            for (o, v) in a.iter_mut().zip(pure.matrix.as_slice()) {
                *o = Complex::new(o.re + *p * v.re, o.im + *p * v.im);
            }
        }
        Self::new(CausalTensor::from_slice(
            &acc.expect("non-empty ensemble"),
            &[dim, dim],
        ))
    }

    /// The Choi *state* of a channel: `ρ_J = J / Tr(J)` — the normalized
    /// Choi–Jamiołkowski operator on `H_in ⊗ H_out` (for a CPTP channel,
    /// `Tr(J) = d_in`). Validation rejects a non-PSD `J`.
    pub fn from_choi(choi: &CausalTensor<Complex<R>>) -> Result<Self, QuantumError> {
        let d = square_dim(choi)?;
        let tr = matrix_trace(choi)?;
        if tr.re <= R::epsilon() {
            return Err(QuantumError::NonUnitTrace(format!(
                "Choi operator has non-positive trace ({:?}, {:?})",
                tr.re, tr.im
            )));
        }
        let inv = R::one() / tr.re;
        let data: Vec<Complex<R>> = choi
            .as_slice()
            .iter()
            .map(|c| Complex::new(c.re * inv, c.im * inv))
            .collect();
        Self::new(CausalTensor::from_slice(&data, &[d, d]))
    }
}

// Read-only accessors need no numeric-conversion bounds, so a validated
// `DensityMatrix` (e.g. inside an immutable `EnvironmentalPrep`) can be read
// under the plain `RealField` bound.
impl<R: RealField> DensityMatrix<R> {
    /// The validated matrix.
    pub fn matrix(&self) -> &CausalTensor<Complex<R>> {
        &self.matrix
    }

    /// The Hilbert dimension.
    pub fn dim(&self) -> usize {
        self.dim
    }

    /// The purity `Tr(ρ²)` — `1` exactly for a pure state, `1/d` for the
    /// maximally mixed state.
    pub fn purity(&self) -> R {
        // Tr(ρ²) = Σ_ij ρ_ij·ρ_ji = Σ_ij |ρ_ij|² for Hermitian ρ.
        self.matrix
            .as_slice()
            .iter()
            .fold(R::zero(), |acc, c| acc + c.re * c.re + c.im * c.im)
    }

    /// Whether the state is pure within `tol`: `|Tr(ρ²) − 1| ≤ tol`.
    pub fn is_pure(&self, tol: R) -> bool {
        let p = self.purity();
        let defect = if p > R::one() {
            p - R::one()
        } else {
            R::one() - p
        };
        defect <= tol
    }
}
