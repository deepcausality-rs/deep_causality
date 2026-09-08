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
    /// An iterative solver reached its iteration cap without meeting its stopping test.
    ///
    /// Distinct from [`NumericalInstability`](Self::NumericalInstability) on purpose. The solvers
    /// that raise this already use that variant for a different failure — a negative discriminant
    /// in the electroweak fixed point — and a caller that wants to retry with a wider cap or a
    /// different starting point needs to tell the two apart. The message carries the cap and the
    /// residual that was still outstanding.
    NotConverged(String),
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
    pub fn NotConverged(msg: String) -> Self {
        Self(PhysicsErrorEnum::NotConverged(msg))
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

impl From<deep_causality_linear::LinearError> for PhysicsError {
    /// Maps a linear-algebra refusal onto this crate's error.
    ///
    /// `PhysicsError` has no shape vocabulary of its own, so a mismatched shape or a bad index
    /// reads as `DimensionMismatch` and everything else — a singular system, a vanishing pivot —
    /// as `NumericalInstability`, which is what those are from a kernel's point of view.
    fn from(error: deep_causality_linear::LinearError) -> Self {
        use deep_causality_linear::LinearErrorEnum;
        let message = format!("{error}");
        match error.kind() {
            LinearErrorEnum::IndexOutOfBounds { .. }
            | LinearErrorEnum::ShapeMismatch { .. }
            | LinearErrorEnum::InnerDimensionMismatch { .. }
            | LinearErrorEnum::LengthMismatch { .. }
            | LinearErrorEnum::NotSquare { .. }
            | LinearErrorEnum::EmptyMatrix => Self::DimensionMismatch(message),
            _ => Self::NumericalInstability(message),
        }
    }
}

impl From<deep_causality_topology::TopologyError> for PhysicsError {
    /// Maps a topology refusal onto this crate's error.
    ///
    /// Added with the induction kernel's move onto `Manifold::interior_product`
    /// (`unified-math-next` task 6.7u). Shape and grade complaints read as `DimensionMismatch`;
    /// a degenerate tetrahedron, a missing metric or an absent coordinate slab are all
    /// `CalculationError`, since from a kernel's point of view the operator could not be formed.
    fn from(error: deep_causality_topology::TopologyError) -> Self {
        use deep_causality_topology::TopologyErrorEnum;
        let message = format!("{error}");
        match error.0 {
            TopologyErrorEnum::DimensionMismatch(_)
            | TopologyErrorEnum::InvalidGradeOperation(_)
            | TopologyErrorEnum::IndexOutOfBounds(_)
            | TopologyErrorEnum::SimplexNotFound => Self::DimensionMismatch(message),
            _ => Self::CalculationError(message),
        }
    }
}

impl From<deep_causality_stats::StatsError> for PhysicsError {
    /// Maps a statistics refusal onto this crate's error.
    ///
    /// `NotConverged` crosses the boundary as [`PhysicsErrorEnum::NotConverged`] rather than
    /// falling into the catch-all. Both crates draw the same line — an iterative fit that ran out
    /// of iterations is retryable at a wider cap, an unstable computation is not — and collapsing
    /// the two here would throw away the only classification a caller can act on.
    fn from(error: deep_causality_stats::StatsError) -> Self {
        use deep_causality_stats::StatsErrorEnum;
        let message = format!("{error}");
        match error.kind() {
            StatsErrorEnum::EmptyInput(_) | StatsErrorEnum::DimensionMismatch(_) => {
                Self::DimensionMismatch(message)
            }
            StatsErrorEnum::NegativeProbability(_) => Self::NormalizationError(message),
            StatsErrorEnum::NotConverged { .. } => Self::NotConverged(message),
            _ => Self::NumericalInstability(message),
        }
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
            PhysicsErrorEnum::NotConverged(msg) => {
                write!(f, "Solver did not converge: {}", msg)
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
