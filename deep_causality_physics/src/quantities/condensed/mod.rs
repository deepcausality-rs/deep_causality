/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::PhysicsError;
use deep_causality_algebra::DivisionAlgebra;
use deep_causality_multivector::{CausalMultiVector, Metric};
use deep_causality_num_complex::Complex;
use deep_causality_tensor::CausalTensor;

// ============================================================================
// Scalars
// ============================================================================

/// Quantum Metric component ($g_{ij}$).
///
/// Represents the real symmetric part of the Quantum Geometric Tensor.
/// It measures the "distance" between quantum states in parameter space.
///
/// *   **Dimensions**: Usually dimensionless (if $k$ is dimensionless) or $L^2$.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct QuantumMetric<R: deep_causality_algebra::RealField>(R);

impl<R: deep_causality_algebra::RealField> Default for QuantumMetric<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: deep_causality_algebra::RealField> QuantumMetric<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        // Metric components can be negative (off-diagonal), so no invariant check here.
        Ok(Self(val))
    }
    pub fn value(&self) -> R {
        self.0
    }
}

impl<R: deep_causality_algebra::RealField + Into<f64>> From<QuantumMetric<R>> for f64 {
    fn from(val: QuantumMetric<R>) -> Self {
        val.0.into()
    }
}

/// Berry Curvature component ($Ω_{ij}$).
///
/// Represents the imaginary antisymmetric part of the Quantum Geometric Tensor.
/// It acts like a magnetic field in momentum space, influencing electron dynamics (anomalous velocity).
///
/// *   **Dimensions**: Area ($L^2$) or dimensionless depending on $k$-space normalization.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct BerryCurvature<R: deep_causality_algebra::RealField>(R);

impl<R: deep_causality_algebra::RealField> Default for BerryCurvature<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: deep_causality_algebra::RealField> BerryCurvature<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        Ok(Self(val))
    }
    pub fn value(&self) -> R {
        self.0
    }
}

impl<R: deep_causality_algebra::RealField + Into<f64>> From<BerryCurvature<R>> for f64 {
    fn from(val: BerryCurvature<R>) -> Self {
        val.0.into()
    }
}

/// Band Drude Weight ($D$).
///
/// A measure of coherent electron transport (conductivity weight) in a band.
/// Includes both conventional (kinetic) and geometric contributions.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct BandDrudeWeight<R: deep_causality_algebra::RealField>(R);

impl<R: deep_causality_algebra::RealField> Default for BandDrudeWeight<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: deep_causality_algebra::RealField> BandDrudeWeight<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        Ok(Self(val))
    }
    pub fn value(&self) -> R {
        self.0
    }
}

impl<R: deep_causality_algebra::RealField + Into<f64>> From<BandDrudeWeight<R>> for f64 {
    fn from(val: BandDrudeWeight<R>) -> Self {
        val.0.into()
    }
}

/// Orbital Angular Momentum ($L$).
///
/// Intrinsic orbital moment of the Bloch packet.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct OrbitalAngularMomentum<R: deep_causality_algebra::RealField>(R);

impl<R: deep_causality_algebra::RealField> Default for OrbitalAngularMomentum<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: deep_causality_algebra::RealField> OrbitalAngularMomentum<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        Ok(Self(val))
    }
    pub fn value(&self) -> R {
        self.0
    }
}

impl<R: deep_causality_algebra::RealField + Into<f64>> From<OrbitalAngularMomentum<R>> for f64 {
    fn from(val: OrbitalAngularMomentum<R>) -> Self {
        val.0.into()
    }
}

/// Electrical Conductance ($G$).
///
/// Units: Siemens ($S = Ω^{-1}$).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Conductance<R: deep_causality_algebra::RealField>(R);

impl<R: deep_causality_algebra::RealField> Default for Conductance<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: deep_causality_algebra::RealField> Conductance<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        if val < R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Negative Conductance".into(),
            ));
        }
        Ok(Self(val))
    }
    pub fn new_unchecked(val: R) -> Self {
        Self(val)
    }
    pub fn value(&self) -> R {
        self.0
    }
}

