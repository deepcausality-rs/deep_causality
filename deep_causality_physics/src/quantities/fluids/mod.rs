/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::PhysicsError;

/// Pressure (Pascals).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Pressure<R: deep_causality_num::RealField>(R);

impl<R: deep_causality_num::RealField> Default for Pressure<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: deep_causality_num::RealField> Pressure<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        if !val.is_finite() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Pressure must be finite".into(),
            ));
        }
        if val < R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Negative Pressure".into(),
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

impl<R: deep_causality_num::RealField + Into<f64>> From<Pressure<R>> for f64 {
    fn from(val: Pressure<R>) -> Self {
        val.0.into()
    }
}

/// Density (kg/m^3).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Density<R: deep_causality_num::RealField>(R);

impl<R: deep_causality_num::RealField> Default for Density<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: deep_causality_num::RealField> Density<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        if !val.is_finite() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Density must be finite".into(),
            ));
        }
        if val < R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Negative Density".into(),
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

impl<R: deep_causality_num::RealField + Into<f64>> From<Density<R>> for f64 {
    fn from(val: Density<R>) -> Self {
        val.0.into()
    }
}

/// Dynamic Viscosity (Pa·s).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Viscosity<R: deep_causality_num::RealField>(R);

impl<R: deep_causality_num::RealField> Default for Viscosity<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: deep_causality_num::RealField> Viscosity<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        if !val.is_finite() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Viscosity must be finite".into(),
            ));
        }
        if val < R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Negative Viscosity".into(),
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

impl<R: deep_causality_num::RealField + Into<f64>> From<Viscosity<R>> for f64 {
    fn from(val: Viscosity<R>) -> Self {
        val.0.into()
    }
}

/// Kinematic Viscosity (m^2/s). Equals dynamic viscosity divided by density.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct KinematicViscosity<R: deep_causality_num::RealField>(R);

impl<R: deep_causality_num::RealField> Default for KinematicViscosity<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: deep_causality_num::RealField> KinematicViscosity<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        if !val.is_finite() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "KinematicViscosity must be finite".into(),
            ));
        }
        if val < R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Negative KinematicViscosity".into(),
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

impl<R: deep_causality_num::RealField + Into<f64>> From<KinematicViscosity<R>> for f64 {
    fn from(val: KinematicViscosity<R>) -> Self {
        val.0.into()
    }
}

/// Specific Enthalpy (J/kg). Reference-state dependent; may be negative.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct SpecificEnthalpy<R: deep_causality_num::RealField>(R);

impl<R: deep_causality_num::RealField> Default for SpecificEnthalpy<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: deep_causality_num::RealField> SpecificEnthalpy<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        if !val.is_finite() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "SpecificEnthalpy must be finite".into(),
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

impl<R: deep_causality_num::RealField + Into<f64>> From<SpecificEnthalpy<R>> for f64 {
    fn from(val: SpecificEnthalpy<R>) -> Self {
        val.0.into()
    }
}

/// Wall Shear Stress magnitude (Pa). Stored as magnitude; sign convention is
/// carried by the calling context, not by this type.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct WallShearStress<R: deep_causality_num::RealField>(R);

impl<R: deep_causality_num::RealField> Default for WallShearStress<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: deep_causality_num::RealField> WallShearStress<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        if !val.is_finite() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "WallShearStress must be finite".into(),
            ));
        }
        if val < R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Negative WallShearStress".into(),
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

impl<R: deep_causality_num::RealField + Into<f64>> From<WallShearStress<R>> for f64 {
    fn from(val: WallShearStress<R>) -> Self {
        val.0.into()
    }
}

// =============================================================================
// Typed vector newtypes — semantic-distinction wrappers around `[R; 3]`.
//
// Constructor invariant: every component must be finite. No magnitude or sign
// constraint (any finite real triple is a valid velocity, vorticity, etc.).
// =============================================================================

/// Fluid velocity vector (m/s).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Velocity3<R: deep_causality_num::RealField>([R; 3]);

impl<R: deep_causality_num::RealField> Default for Velocity3<R> {
    fn default() -> Self {
        Self([R::zero(); 3])
    }
}

impl<R: deep_causality_num::RealField> Velocity3<R> {
    pub fn new(raw: [R; 3]) -> Result<Self, PhysicsError> {
        if !raw[0].is_finite() || !raw[1].is_finite() || !raw[2].is_finite() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Velocity3 components must be finite".into(),
            ));
        }
        Ok(Self(raw))
    }
    pub fn new_unchecked(raw: [R; 3]) -> Self {
        Self(raw)
    }
    pub fn value(&self) -> &[R; 3] {
        &self.0
    }
    pub fn into_inner(self) -> [R; 3] {
        self.0
    }
}

