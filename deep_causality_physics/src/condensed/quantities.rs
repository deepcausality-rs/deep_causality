/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::PhysicsError;
use deep_causality_multivector::{CausalMultiVector, Metric};
use deep_causality_num::{Complex, DivisionAlgebra};
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
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct QuantumMetric(f64);

impl QuantumMetric {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        // Metric components can be negative (off-diagonal), so no invariant check here.
        Ok(Self(val))
    }
    pub fn value(&self) -> f64 {
        self.0
    }
}
impl From<QuantumMetric> for f64 {
    fn from(val: QuantumMetric) -> Self {
        val.0
    }
}

/// Berry Curvature component ($Ω_{ij}$).
///
/// Represents the imaginary antisymmetric part of the Quantum Geometric Tensor.
/// It acts like a magnetic field in momentum space, influencing electron dynamics (anomalous velocity).
///
/// *   **Dimensions**: Area ($L^2$) or dimensionless depending on $k$-space normalization.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct BerryCurvature(f64);

impl BerryCurvature {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        Ok(Self(val))
    }
    pub fn value(&self) -> f64 {
        self.0
    }
}
impl From<BerryCurvature> for f64 {
    fn from(val: BerryCurvature) -> Self {
        val.0
    }
}

/// Band Drude Weight ($D$).
///
/// A measure of coherent electron transport (conductivity weight) in a band.
/// Includes both conventional (kinetic) and geometric contributions.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct BandDrudeWeight(f64);

impl BandDrudeWeight {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        Ok(Self(val))
    }
    pub fn value(&self) -> f64 {
        self.0
    }
}
impl From<BandDrudeWeight> for f64 {
    fn from(val: BandDrudeWeight) -> Self {
        val.0
    }
}

/// Orbital Angular Momentum ($L$).
///
/// Intrinsic orbital moment of the Bloch packet.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct OrbitalAngularMomentum(f64);

impl OrbitalAngularMomentum {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        Ok(Self(val))
    }
    pub fn value(&self) -> f64 {
        self.0
    }
}
impl From<OrbitalAngularMomentum> for f64 {
    fn from(val: OrbitalAngularMomentum) -> Self {
        val.0
    }
}

/// Electrical Conductance ($G$).
///
/// Units: Siemens ($S = Ω^{-1}$).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Conductance(f64);

impl Conductance {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if val < 0.0 {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Negative Conductance".into(),
            ));
        }
        Ok(Self(val))
    }
    pub fn new_unchecked(val: f64) -> Self {
        Self(val)
    }
    pub fn value(&self) -> f64 {
        self.0
    }
}
impl From<Conductance> for f64 {
    fn from(val: Conductance) -> Self {
        val.0
    }
}

/// Charge Carrier Mobility ($μ$).
///
/// Units: $m^2 / (V · s)$.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Mobility(f64);

impl Mobility {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        // Mobility is typically magnitude, thus non-negative.
        if val < 0.0 {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Negative Mobility".into(),
            ));
        }
        Ok(Self(val))
    }
    pub fn new_unchecked(val: f64) -> Self {
        Self(val)
    }
    pub fn value(&self) -> f64 {
        self.0
    }
}
impl From<Mobility> for f64 {
    fn from(val: Mobility) -> Self {
        val.0
    }
}

/// Moiré Twist Angle ($θ$).
///
/// Units: Radians.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct TwistAngle(f64);

impl TwistAngle {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        Ok(Self(val))
    }
    pub fn value(&self) -> f64 {
        self.0
    }
    pub fn as_degrees(&self) -> f64 {
        self.0.to_degrees()
    }
    pub fn from_degrees(deg: f64) -> Self {
        Self(deg.to_radians())
    }
}
impl From<TwistAngle> for f64 {
    fn from(val: TwistAngle) -> Self {
        val.0
    }
}

/// Superconducting Order Parameter ($ψ$).
///
/// A complex scalar field describing the macroscopic condensate wavefunction.
/// *   $|ψ|^2 ≠ n_s$ (superfluid density).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct OrderParameter(Complex<f64>);

