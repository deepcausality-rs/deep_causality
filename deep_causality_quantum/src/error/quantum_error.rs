/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use core::fmt::{Debug, Display, Formatter};
use deep_causality_core::{CausalityError, CausalityErrorEnum};

/// The crate-local quantum error: an outer newtype over [`QuantumErrorEnum`],
/// mirroring the repo convention (`CausalityError(CausalityErrorEnum::…)`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QuantumError(pub QuantumErrorEnum);

/// Detailed classification of quantum errors. Typed variants name the exact
/// failure; a `String` payload carries the operation-specific context.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum QuantumErrorEnum {
    /// Operations attempted on states or operators with incompatible dimensions/shapes.
    DimensionMismatch(String),
    /// The operands carry different Clifford metric signatures.
    MetricMismatch(String),
    /// The Clifford metric is unsupported for the requested operation
    /// (e.g. an odd-dimensional metric for the ket↔matrix bridge), or a
    /// metric convention error surfaced from `deep_causality_metric`.
    UnsupportedMetric(String),
    /// A non-finite value (NaN, ±inf) was produced or encountered.
    NonFiniteValue(String),
    /// Probability normalization failed (value < 0, > 1, or sum ≠ 1).
    NormalizationError(String),
    /// An operator required to be positive (semi-)definite is not.
    NonPositiveOperator(String),
    /// A density or Choi–Jamiołkowski operator does not have the required trace.
    NonUnitTrace(String),
    /// A channel is not completely positive and trace-preserving.
    NonCptpChannel(String),
    /// A partial trace was requested with an inconsistent subsystem shape.
    PartialTraceShape(String),
    /// The freeze-time quantum Markov check found a non-commuting factor pair;
    /// `node_j`/`node_k` name the offending operators by graph node index.
    CommutatorNonZero {
        node_j: usize,
        node_k: usize,
        detail: String,
    },
    /// The declared causal structure contains a `C₃` sub-relation and therefore
    /// has no traditional-circuit causally faithful decomposition
    /// (van der Lugt & Lorenz, arXiv:2508.11762, Thm 3.2).
    NotFaithfullyRepresentable(String),
    /// Numerical conversion or general calculation failure.
    CalculationError(String),
}

impl QuantumError {
    pub(crate) fn new(variant: QuantumErrorEnum) -> Self {
        Self(variant)
    }

    #[allow(non_snake_case)]
    pub fn DimensionMismatch(msg: String) -> Self {
        Self(QuantumErrorEnum::DimensionMismatch(msg))
    }

    #[allow(non_snake_case)]
    pub fn MetricMismatch(msg: String) -> Self {
        Self(QuantumErrorEnum::MetricMismatch(msg))
    }

    #[allow(non_snake_case)]
    pub fn UnsupportedMetric(msg: String) -> Self {
        Self(QuantumErrorEnum::UnsupportedMetric(msg))
    }

    #[allow(non_snake_case)]
    pub fn NonFiniteValue(msg: String) -> Self {
        Self(QuantumErrorEnum::NonFiniteValue(msg))
    }

    #[allow(non_snake_case)]
    pub fn NormalizationError(msg: String) -> Self {
        Self(QuantumErrorEnum::NormalizationError(msg))
    }

    #[allow(non_snake_case)]
    pub fn NonPositiveOperator(msg: String) -> Self {
        Self(QuantumErrorEnum::NonPositiveOperator(msg))
    }

    #[allow(non_snake_case)]
    pub fn NonUnitTrace(msg: String) -> Self {
        Self(QuantumErrorEnum::NonUnitTrace(msg))
    }

    #[allow(non_snake_case)]
    pub fn NonCptpChannel(msg: String) -> Self {
        Self(QuantumErrorEnum::NonCptpChannel(msg))
    }

    #[allow(non_snake_case)]
    pub fn PartialTraceShape(msg: String) -> Self {
        Self(QuantumErrorEnum::PartialTraceShape(msg))
    }

    #[allow(non_snake_case)]
    pub fn CommutatorNonZero(node_j: usize, node_k: usize, detail: String) -> Self {
        Self(QuantumErrorEnum::CommutatorNonZero {
            node_j,
            node_k,
            detail,
        })
    }

    #[allow(non_snake_case)]
    pub fn NotFaithfullyRepresentable(msg: String) -> Self {
        Self(QuantumErrorEnum::NotFaithfullyRepresentable(msg))
    }

    #[allow(non_snake_case)]
    pub fn CalculationError(msg: String) -> Self {
        Self(QuantumErrorEnum::CalculationError(msg))
    }
}

// Integration with the generic CausalityError, mirroring the physics crate.
impl From<QuantumError> for CausalityError {
    fn from(e: QuantumError) -> Self {
        CausalityError::new(CausalityErrorEnum::Custom(format!("{}", e)))
    }
}

impl From<deep_causality_metric::MetricError> for QuantumError {
    fn from(e: deep_causality_metric::MetricError) -> Self {
        QuantumError::new(QuantumErrorEnum::UnsupportedMetric(format!("{}", e)))
    }
}

impl core::error::Error for QuantumError {}

impl Display for QuantumError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match &self.0 {
            QuantumErrorEnum::DimensionMismatch(msg) => write!(f, "Dimension Mismatch: {}", msg),
            QuantumErrorEnum::MetricMismatch(msg) => write!(f, "Metric Mismatch: {}", msg),
            QuantumErrorEnum::UnsupportedMetric(msg) => write!(f, "Unsupported Metric: {}", msg),
            QuantumErrorEnum::NonFiniteValue(msg) => write!(f, "Non-Finite Value: {}", msg),
            QuantumErrorEnum::NormalizationError(msg) => {
                write!(f, "Normalization Error: {}", msg)
            }
            QuantumErrorEnum::NonPositiveOperator(msg) => {
                write!(f, "Non-Positive Operator: {}", msg)
            }
            QuantumErrorEnum::NonUnitTrace(msg) => write!(f, "Non-Unit Trace: {}", msg),
            QuantumErrorEnum::NonCptpChannel(msg) => write!(f, "Non-CPTP Channel: {}", msg),
            QuantumErrorEnum::PartialTraceShape(msg) => {
                write!(f, "Partial Trace Shape Error: {}", msg)
            }
            QuantumErrorEnum::CommutatorNonZero {
                node_j,
                node_k,
                detail,
            } => write!(
                f,
                "Non-Zero Commutator: factors at nodes {} and {} do not commute: {}",
                node_j, node_k, detail
            ),
            QuantumErrorEnum::NotFaithfullyRepresentable(msg) => {
                write!(f, "Not Faithfully Representable (C3 obstruction): {}", msg)
            }
            QuantumErrorEnum::CalculationError(msg) => write!(f, "Calculation Error: {}", msg),
        }
    }
}