impl<R: deep_causality_num::RealField> From<Velocity3<R>> for [R; 3] {
    fn from(val: Velocity3<R>) -> Self {
        val.0
    }
}

/// Vorticity vector `ω = ∇ × u` (1/s). Pseudovector under spatial reflection.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VorticityVector<R: deep_causality_num::RealField>([R; 3]);

impl<R: deep_causality_num::RealField> Default for VorticityVector<R> {
    fn default() -> Self {
        Self([R::zero(); 3])
    }
}

impl<R: deep_causality_num::RealField> VorticityVector<R> {
    pub fn new(raw: [R; 3]) -> Result<Self, PhysicsError> {
        if !raw[0].is_finite() || !raw[1].is_finite() || !raw[2].is_finite() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "VorticityVector components must be finite".into(),
            ));
        }
        Ok(Self(raw))
    }
    pub fn new_unchecked(raw: [R; 3]) -> Self {
        Self(raw)
    }
    pub fn value(&self) -> &[R; 3] {
        &self.0
    }
    pub fn into_inner(self) -> [R; 3] {
        self.0
    }
}

impl<R: deep_causality_num::RealField> From<VorticityVector<R>> for [R; 3] {
    fn from(val: VorticityVector<R>) -> Self {
        val.0
    }
}

/// Acceleration vector (m/s²). Return type of momentum-equation RHS evaluators.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AccelerationVector<R: deep_causality_num::RealField>([R; 3]);

impl<R: deep_causality_num::RealField> Default for AccelerationVector<R> {
    fn default() -> Self {
        Self([R::zero(); 3])
    }
}

impl<R: deep_causality_num::RealField> AccelerationVector<R> {
    pub fn new(raw: [R; 3]) -> Result<Self, PhysicsError> {
        if !raw[0].is_finite() || !raw[1].is_finite() || !raw[2].is_finite() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "AccelerationVector components must be finite".into(),
            ));
        }
        Ok(Self(raw))
    }
    pub fn new_unchecked(raw: [R; 3]) -> Self {
        Self(raw)
    }
    pub fn value(&self) -> &[R; 3] {
        &self.0
    }
    pub fn into_inner(self) -> [R; 3] {
        self.0
    }
}

impl<R: deep_causality_num::RealField> From<AccelerationVector<R>> for [R; 3] {
    fn from(val: AccelerationVector<R>) -> Self {
        val.0
    }
}

/// Body force per unit volume (N/m³).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BodyForceDensity<R: deep_causality_num::RealField>([R; 3]);

impl<R: deep_causality_num::RealField> Default for BodyForceDensity<R> {
    fn default() -> Self {
        Self([R::zero(); 3])
    }
}

impl<R: deep_causality_num::RealField> BodyForceDensity<R> {
    pub fn new(raw: [R; 3]) -> Result<Self, PhysicsError> {
        if !raw[0].is_finite() || !raw[1].is_finite() || !raw[2].is_finite() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "BodyForceDensity components must be finite".into(),
            ));
        }
        Ok(Self(raw))
    }
    pub fn new_unchecked(raw: [R; 3]) -> Self {
        Self(raw)
    }
    pub fn value(&self) -> &[R; 3] {
        &self.0
    }
    pub fn into_inner(self) -> [R; 3] {
        self.0
    }
}

impl<R: deep_causality_num::RealField> From<BodyForceDensity<R>> for [R; 3] {
    fn from(val: BodyForceDensity<R>) -> Self {
        val.0
    }
}

// =============================================================================
// Typed rank-2 tensor newtypes — convention and algebraic-structure wrappers
// around `[[R; 3]; 3]`.
// =============================================================================

#[inline]
fn all_finite_3x3<R: deep_causality_num::RealField>(raw: &[[R; 3]; 3]) -> bool {
    raw[0][0].is_finite()
        && raw[0][1].is_finite()
        && raw[0][2].is_finite()
        && raw[1][0].is_finite()
        && raw[1][1].is_finite()
        && raw[1][2].is_finite()
        && raw[2][0].is_finite()
        && raw[2][1].is_finite()
        && raw[2][2].is_finite()
}

/// Velocity gradient tensor `∇u`. Pinned to the Jacobian convention:
/// `value[i][j] = ∂u_i / ∂x_j`. Construction-time check is finiteness only —
/// any finite 3×3 matrix is a valid velocity gradient.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VelocityGradient<R: deep_causality_num::RealField>([[R; 3]; 3]);

