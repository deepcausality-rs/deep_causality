/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Core components for the `deep_causality_uncertain` crate.

mod algos;
mod errors;
mod traits;
mod types;

// Re-export key types for easier access.
// Algorithms
pub use algos::hypothesis::sprt_eval;
// Errors
pub use crate::errors::UncertainError;
// Traits
pub use crate::traits::sampler::Sampler;
// types
pub use crate::types::cache::{SampledValue, get_global_cache};
pub use crate::types::computation::{
    ArithmeticOperator, ComparisonOperator, ComputationNode, LogicalOperator,
};
pub use crate::types::distribution::DistributionEnum;
pub use crate::types::distribution_parameters::BernoulliParams;
pub use crate::types::distribution_parameters::NormalDistributionParams;
pub use crate::types::distribution_parameters::UniformDistributionParams;
pub use crate::types::sampler::SequentialSampler;
pub use crate::types::uncertain::Uncertain;