impl OrderParameter {
    pub fn new(val: Complex<f64>) -> Self {
        Self(val)
    }
    pub fn value(&self) -> Complex<f64> {
        self.0
    }
    pub fn magnitude_squared(&self) -> f64 {
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
pub struct QuantumEigenvector(CausalTensor<Complex<f64>>);

impl QuantumEigenvector {
    pub fn new(tensor: CausalTensor<Complex<f64>>) -> Self {
        Self(tensor)
    }
    pub fn inner(&self) -> &CausalTensor<Complex<f64>> {
        &self.0
    }
}

/// Quantum Velocity vector $\partial_i H |u_n➢$.
///
/// Represents the velocity operator applied to the eigenstates. Used in perturbative calculations
/// like the QGT or Kub-Greenwood conductivity.
/// *   **Rank 2 Tensor**: [basis_size, num_states].
#[derive(Debug, Clone, PartialEq)]
pub struct QuantumVelocity(CausalTensor<Complex<f64>>);

impl QuantumVelocity {
    pub fn new(tensor: CausalTensor<Complex<f64>>) -> Self {
        Self(tensor)
    }
    pub fn inner(&self) -> &CausalTensor<Complex<f64>> {
        &self.0
    }
}

/// Momentum vector $\mathbf{k}$.
///
/// Represents a point in the Brillouin Zone.
#[derive(Debug, Clone, PartialEq)]
pub struct Momentum(CausalMultiVector<f64>);

impl Default for Momentum {
    fn default() -> Self {
        Self(CausalMultiVector::new(vec![0.0], Metric::Euclidean(0)).unwrap())
    }
}

impl Momentum {
    pub fn new(mv: CausalMultiVector<f64>) -> Self {
        Self(mv)
    }
    pub fn inner(&self) -> &CausalMultiVector<f64> {
        &self.0
    }
}

/// Displacement field $\mathbf{u}(\mathbf{r})$.
///
/// Represents the mechanical displacement vector field or strain tensor components.
#[derive(Debug, Clone, PartialEq)]
pub struct Displacement(CausalTensor<f64>);

impl Displacement {
    pub fn new(tensor: CausalTensor<f64>) -> Self {
        Self(tensor)
    }
    pub fn inner(&self) -> &CausalTensor<f64> {
        &self.0
    }
}

/// Concentration field $c(\mathbf{r})$.
///
/// Represents the local concentration (mole fraction) of a species.
/// *   **Values**: Must be non-negative.
#[derive(Debug, Clone, PartialEq)]
pub struct Concentration(CausalTensor<f64>);

impl Concentration {
    pub fn new(tensor: CausalTensor<f64>) -> Result<Self, PhysicsError> {
        // Concentration cannot be negative
        for &val in tensor.as_slice() {
            if val < 0.0 {
                return Err(PhysicsError::PhysicalInvariantBroken(
                    "Negative Concentration detected".into(),
                ));
            }
        }
        Ok(Self(tensor))
    }
    /// Creates a new Concentration without validation.
    /// Use only if the tensor is guaranteed to be non-negative.
    pub fn new_unchecked(tensor: CausalTensor<f64>) -> Self {
        Self(tensor)
    }
    pub fn inner(&self) -> &CausalTensor<f64> {
        &self.0
    }
}

/// Chemical Potential Gradient $\nabla μ$.
///
/// Driving force for diffusion.
#[derive(Debug, Clone, PartialEq)]
pub struct ChemicalPotentialGradient(CausalTensor<f64>);

impl ChemicalPotentialGradient {
    pub fn new(tensor: CausalTensor<f64>) -> Self {
        Self(tensor)
    }
    pub fn inner(&self) -> &CausalTensor<f64> {
        &self.0
    }
}

/// Electromagnetic Vector Potential $\mathbf{A}$.
///
/// Used in the covariant derivative $\nabla - i\mathbf{A}$.
#[derive(Debug, Clone, PartialEq)]
pub struct VectorPotential(CausalMultiVector<f64>);

impl Default for VectorPotential {
    fn default() -> Self {
        Self(CausalMultiVector::new(vec![0.0], Metric::Euclidean(0)).unwrap())
    }
}

impl VectorPotential {
    pub fn new(mv: CausalMultiVector<f64>) -> Self {
        Self(mv)
    }
    pub fn inner(&self) -> &CausalMultiVector<f64> {
        &self.0
    }
}