impl<R: deep_causality_num::RealField> Default for VelocityGradient<R> {
    fn default() -> Self {
        Self([[R::zero(); 3]; 3])
    }
}

impl<R: deep_causality_num::RealField> VelocityGradient<R> {
    pub fn new(raw: [[R; 3]; 3]) -> Result<Self, PhysicsError> {
        if !all_finite_3x3(&raw) {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "VelocityGradient components must be finite".into(),
            ));
        }
        Ok(Self(raw))
    }
    pub fn new_unchecked(raw: [[R; 3]; 3]) -> Self {
        Self(raw)
    }
    pub fn value(&self) -> &[[R; 3]; 3] {
        &self.0
    }
    pub fn into_inner(self) -> [[R; 3]; 3] {
        self.0
    }
}

impl<R: deep_causality_num::RealField> From<VelocityGradient<R>> for [[R; 3]; 3] {
    fn from(val: VelocityGradient<R>) -> Self {
        val.0
    }
}

/// Strain-rate tensor `S = 0.5·(∇u + ∇uᵀ)`. Symmetric: `S_ij = S_ji`.
/// `new` checks the symmetry invariant by exact equality, matching what natural
/// construction `0.5·(G + Gᵀ)` produces in IEEE 754. Use `new_unchecked` to
/// bypass the check in hot kernels where symmetry is guaranteed by the algebra.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StrainRateTensor<R: deep_causality_num::RealField>([[R; 3]; 3]);

impl<R: deep_causality_num::RealField> Default for StrainRateTensor<R> {
    fn default() -> Self {
        Self([[R::zero(); 3]; 3])
    }
}

impl<R: deep_causality_num::RealField> StrainRateTensor<R> {
    pub fn new(raw: [[R; 3]; 3]) -> Result<Self, PhysicsError> {
        if !all_finite_3x3(&raw) {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "StrainRateTensor components must be finite".into(),
            ));
        }
        if raw[0][1] != raw[1][0] || raw[0][2] != raw[2][0] || raw[1][2] != raw[2][1] {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "StrainRateTensor must be symmetric (S_ij == S_ji)".into(),
            ));
        }
        Ok(Self(raw))
    }
    pub fn new_unchecked(raw: [[R; 3]; 3]) -> Self {
        Self(raw)
    }
    pub fn value(&self) -> &[[R; 3]; 3] {
        &self.0
    }
    pub fn into_inner(self) -> [[R; 3]; 3] {
        self.0
    }
}

impl<R: deep_causality_num::RealField> From<StrainRateTensor<R>> for [[R; 3]; 3] {
    fn from(val: StrainRateTensor<R>) -> Self {
        val.0
    }
}

/// Rate-of-rotation (spin) tensor `Ω = 0.5·(∇u − ∇uᵀ)`. Antisymmetric:
/// `Ω_ji = −Ω_ij`, with `Ω_ii = 0`. `new` checks the antisymmetry invariant
/// by exact equality.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RotationRateTensor<R: deep_causality_num::RealField>([[R; 3]; 3]);

impl<R: deep_causality_num::RealField> Default for RotationRateTensor<R> {
    fn default() -> Self {
        Self([[R::zero(); 3]; 3])
    }
}

impl<R: deep_causality_num::RealField> RotationRateTensor<R> {
    pub fn new(raw: [[R; 3]; 3]) -> Result<Self, PhysicsError> {
        if !all_finite_3x3(&raw) {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "RotationRateTensor components must be finite".into(),
            ));
        }
        let zero = R::zero();
        if raw[0][0] != zero || raw[1][1] != zero || raw[2][2] != zero {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "RotationRateTensor diagonal must be zero (antisymmetric)".into(),
            ));
        }
        if raw[0][1] != -raw[1][0] || raw[0][2] != -raw[2][0] || raw[1][2] != -raw[2][1] {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "RotationRateTensor must be antisymmetric (Ω_ji == -Ω_ij)".into(),
            ));
        }
        Ok(Self(raw))
    }
    pub fn new_unchecked(raw: [[R; 3]; 3]) -> Self {
        Self(raw)
    }
    pub fn value(&self) -> &[[R; 3]; 3] {
        &self.0
    }
    pub fn into_inner(self) -> [[R; 3]; 3] {
        self.0
    }
}

impl<R: deep_causality_num::RealField> From<RotationRateTensor<R>> for [[R; 3]; 3] {
    fn from(val: RotationRateTensor<R>) -> Self {
        val.0
    }
}

