/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::PhysicsError;
use deep_causality_num::{Complex, RealField};
use deep_causality_tensor::CausalTensor;

/// Focal Length ($f$).
/// Unit: Meters. Constraint: None (can be negative for diverging lens).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct FocalLength<R: deep_causality_num::RealField>(R);

impl<R: deep_causality_num::RealField> Default for FocalLength<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: deep_causality_num::RealField> FocalLength<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        Ok(Self(val))
    }
    pub fn value(&self) -> R {
        self.0
    }
}

impl<R: deep_causality_num::RealField + Into<f64>> From<FocalLength<R>> for f64 {
    fn from(val: FocalLength<R>) -> Self {
        val.0.into()
    }
}

/// Optical Power ($D = 1/f$).
/// Unit: Diopters ($m^{-1}$). Constraint: None.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct OpticalPower<R: deep_causality_num::RealField>(R);

impl<R: deep_causality_num::RealField> Default for OpticalPower<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: deep_causality_num::RealField> OpticalPower<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        Ok(Self(val))
    }
    pub fn value(&self) -> R {
        self.0
    }
}

impl<R: deep_causality_num::RealField + Into<f64>> From<OpticalPower<R>> for f64 {
    fn from(val: OpticalPower<R>) -> Self {
        val.0.into()
    }
}

/// Wavelength ($\lambda$).
/// Unit: Meters. Constraint: > 0.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Wavelength<R: deep_causality_num::RealField>(R);

impl<R: deep_causality_num::RealField> Default for Wavelength<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: deep_causality_num::RealField> Wavelength<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        if val <= R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Wavelength must be positive".into(),
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

impl<R: deep_causality_num::RealField + Into<f64>> From<Wavelength<R>> for f64 {
    fn from(val: Wavelength<R>) -> Self {
        val.0.into()
    }
}

/// Numerical Aperture ($NA = n \sin \theta$).
/// Unit: Dimensionless. Constraint: > 0.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct NumericalAperture<R: deep_causality_num::RealField>(R);

impl<R: deep_causality_num::RealField> Default for NumericalAperture<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: deep_causality_num::RealField> NumericalAperture<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        if val <= R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Numerical Aperture must be positive".into(),
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

impl<R: deep_causality_num::RealField + Into<f64>> From<NumericalAperture<R>> for f64 {
    fn from(val: NumericalAperture<R>) -> Self {
        val.0.into()
    }
}

/// Beam Waist ($w_0$). Minimum radius of Gaussian beam.
/// Unit: Meters. Constraint: > 0.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct BeamWaist<R: deep_causality_num::RealField>(R);

impl<R: deep_causality_num::RealField> Default for BeamWaist<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: deep_causality_num::RealField> BeamWaist<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        if val <= R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Beam Waist must be positive".into(),
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

impl<R: deep_causality_num::RealField + Into<f64>> From<BeamWaist<R>> for f64 {
    fn from(val: BeamWaist<R>) -> Self {
        val.0.into()
    }
}

/// Ray Height ($y$). Distance from optical axis.
/// Unit: Meters.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct RayHeight<R: deep_causality_num::RealField>(R);

impl<R: deep_causality_num::RealField> Default for RayHeight<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: deep_causality_num::RealField> RayHeight<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        Ok(Self(val))
    }
    pub fn value(&self) -> R {
        self.0
    }
}

impl<R: deep_causality_num::RealField + Into<f64>> From<RayHeight<R>> for f64 {
    fn from(val: RayHeight<R>) -> Self {
        val.0.into()
    }
}

/// Ray Angle ($\theta$). Angle relative to optical axis.
/// Unit: Radians.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct RayAngle<R: deep_causality_num::RealField>(R);

impl<R: deep_causality_num::RealField> Default for RayAngle<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: deep_causality_num::RealField> RayAngle<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        Ok(Self(val))
    }
    pub fn value(&self) -> R {
        self.0
    }
}

impl<R: deep_causality_num::RealField + Into<f64>> From<RayAngle<R>> for f64 {
    fn from(val: RayAngle<R>) -> Self {
        val.0.into()
    }
}

/// ABCD Matrix. $2 \times 2$ Ray Transfer Matrix.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AbcdMatrix<R: deep_causality_num::RealField>(CausalTensor<R>);

impl<R: deep_causality_num::RealField> AbcdMatrix<R> {
    pub fn new(tensor: CausalTensor<R>) -> Self {
        Self(tensor)
    }
    pub fn inner(&self) -> &CausalTensor<R> {
        &self.0
    }
}

/// Jones Vector. Polarized Electric Field. Rank 1, Dim 2 Complex Tensor.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct JonesVector<R: deep_causality_num::RealField>(CausalTensor<Complex<R>>);

impl<R: deep_causality_num::RealField> JonesVector<R> {
    pub fn new(tensor: CausalTensor<Complex<R>>) -> Self {
        Self(tensor)
    }
    pub fn inner(&self) -> &CausalTensor<Complex<R>> {
        &self.0
    }
}

/// Stokes Vector. Intensity vector $(S_0, S_1, S_2, S_3)$. Rank 1, Dim 4 Tensor.
/// Constraint: $S_0^2 \ge S_1^2 + S_2^2 + S_3^2$.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct StokesVector<R: deep_causality_num::RealField>(CausalTensor<R>);

impl<R: deep_causality_num::RealField> StokesVector<R> {
    pub fn new(tensor: CausalTensor<R>) -> Result<Self, PhysicsError> {
        if tensor.shape() != [4] {
            return Err(PhysicsError::DimensionMismatch(
                "StokesVector must be a 4-element tensor".into(),
            ));
        }
        let s = tensor.data();
        let s0_sq = s[0] * s[0];
        let s_vec_sq = s[1] * s[1] + s[2] * s[2] + s[3] * s[3];

        if s0_sq < s_vec_sq {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "StokesVector invariant violated: S0^2 < S1^2 + S2^2 + S3^2".into(),
            ));
        }

        Ok(Self(tensor))
    }

    pub fn inner(&self) -> &CausalTensor<R> {
        &self.0
    }
}

/// Complex Beam Parameter ($q(z) = z + i z_R$).
/// Constraint: $\text{Im}(q) > 0$.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ComplexBeamParameter<R: deep_causality_num::RealField>(Complex<R>);

impl<R: deep_causality_num::RealField> ComplexBeamParameter<R> {
    pub fn new(val: Complex<R>) -> Result<Self, PhysicsError> {
        if val.im <= R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Imaginary part of q (Rayleigh range) must be positive".into(),
            ));
        }
        Ok(Self(val))
    }
    pub fn new_unchecked(val: Complex<R>) -> Self {
        Self(val)
    }
    pub fn value(&self) -> Complex<R> {
        self.0
    }
}

/// Index of refraction for a medium (ratio of c to phase velocity in the medium).
/// Typically > 1; negative values are physically possible in metamaterials but
/// zero is rejected to prevent division errors in downstream calculations.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct IndexOfRefraction<R: RealField>(R);

impl<R: RealField> Default for IndexOfRefraction<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: RealField> IndexOfRefraction<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        if val == R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Index of Refraction cannot be zero".into(),
            ));
        }
        Ok(Self(val))
    }
    pub fn value(&self) -> R {
        self.0
    }
}

impl<R: RealField + Into<f64>> From<IndexOfRefraction<R>> for f64 {
    fn from(val: IndexOfRefraction<R>) -> Self {
        val.0.into()
    }
}