impl<R: deep_causality_algebra::RealField + Into<f64>> From<Conductance<R>> for f64 {
    fn from(val: Conductance<R>) -> Self {
        val.0.into()
    }
}

/// Charge Carrier Mobility ($μ$).
///
/// Units: $m^2 / (V · s)$.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Mobility<R: deep_causality_algebra::RealField>(R);

impl<R: deep_causality_algebra::RealField> Default for Mobility<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: deep_causality_algebra::RealField> Mobility<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        if val < R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Negative Mobility".into(),
            ));
        }
        Ok(Self(val))
    }
    pub fn new_unchecked(val: R) -> Self {
        Self(val)
    }
    pub fn value(&self) -> R {
        self.0
    }
}

impl<R: deep_causality_algebra::RealField + Into<f64>> From<Mobility<R>> for f64 {
    fn from(val: Mobility<R>) -> Self {
        val.0.into()
    }
}

/// Moiré Twist Angle ($θ$).
///
/// Units: Radians.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct TwistAngle<R: deep_causality_algebra::RealField>(R);

impl<R: deep_causality_algebra::RealField> Default for TwistAngle<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: deep_causality_algebra::RealField> TwistAngle<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        Ok(Self(val))
    }
    pub fn value(&self) -> R {
        self.0
    }
}

impl<R: deep_causality_algebra::RealField + deep_causality_num::FromPrimitive> TwistAngle<R> {
    pub fn as_degrees(&self) -> R {
        let factor =
            R::from_f64(180.0 / core::f64::consts::PI).expect("R::from_f64(180/PI) failed");
        self.0 * factor
    }
    pub fn from_degrees(deg: R) -> Self {
        let factor =
            R::from_f64(core::f64::consts::PI / 180.0).expect("R::from_f64(PI/180) failed");
        Self(deg * factor)
    }
}

impl<R: deep_causality_algebra::RealField + Into<f64>> From<TwistAngle<R>> for f64 {
    fn from(val: TwistAngle<R>) -> Self {
        val.0.into()
    }
}

/// Superconducting Order Parameter ($ψ$).
///
/// A complex scalar field describing the macroscopic condensate wavefunction.
/// *   $|ψ|^2 ≠ n_s$ (superfluid density).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct OrderParameter<R: deep_causality_algebra::RealField>(Complex<R>);

impl<R: deep_causality_algebra::RealField> OrderParameter<R> {
    pub fn new(val: Complex<R>) -> Self {
        Self(val)
    }
    pub fn value(&self) -> Complex<R> {
        self.0
    }
    pub fn magnitude_squared(&self) -> R {
        self.0.norm_sqr()
    }
}

// ============================================================================
// Data Structures
// ============================================================================

/// Quantum Eigenvector $|u_n➢$.
///
/// Represents the cell-periodic part of the Bloch function.
/// *   **Rank 2 Tensor**: [basis_size, num_states].
/// *   Columns correspond to different bands $n$.
#[derive(Debug, Clone, PartialEq)]
pub struct QuantumEigenvector<R: deep_causality_algebra::RealField>(CausalTensor<Complex<R>>);

impl<R: deep_causality_algebra::RealField> QuantumEigenvector<R> {
    pub fn new(tensor: CausalTensor<Complex<R>>) -> Self {
        Self(tensor)
    }
    pub fn inner(&self) -> &CausalTensor<Complex<R>> {
        &self.0
    }
}

/// Quantum Velocity vector $\partial_i H |u_n➢$.
///
/// Represents the velocity operator applied to the eigenstates. Used in perturbative calculations
/// like the QGT or Kub-Greenwood conductivity.
/// *   **Rank 2 Tensor**: [basis_size, num_states].
#[derive(Debug, Clone, PartialEq)]
pub struct QuantumVelocity<R: deep_causality_algebra::RealField>(CausalTensor<Complex<R>>);

