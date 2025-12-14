/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{PhysicsError, PhysicsErrorEnum};
use deep_causality_num::Complex;
use deep_causality_tensor::CausalTensor;

/// Focal Length ($f$).
/// Unit: Meters. Constraint: None (can be negative for diverging lens).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct FocalLength(f64);

impl FocalLength {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        Ok(Self(val))
    }
    pub fn value(&self) -> f64 {
        self.0
    }
}

/// Optical Power ($D = 1/f$).
/// Unit: Diopters ($m^{-1}$). Constraint: None.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct OpticalPower(f64);

impl OpticalPower {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        Ok(Self(val))
    }
    pub fn value(&self) -> f64 {
        self.0
    }
}

/// Wavelength ($\lambda$).
/// Unit: Meters. Constraint: > 0.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Wavelength(f64);

impl Wavelength {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if val <= 0.0 {
            return Err(PhysicsError::new(
                PhysicsErrorEnum::PhysicalInvariantBroken("Wavelength must be positive".into()),
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

/// Numerical Aperture ($NA = n \sin \theta$).
/// Unit: Dimensionless. Constraint: > 0.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct NumericalAperture(f64);

impl NumericalAperture {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if val <= 0.0 {
            return Err(PhysicsError::new(
                PhysicsErrorEnum::PhysicalInvariantBroken(
                    "Numerical Aperture must be positive".into(),
                ),
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

/// Beam Waist ($w_0$). Minimum radius of Gaussian beam.
/// Unit: Meters. Constraint: > 0.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct BeamWaist(f64);

impl BeamWaist {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if val <= 0.0 {
            return Err(PhysicsError::new(
                PhysicsErrorEnum::PhysicalInvariantBroken("Beam Waist must be positive".into()),
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

/// Ray Height ($y$). Distance from optical axis.
/// Unit: Meters.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct RayHeight(f64);

impl RayHeight {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        Ok(Self(val))
    }
    pub fn value(&self) -> f64 {
        self.0
    }
}

/// Ray Angle ($\theta$). Angle relative to optical axis.
/// Unit: Radians.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct RayAngle(f64);

impl RayAngle {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        Ok(Self(val))
    }
    pub fn value(&self) -> f64 {
        self.0
    }
}

/// ABCD Matrix. $2 \times 2$ Ray Transfer Matrix.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AbcdMatrix(CausalTensor<f64>);

impl AbcdMatrix {
    pub fn new(tensor: CausalTensor<f64>) -> Self {
        Self(tensor)
    }
    pub fn inner(&self) -> &CausalTensor<f64> {
        &self.0
    }
}

/// Jones Vector. Polarized Electric Field. Rank 1, Dim 2 Complex Tensor.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct JonesVector(CausalTensor<Complex<f64>>);

impl JonesVector {
    pub fn new(tensor: CausalTensor<Complex<f64>>) -> Self {
        Self(tensor)
    }
    pub fn inner(&self) -> &CausalTensor<Complex<f64>> {
        &self.0
    }
}

/// Stokes Vector. Intensity vector $(S_0, S_1, S_2, S_3)$. Rank 1, Dim 4 Tensor.
/// Constraint: $S_0^2 \ge S_1^2 + S_2^2 + S_3^2$.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct StokesVector(CausalTensor<f64>);

impl StokesVector {
    pub fn new(tensor: CausalTensor<f64>) -> Result<Self, PhysicsError> {
        if tensor.shape() != [4] {
            return Err(PhysicsError::new(PhysicsErrorEnum::DimensionMismatch(
                "StokesVector must be a 4-element tensor".into(),
            )));
        }
        let s = tensor.data();
        let s0_sq = s[0] * s[0];
        let s_vec_sq = s[1] * s[1] + s[2] * s[2] + s[3] * s[3];

        if s0_sq < s_vec_sq {
            return Err(PhysicsError::new(
                PhysicsErrorEnum::PhysicalInvariantBroken(
                    "StokesVector invariant violated: S0^2 < S1^2 + S2^2 + S3^2".into(),
                ),
            ));
        }

        Ok(Self(tensor))
    }

    pub fn inner(&self) -> &CausalTensor<f64> {
        &self.0
    }
}

/// Complex Beam Parameter ($q(z) = z + i z_R$).
/// Constraint: $\text{Im}(q) > 0$.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ComplexBeamParameter(Complex<f64>);

impl ComplexBeamParameter {
    pub fn new(val: Complex<f64>) -> Result<Self, PhysicsError> {
        if val.im <= 0.0 {
            return Err(PhysicsError::new(
                PhysicsErrorEnum::PhysicalInvariantBroken(
                    "Imaginary part of q (Rayleigh range) must be positive".into(),
                ),
            ));
        }
        Ok(Self(val))
    }
    pub fn new_unchecked(val: Complex<f64>) -> Self {
        Self(val)
    }
    pub fn value(&self) -> Complex<f64> {
        self.0
    }
}
