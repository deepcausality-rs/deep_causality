/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::QuantumError;
use crate::types::carriers::qubit_operator::QubitOperator;
use crate::types::decision::Tolerance;
use crate::types::qgates::channel::{
    apply_choi, apply_kraus, check_completely_positive, check_trace_preserving, choi_compose,
    choi_from_kraus,
};
use crate::types::qgates::operator_linalg::frobenius_norm;
use alloc::format;
use alloc::vec::Vec;
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;
use deep_causality_num_complex::Complex;
use deep_causality_tensor::CausalTensor;

/// A quantum channel, CPTP-checked once, at construction.
///
/// Construction builds the Choi operator with the shipped `choi_from_kraus` and runs the shipped
/// `check_completely_positive` and `check_trace_preserving` against a threshold drawn from the
/// [`Tolerance`] family. Application never re-checks: it routes to `apply_kraus` when the family
/// is held and to `apply_choi` when only the Choi is, which is the case after composition.
///
/// Unitary evolution enters through [`unitary`](Self::unitary), which builds the one-element
/// family itself. That closes the friction where a unitary was passed to `apply_kraus(&[u], ρ)`
/// and read as a channel at the call site.
#[derive(Debug, Clone, PartialEq)]
pub struct Channel<R: RealField> {
    kraus: Option<Vec<CausalTensor<Complex<R>>>>,
    choi: CausalTensor<Complex<R>>,
    d_in: usize,
    d_out: usize,
}

impl<R> Channel<R>
where
    R: RealField + FromPrimitive + Default + core::fmt::Debug,
{
    /// A channel from its Kraus family, CPTP-checked against the state member of the tolerance
    /// family scaled by the Choi operator's Frobenius norm.
    ///
    /// # Errors
    ///
    /// The structured error the shipped check produced: [`QuantumError::NonCptpChannel`] for a
    /// negative Choi eigenvalue or a `Tr_out(J) ≠ I_in` defect, [`QuantumError::NonPositiveOperator`]
    /// for a non-Hermitian Choi, and [`QuantumError::DimensionMismatch`] for a malformed family.
    /// No partially validated channel value is reachable.
    pub fn from_kraus(kraus: &[CausalTensor<Complex<R>>]) -> Result<Self, QuantumError> {
        Self::from_kraus_with_tolerance(kraus, &Tolerance::state())
    }

    /// [`from_kraus`](Self::from_kraus) against an explicit member of the tolerance family.
    ///
    /// # Errors
    ///
    /// As [`from_kraus`](Self::from_kraus), and [`QuantumError::CalculationError`] if the member
    /// cannot answer the single-operator form, which only the commutator member cannot.
    pub fn from_kraus_with_tolerance(
        kraus: &[CausalTensor<Complex<R>>],
        tolerance: &Tolerance<R>,
    ) -> Result<Self, QuantumError> {
        let (d_out, d_in) = kraus_dims(kraus)?;
        let choi = choi_from_kraus(kraus)?;
        let scale = frobenius_norm(&choi);
        let tol = tolerance.threshold(d_in * d_out, scale).ok_or_else(|| {
            QuantumError::CalculationError(format!(
                "the {} member has no single-operator threshold",
                tolerance.name()
            ))
        })?;
        check_completely_positive(&choi, tol)?;
        check_trace_preserving(&choi, d_in, d_out, tol)?;
        Ok(Self {
            kraus: Some(kraus.to_vec()),
            choi,
            d_in,
            d_out,
        })
    }

    /// The channel `ρ ↦ U ρ U†` of a qubit unitary.
    ///
    /// The one-element Kraus family is built here. No CPTP check runs, because the operator's
    /// unitarity was established when it was constructed and a unitary is CPTP exactly; the check
    /// that admits it is the one [`QubitOperator`] already ran.
    ///
    /// # Errors
    ///
    /// [`QuantumError::DimensionMismatch`] only if the Choi cannot be formed, which a `2 × 2`
    /// operator never triggers.
    pub fn unitary(u: &QubitOperator<R>) -> Result<Self, QuantumError> {
        let kraus = alloc::vec![u.matrix().clone()];
        let choi = choi_from_kraus(&kraus)?;
        Ok(Self {
            kraus: Some(kraus),
            choi,
            d_in: 2,
            d_out: 2,
        })
    }

    /// `E(ρ)`, through the Kraus family when it is held and through the Choi otherwise.
    ///
    /// # Errors
    ///
    /// [`QuantumError::DimensionMismatch`] from the shipped shape check when `ρ` is not
    /// `d_in × d_in`.
    pub fn apply(
        &self,
        rho: &CausalTensor<Complex<R>>,
    ) -> Result<CausalTensor<Complex<R>>, QuantumError> {
        match &self.kraus {
            Some(k) => apply_kraus(k, rho),
            None => apply_choi(&self.choi, rho, self.d_in, self.d_out),
        }
    }

    /// The composite that applies `self` and then `then`, built by `choi_compose` with no CPTP
    /// re-validation.
    ///
    /// Complete positivity and trace preservation are properties of the composed maps and are
    /// inherited by the composite, which is what `choi_compose`'s `RealField`-only bound records.
    /// The composite holds no Kraus family and applies through its Choi.
    ///
    /// # Errors
    ///
    /// [`QuantumError::DimensionMismatch`] if `self.d_out() != then.d_in()`.
    pub fn compose(&self, then: &Channel<R>) -> Result<Self, QuantumError> {
        if self.d_out != then.d_in {
            return Err(QuantumError::DimensionMismatch(format!(
                "cannot compose: first channel outputs dimension {}, second inputs {}",
                self.d_out, then.d_in
            )));
        }
        let choi = choi_compose(&self.choi, &then.choi, self.d_in, self.d_out, then.d_out)?;
        Ok(Self {
            kraus: None,
            choi,
            d_in: self.d_in,
            d_out: then.d_out,
        })
    }
}

impl<R: RealField> Channel<R> {
    /// The Choi operator `J(E)`, `(d_in·d_out)²`.
    pub fn choi(&self) -> &CausalTensor<Complex<R>> {
        &self.choi
    }

    /// The Kraus family, if the channel was built from one.
    pub fn kraus(&self) -> Option<&[CausalTensor<Complex<R>>]> {
        self.kraus.as_deref()
    }

    /// The input dimension.
    pub fn d_in(&self) -> usize {
        self.d_in
    }

    /// The output dimension.
    pub fn d_out(&self) -> usize {
        self.d_out
    }
}

/// `(d_out, d_in)` of a Kraus family, from its first operator's shape.
fn kraus_dims<R: RealField>(
    kraus: &[CausalTensor<Complex<R>>],
) -> Result<(usize, usize), QuantumError> {
    let first = kraus
        .first()
        .ok_or_else(|| QuantumError::NonCptpChannel("empty Kraus family".into()))?;
    match first.shape() {
        [d_out, d_in] if *d_out > 0 && *d_in > 0 => Ok((*d_out, *d_in)),
        other => Err(QuantumError::DimensionMismatch(format!(
            "Kraus operators must be non-empty matrices, got shape {other:?}"
        ))),
    }
}

/// The identity channel on a qubit, because the causal monad requires a `Default` for every value
/// it carries.
impl<R> Default for Channel<R>
where
    R: RealField + FromPrimitive + Default + core::fmt::Debug,
{
    fn default() -> Self {
        Self::unitary(&QubitOperator::identity()).expect("the identity forms a Choi")
    }
}
