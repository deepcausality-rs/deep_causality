/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::QuantumError;
use crate::types::decision::Tolerance;
use alloc::format;
use deep_causality_algebra::{ComplexField, Normed, RealField};
use deep_causality_haft::Functor;
use deep_causality_num::FromPrimitive;
use deep_causality_num_complex::{Complex, ComplexWitness};
use deep_causality_tensor::{CausalTensor, CausalTensorWitness};

/// The axis of a single-qubit rotation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Axis {
    /// `R_x(θ) = exp(−iθX/2)`.
    X,
    /// `R_y(θ) = exp(−iθY/2)`.
    Y,
    /// `R_z(θ) = exp(−iθZ/2)`.
    Z,
}

/// A single-qubit unitary, built by name and owning its shape.
///
/// The alphabet is the one the crate already named in `QuantumGates` and never implemented:
/// identity, the three Paulis, Hadamard, the axis rotations and the phase gate. Each named
/// constructor is unitary by construction, and [`from_matrix`](Self::from_matrix) admits any
/// other `2 × 2` matrix after checking that it is. The interior is exposed as
/// `&CausalTensor<Complex<R>>`, which is what `matrix_commutator`, `embed_on_legs`,
/// `hermiticity_defect` and `choi_from_kraus` consume, so no conversion sits between a carrier
/// and the shipped operator layer.
#[derive(Debug, Clone, PartialEq)]
pub struct QubitOperator<R: RealField> {
    matrix: CausalTensor<Complex<R>>,
}

