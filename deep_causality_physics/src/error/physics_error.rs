/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use alloc::format;
use alloc::string::String;
use core::fmt::{Debug, Display, Formatter};
use deep_causality_core::{CausalityError, CausalityErrorEnum};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]

pub struct PhysicsError(pub PhysicsErrorEnum);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]

pub enum PhysicsErrorEnum {
    // Fundamental
    PhysicalInvariantBroken(String),
    DimensionMismatch(String),

    // Relativistic
    CausalityViolation(String),
    MetricSingularity(String),

    // Quantum
    NormalizationError(String),

    // Thermodynamics
    ZeroKelvinViolation,
    EntropyViolation(String),

    // Numerical
    Singularity(String),
    NumericalInstability(String),
    CalculationError(String),
}

impl Display for PhysicsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for PhysicsError {}

impl PhysicsError {
    pub fn new(variant: PhysicsErrorEnum) -> Self {
        Self(variant)
    }
}

// Integration with Generic CausalityError
impl From<PhysicsError> for CausalityError {
    fn from(e: PhysicsError) -> Self {
        // Wrap in Custom error until core is updated
        CausalityError::new(CausalityErrorEnum::Custom(format!("PhysicsError: {:?}", e)))
    }
}

impl From<deep_causality_tensor::CausalTensorError> for PhysicsError {
    fn from(e: deep_causality_tensor::CausalTensorError) -> Self {
        PhysicsError::new(PhysicsErrorEnum::Singularity(format!(
            "Tensor Error: {:?}",
            e
        )))
    }
}