/// Cauchy stress tensor (Pa). Symmetric, positive-in-tension sign convention.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CauchyStress<R: deep_causality_num::RealField>([[R; 3]; 3]);

impl<R: deep_causality_num::RealField> Default for CauchyStress<R> {
    fn default() -> Self {
        Self([[R::zero(); 3]; 3])
    }
}

impl<R: deep_causality_num::RealField> CauchyStress<R> {
    pub fn new(raw: [[R; 3]; 3]) -> Result<Self, PhysicsError> {
        if !all_finite_3x3(&raw) {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "CauchyStress components must be finite".into(),
            ));
        }
        if raw[0][1] != raw[1][0] || raw[0][2] != raw[2][0] || raw[1][2] != raw[2][1] {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "CauchyStress must be symmetric (σ_ij == σ_ji)".into(),
            ));
        }
        Ok(Self(raw))
    }
    pub fn new_unchecked(raw: [[R; 3]; 3]) -> Self {
        Self(raw)
    }
    pub fn value(&self) -> &[[R; 3]; 3] {
        &self.0
    }
    pub fn into_inner(self) -> [[R; 3]; 3] {
        self.0
    }
}

impl<R: deep_causality_num::RealField> From<CauchyStress<R>> for [[R; 3]; 3] {
    fn from(val: CauchyStress<R>) -> Self {
        val.0
    }
}

/// Viscous (deviatoric) stress tensor `τ` (Pa). Symmetric. Distinct from the
/// full Cauchy stress `σ = −p I + τ` — only the viscous part appears in the
/// dissipation `Φ = τ:∇u ≥ 0` and entropy-production guarantees.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViscousStress<R: deep_causality_num::RealField>([[R; 3]; 3]);

impl<R: deep_causality_num::RealField> Default for ViscousStress<R> {
    fn default() -> Self {
        Self([[R::zero(); 3]; 3])
    }
}

impl<R: deep_causality_num::RealField> ViscousStress<R> {
    pub fn new(raw: [[R; 3]; 3]) -> Result<Self, PhysicsError> {
        if !all_finite_3x3(&raw) {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "ViscousStress components must be finite".into(),
            ));
        }
        if raw[0][1] != raw[1][0] || raw[0][2] != raw[2][0] || raw[1][2] != raw[2][1] {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "ViscousStress must be symmetric (τ_ij == τ_ji)".into(),
            ));
        }
        Ok(Self(raw))
    }
    pub fn new_unchecked(raw: [[R; 3]; 3]) -> Self {
        Self(raw)
    }
    pub fn value(&self) -> &[[R; 3]; 3] {
        &self.0
    }
    pub fn into_inner(self) -> [[R; 3]; 3] {
        self.0
    }
}

impl<R: deep_causality_num::RealField> From<ViscousStress<R>> for [[R; 3]; 3] {
    fn from(val: ViscousStress<R>) -> Self {
        val.0
    }
}

/// Reynolds stress tensor `R_ij = ⟨u'_i u'_j⟩` (Pa, after multiplication by ρ
/// in caller; here a kinematic Reynolds stress in m²/s² is also acceptable).
/// Symmetric. Diagonal entries are non-negative (variances) — *not* enforced
/// by the newtype to keep the constructor cheap; callers passing a tensor
/// that violates the diagonal-positivity property are responsible for
/// downstream interpretation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReynoldsStress<R: deep_causality_num::RealField>([[R; 3]; 3]);

impl<R: deep_causality_num::RealField> Default for ReynoldsStress<R> {
    fn default() -> Self {
        Self([[R::zero(); 3]; 3])
    }
}

impl<R: deep_causality_num::RealField> ReynoldsStress<R> {
    pub fn new(raw: [[R; 3]; 3]) -> Result<Self, PhysicsError> {
        if !all_finite_3x3(&raw) {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "ReynoldsStress components must be finite".into(),
            ));
        }
        if raw[0][1] != raw[1][0] || raw[0][2] != raw[2][0] || raw[1][2] != raw[2][1] {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "ReynoldsStress must be symmetric (R_ij == R_ji)".into(),
            ));
        }
        Ok(Self(raw))
    }
    pub fn new_unchecked(raw: [[R; 3]; 3]) -> Self {
        Self(raw)
    }
    pub fn value(&self) -> &[[R; 3]; 3] {
        &self.0
    }
    pub fn into_inner(self) -> [[R; 3]; 3] {
        self.0
    }
}

impl<R: deep_causality_num::RealField> From<ReynoldsStress<R>> for [[R; 3]; 3] {
    fn from(val: ReynoldsStress<R>) -> Self {
        val.0
    }
}