impl<R> QubitOperator<R>
where
    R: RealField + FromPrimitive + Default + core::fmt::Debug,
{
    fn from_entries(entries: [Complex<R>; 4]) -> Self {
        Self {
            matrix: CausalTensor::from_slice(&entries, &[2, 2]),
        }
    }

    /// `I`.
    pub fn identity() -> Self {
        let (z, o) = (R::zero(), R::one());
        Self::from_entries([
            Complex::new(o, z),
            Complex::new(z, z),
            Complex::new(z, z),
            Complex::new(o, z),
        ])
    }

    /// `X = [[0, 1], [1, 0]]`.
    pub fn pauli_x() -> Self {
        let (z, o) = (R::zero(), R::one());
        Self::from_entries([
            Complex::new(z, z),
            Complex::new(o, z),
            Complex::new(o, z),
            Complex::new(z, z),
        ])
    }

    /// `Y = [[0, −i], [i, 0]]`.
    pub fn pauli_y() -> Self {
        let (z, o) = (R::zero(), R::one());
        Self::from_entries([
            Complex::new(z, z),
            Complex::new(z, -o),
            Complex::new(z, o),
            Complex::new(z, z),
        ])
    }

    /// `Z = [[1, 0], [0, −1]]`.
    pub fn pauli_z() -> Self {
        let (z, o) = (R::zero(), R::one());
        Self::from_entries([
            Complex::new(o, z),
            Complex::new(z, z),
            Complex::new(z, z),
            Complex::new(-o, z),
        ])
    }

    /// `H = [[1, 1], [1, −1]] / √2`.
    pub fn hadamard() -> Self {
        let z = R::zero();
        let h = R::one() / (R::one() + R::one()).sqrt();
        Self::from_entries([
            Complex::new(h, z),
            Complex::new(h, z),
            Complex::new(h, z),
            Complex::new(-h, z),
        ])
    }

    /// `R_axis(θ) = cos(θ/2)·I − i·sin(θ/2)·σ_axis`.
    ///
    /// # Errors
    ///
    /// [`QuantumError::NonFiniteValue`] if `angle` is not finite; no operator value exists then.
    pub fn rotation(axis: Axis, angle: R) -> Result<Self, QuantumError> {
        if !angle.is_finite() {
            return Err(QuantumError::NonFiniteValue(format!(
                "rotation angle is not finite: {angle:?}"
            )));
        }
        let half = angle / (R::one() + R::one());
        let (cs, sn, z) = (half.cos(), half.sin(), R::zero());
        Ok(match axis {
            Axis::X => Self::from_entries([
                Complex::new(cs, z),
                Complex::new(z, -sn),
                Complex::new(z, -sn),
                Complex::new(cs, z),
            ]),
            Axis::Y => Self::from_entries([
                Complex::new(cs, z),
                Complex::new(-sn, z),
                Complex::new(sn, z),
                Complex::new(cs, z),
            ]),
            Axis::Z => Self::from_entries([
                Complex::new(cs, -sn),
                Complex::new(z, z),
                Complex::new(z, z),
                Complex::new(cs, sn),
            ]),
        })
    }

    /// `P(θ) = diag(1, e^{iθ})`.
    ///
    /// # Errors
    ///
    /// [`QuantumError::NonFiniteValue`] if `theta` is not finite.
    pub fn phase(theta: R) -> Result<Self, QuantumError> {
        if !theta.is_finite() {
            return Err(QuantumError::NonFiniteValue(format!(
                "phase angle is not finite: {theta:?}"
            )));
        }
        let (z, o) = (R::zero(), R::one());
        Ok(Self::from_entries([
            Complex::new(o, z),
            Complex::new(z, z),
            Complex::new(z, z),
            Complex::new(theta.cos(), theta.sin()),
        ]))
    }

    /// Any `2 × 2` unitary, checked.
    ///
    /// The check is `max |U U† − I| ≤ √ε`, the validation member of the [`Tolerance`] family, so
    /// it tightens with the scalar. A named constructor needs no such check and runs none.
    ///
    /// # Errors
    ///
    /// [`QuantumError::DimensionMismatch`] if the shape is not `[2, 2]`;
    /// [`QuantumError::NonFiniteValue`] on a non-finite entry;
    /// [`QuantumError::NonPositiveOperator`] if the matrix is not unitary within tolerance.
    pub fn from_matrix(matrix: CausalTensor<Complex<R>>) -> Result<Self, QuantumError> {
        if matrix.shape() != [2, 2] {
            return Err(QuantumError::DimensionMismatch(format!(
                "a qubit operator is 2 × 2, got shape {:?}",
                matrix.shape()
            )));
        }
        if matrix
            .as_slice()
            .iter()
            .any(|z| !z.re.is_finite() || !z.im.is_finite())
        {
            return Err(QuantumError::NonFiniteValue(
                "qubit operator contains a non-finite entry".into(),
            ));
        }
        let candidate = Self { matrix };
        let defect = candidate.unitarity_defect();
        let tol = Tolerance::<R>::validation()
            .threshold(2, R::one())
            .expect("the validation member answers the single-operator form");
        if defect > tol {
            return Err(QuantumError::NonPositiveOperator(format!(
                "not unitary: max |UU† − I| = {defect:?} > {tol:?}"
            )));
        }
        Ok(candidate)
    }

    /// `max |U U† − I|` over the four entries: zero for a unitary, up to rounding. The modulus
    /// is taken through `Normed::modulus`, whose scaled form does not overflow on a large entry.
    pub fn unitarity_defect(&self) -> R {
        let u = self.matrix.as_slice();
        let mut worst = R::zero();
        for i in 0..2 {
            for j in 0..2 {
                // (U U†)_ij = Σ_k U_ik · conj(U_jk)
                let mut acc = Complex::new(R::zero(), R::zero());
                for k in 0..2 {
                    acc += u[i * 2 + k] * u[j * 2 + k].conjugate();
                }
                let target = if i == j { R::one() } else { R::zero() };
                let d = Complex::new(acc.re - target, acc.im).modulus();
                if d > worst {
                    worst = d;
                }
            }
        }
        worst
    }

    /// The same operator at another scalar, re-validated there.
    ///
    /// The lift is a composition of two functors: the outer over the tensor's cells, the inner over
    /// the real and imaginary slots of each `Complex`. That is what makes "precision is a
    /// parameter" true of this carrier rather than of a real-valued example, and the re-validation
    /// is what makes the invariant survive it.
    ///
    /// # The direction matters, and the re-validation says so
    ///
    /// The target's tolerance is the target's `√ε`. Lifting down, from `f64` to `f32`, the source's
    /// rounding is far inside the target's tolerance and the lift passes. Lifting up, from `f32`
    /// to `f64`, the source's rounding of about `6e-8` per entry is *visible* at the target's
    /// `1.5e-8`, and the lift is refused unless the entries were exactly representable. That is
    /// not a defect of the lift: the check tightened with the scalar and the data did not, which
    /// is what running at two scalars is meant to show.
    ///
    /// # Errors
    ///
    /// As [`from_matrix`](Self::from_matrix) at the target scalar.
    pub fn lift<S, F>(&self, mut f: F) -> Result<QubitOperator<S>, QuantumError>
    where
        S: RealField + FromPrimitive + Default + core::fmt::Debug,
        F: FnMut(R) -> S,
    {
        let lifted =
            CausalTensorWitness::fmap(self.matrix.clone(), |z| ComplexWitness::fmap(z, &mut f));
        QubitOperator::<S>::from_matrix(lifted)
    }
}

impl<R: RealField> QubitOperator<R> {
    /// The `[2, 2]` matrix, which the shipped channel and linear-algebra functions consume directly.
    pub fn matrix(&self) -> &CausalTensor<Complex<R>> {
        &self.matrix
    }

    /// Always two.
    pub fn dim(&self) -> usize {
        2
    }
}

/// The identity, because the causal monad requires a `Default` for every value it carries.
impl<R> Default for QubitOperator<R>
where
    R: RealField + FromPrimitive + Default + core::fmt::Debug,
{
    fn default() -> Self {
        Self::identity()
    }
}