impl<R: deep_causality_algebra::RealField> QuantumVelocity<R> {
    pub fn new(tensor: CausalTensor<Complex<R>>) -> Self {
        Self(tensor)
    }
    pub fn inner(&self) -> &CausalTensor<Complex<R>> {
        &self.0
    }
}

/// Momentum vector $\mathbf{k}$.
///
/// Represents a point in the Brillouin Zone.
#[derive(Debug, Clone, PartialEq)]
pub struct Momentum<R: deep_causality_algebra::RealField>(CausalMultiVector<R>);

impl<R: deep_causality_algebra::RealField> Default for Momentum<R> {
    fn default() -> Self {
        Self(CausalMultiVector::new(vec![R::zero()], Metric::Euclidean(0)).unwrap())
    }
}

impl<R: deep_causality_algebra::RealField> Momentum<R> {
    pub fn new(mv: CausalMultiVector<R>) -> Self {
        Self(mv)
    }
    pub fn inner(&self) -> &CausalMultiVector<R> {
        &self.0
    }
}

/// Displacement field $\mathbf{u}(\mathbf{r})$.
///
/// Represents the mechanical displacement vector field or strain tensor components.
#[derive(Debug, Clone, PartialEq)]
pub struct Displacement<R: deep_causality_algebra::RealField>(CausalTensor<R>);

impl<R: deep_causality_algebra::RealField> Displacement<R> {
    pub fn new(tensor: CausalTensor<R>) -> Self {
        Self(tensor)
    }
    pub fn inner(&self) -> &CausalTensor<R> {
        &self.0
    }
}

/// Concentration field $c(\mathbf{r})$.
///
/// Represents the local concentration (mole fraction) of a species.
/// *   **Values**: Must be non-negative.
#[derive(Debug, Clone, PartialEq)]
pub struct Concentration<R: deep_causality_algebra::RealField>(CausalTensor<R>);

impl<R: deep_causality_algebra::RealField> Concentration<R> {
    pub fn new(tensor: CausalTensor<R>) -> Result<Self, PhysicsError> {
        // Concentration cannot be negative
        for &val in tensor.as_slice() {
            if val < R::zero() {
                return Err(PhysicsError::PhysicalInvariantBroken(
                    "Negative Concentration detected".into(),
                ));
            }
        }
        Ok(Self(tensor))
    }
    /// Creates a new Concentration without validation.
    /// Use only if the tensor is guaranteed to be non-negative.
    pub fn new_unchecked(tensor: CausalTensor<R>) -> Self {
        Self(tensor)
    }
    pub fn inner(&self) -> &CausalTensor<R> {
        &self.0
    }
}

/// Chemical Potential Gradient $\nabla μ$.
///
/// Driving force for diffusion.
#[derive(Debug, Clone, PartialEq)]
pub struct ChemicalPotentialGradient<R: deep_causality_algebra::RealField>(CausalTensor<R>);

impl<R: deep_causality_algebra::RealField> ChemicalPotentialGradient<R> {
    pub fn new(tensor: CausalTensor<R>) -> Self {
        Self(tensor)
    }
    pub fn inner(&self) -> &CausalTensor<R> {
        &self.0
    }
}

/// Electromagnetic Vector Potential $\mathbf{A}$.
///
/// Used in the covariant derivative $\nabla - i\mathbf{A}$.
#[derive(Debug, Clone, PartialEq)]
pub struct VectorPotential<R: deep_causality_algebra::RealField>(CausalMultiVector<R>);

impl<R: deep_causality_algebra::RealField> Default for VectorPotential<R> {
    fn default() -> Self {
        Self(CausalMultiVector::new(vec![R::zero()], Metric::Euclidean(0)).unwrap())
    }
}

impl<R: deep_causality_algebra::RealField> VectorPotential<R> {
    pub fn new(mv: CausalMultiVector<R>) -> Self {
        Self(mv)
    }
    pub fn inner(&self) -> &CausalMultiVector<R> {
        &self.0
    }
}
