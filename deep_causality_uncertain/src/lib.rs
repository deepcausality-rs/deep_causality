/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Core components for the `deep_causality_uncertain` crate.

mod algos;
mod errors;
mod traits;
mod types;
pub mod utils_tests;
mod alias;

// Algos
pub use crate::algos::hypothesis::sprt_eval;
// Alias
pub use crate::alias::{MaybeUncertainBool, MaybeUncertainF64};
// Errors
pub use crate::errors::UncertainError;
// Traits
pub use crate::traits::sampler::Sampler;
// Types
pub use crate::types::cache::{GlobalSampleCache, SampledValue, with_global_cache};
pub use crate::types::computation::node::ComputationNode;
pub use crate::types::computation::node_id::NodeId;
pub use crate::types::computation::{ArithmeticOperator, ComparisonOperator, LogicalOperator};
pub use crate::types::distribution::DistributionEnum;
pub use crate::types::distribution_parameters::BernoulliParams;
pub use crate::types::distribution_parameters::NormalDistributionParams;
pub use crate::types::distribution_parameters::UniformDistributionParams;
pub use crate::types::sampler::SequentialSampler;
pub use crate::types::uncertain::Uncertain;
pub use crate::types::uncertain_maybe::MaybeUncertain;
