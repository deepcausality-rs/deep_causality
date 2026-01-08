/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use alloc::format;
use alloc::string::String;
use core::fmt::{Debug, Display, Formatter};
use deep_causality_core::{CausalityError, CausalityErrorEnum};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PhysicsError(pub PhysicsErrorEnum);

/// Detailed classification of physics-related errors.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PhysicsErrorEnum {
    // Fundamental
    /// A fundamental physical invariant (e.g., non-negative mass, speed of light limit) was violated.
    PhysicalInvariantBroken(String),
    /// Operations attempted on tensors or quantities with incompatible dimensions.
    DimensionMismatch(String),
    // Relativistic
    /// A causality violation occurred (e.g., spacelike interval for causal connection).
    CausalityViolation(String),
    /// A singularity in the spacetime metric was encountered.
    MetricSingularity(String),
    // Quantum
    /// Probability normalization failed (sum != 1 or value < 0 or > 1).
    NormalizationError(String),
    // Thermodynamics
    /// Absolute zero violations.
    ZeroKelvinViolation,
    /// Second law of thermodynamics violations.
    EntropyViolation(String),
    // Numerical
    /// Mathematical singularity (division by zero, infinite value).
    Singularity(String),
    /// Numerical instability detected (NaN, loss of precision).
    NumericalInstability(String),
    /// General calculation error.
    CalculationError(String),
    /// Metric convention error (wraps MetricError from metric crate).
    MetricConventionError(String),
    /// Topology/GaugeField structure error.
    TopologyError(String),
}

impl PhysicsError {
    pub(crate) fn new(variant: PhysicsErrorEnum) -> Self {
        Self(variant)
    }

    #[allow(non_snake_case)]
    pub fn PhysicalInvariantBroken(msg: String) -> Self {
        Self(PhysicsErrorEnum::PhysicalInvariantBroken(msg))
    }

    #[allow(non_snake_case)]
    pub fn DimensionMismatch(msg: String) -> Self {
        Self(PhysicsErrorEnum::DimensionMismatch(msg))
    }

    #[allow(non_snake_case)]
    pub fn CausalityViolation(msg: String) -> Self {
        Self(PhysicsErrorEnum::CausalityViolation(msg))
    }

    #[allow(non_snake_case)]
    pub fn MetricSingularity(msg: String) -> Self {
        Self(PhysicsErrorEnum::MetricSingularity(msg))
    }

    #[allow(non_snake_case)]
    pub fn NormalizationError(msg: String) -> Self {
        Self(PhysicsErrorEnum::NormalizationError(msg))
    }

    #[allow(non_snake_case)]
    pub fn ZeroKelvinViolation() -> Self {
        Self(PhysicsErrorEnum::ZeroKelvinViolation)
    }

    #[allow(non_snake_case)]
    pub fn EntropyViolation(msg: String) -> Self {
        Self(PhysicsErrorEnum::EntropyViolation(msg))
    }

    #[allow(non_snake_case)]
    pub fn Singularity(msg: String) -> Self {
        Self(PhysicsErrorEnum::Singularity(msg))
    }

    #[allow(non_snake_case)]
    pub fn NumericalInstability(msg: String) -> Self {
        Self(PhysicsErrorEnum::NumericalInstability(msg))
    }

    #[allow(non_snake_case)]
    pub fn CalculationError(msg: String) -> Self {
        Self(PhysicsErrorEnum::CalculationError(msg))
    }

    #[allow(non_snake_case)]
    pub fn MetricConventionError(msg: String) -> Self {
        Self(PhysicsErrorEnum::MetricConventionError(msg))
    }

    #[allow(non_snake_case)]
    pub fn TopologyError(msg: String) -> Self {
        Self(PhysicsErrorEnum::TopologyError(msg))
    }
}

// Integration with Generic CausalityError
impl From<PhysicsError> for CausalityError {
    fn from(e: PhysicsError) -> Self {
        // Wrap in Custom error until core is updated
        CausalityError::new(CausalityErrorEnum::Custom(format!("{}", e)))
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

impl From<deep_causality_metric::MetricError> for PhysicsError {
    fn from(e: deep_causality_metric::MetricError) -> Self {
        PhysicsError::new(PhysicsErrorEnum::MetricConventionError(format!("{}", e)))
    }
}

impl Display for PhysicsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match &self.0 {
            PhysicsErrorEnum::PhysicalInvariantBroken(msg) => {
                write!(f, "Physical Invariant Broken: {}", msg)
            }
            PhysicsErrorEnum::DimensionMismatch(msg) => write!(f, "Dimension Mismatch: {}", msg),
            PhysicsErrorEnum::CausalityViolation(msg) => write!(f, "Causality Violation: {}", msg),
            PhysicsErrorEnum::MetricSingularity(msg) => write!(f, "Metric Singularity: {}", msg),
            PhysicsErrorEnum::NormalizationError(msg) => write!(f, "Normalization Error: {}", msg),
            PhysicsErrorEnum::ZeroKelvinViolation => {
                write!(f, "Zero Kelvin Violation: Temperature cannot be negative")
            }
            PhysicsErrorEnum::EntropyViolation(msg) => write!(f, "Entropy Violation: {}", msg),
            PhysicsErrorEnum::Singularity(msg) => write!(f, "Singularity: {}", msg),
            PhysicsErrorEnum::NumericalInstability(msg) => {
                write!(f, "Numerical Instability: {}", msg)
            }
            PhysicsErrorEnum::CalculationError(msg) => write!(f, "Calculation Error: {}", msg),
            PhysicsErrorEnum::MetricConventionError(msg) => {
                write!(f, "Metric Convention Error: {}", msg)
            }
            PhysicsErrorEnum::TopologyError(msg) => {
                write!(f, "Topology Error: {}", msg)
            }
        }
    }
}
