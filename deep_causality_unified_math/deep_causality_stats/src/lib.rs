/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Descriptive and information statistics over slices, generic in the scalar.
//!
//! # Why this crate exists
//!
//! Not to remove duplication. It adds more source than it removes, and says so in its own change
//! notes. It exists because shipped implementations of the same statistic disagree about what they
//! compute.
//!
//! Shannon entropy is the case that decides it. Three implementations in this workspace differ on
//! three axes at once — the base, whether the input is normalised, and where the cutoff for a
//! negligible entry sits. Two of them return answers in different units for the same input: one in
//! bits, one in nats, a factor of `ln 2` apart. That is not a rounding difference to be tidied
//! away, it is a semantic disagreement, and one parameterised function that can reproduce each is
//! the resolution.
//!
//! So the parameters here are not configurability for its own sake. Each combination named in
//! [`EntropyConfig`] has a caller in this workspace today.
//!
//! # Scope
//!
//! The crate implements only functions something calls. That rule is what keeps a new crate from
//! becoming a survey of the field, and it excludes several things a statistics library
//! conventionally carries: cross-entropy, mutual information, Kullback–Leibler divergence,
//! Jensen–Shannon divergence, the Hellinger distance and a general Bhattacharyya coefficient.
//!
//! Mutual information is the one that most invites an exception, because the causal-discovery
//! code computes information quantities. What it computes is specific mutual information and
//! information leak over marginalised tensor axes, which is not the slice-shaped general function;
//! building the general one would leave that version in place beside it. The Bhattacharyya case is
//! the same shape: the quantum crate computes the two-outcome Bernoulli coefficient in bits,
//! inline, which is not the general slice coefficient.
//!
//! Each is a short addition whenever a caller appears.
//!
//! # Precision
//!
//! Every function is generic over its scalar under the algebra tower's bounds, and no public
//! signature names a concrete float. Two of the implementations this crate absorbs compute in
//! `f64` behind a generic signature, which silently discards the caller's precision; the point of
//! the bound is that `f32`, `f64` and `Float106` each get their own.
//!
//! # Shape
//!
//! The surface is over slices. Anything with axes — marginalising a joint distribution, reducing a
//! tensor — belongs with the container that has them, and this crate sits below those containers
//! so they can delegate to it without a cycle.

#![cfg_attr(not(feature = "std"), no_std)]

// Unconditional, matching `deep_causality_linear`. Bazel does not read Cargo features, so a
// feature-gated `extern crate alloc` is absent under `bazel test` and every `alloc::` path in the
// crate fails to resolve.
extern crate alloc;
extern crate core;

pub mod algorithms;
pub mod errors;
pub mod types;
// Test fixtures, public because Bazel test targets cannot reach the `tests` tree, hidden because
// they are not API.
#[doc(hidden)]
pub mod utils_tests;

pub use crate::algorithms::binning::{bin_equal_frequency, bin_equal_width};
pub use crate::algorithms::correlation::{pearson, pearson_pairwise_complete};
pub use crate::algorithms::covariance::{column_means, conditional_variance, covariance_matrix};
pub use crate::algorithms::density::gaussian_log_density;
pub use crate::algorithms::entropy::{conditional_entropy, entropy};
pub use crate::algorithms::log_sum_exp::{log_add_exp, log_sum_exp};
pub use crate::algorithms::logistic::{fit_logistic, sigmoid};
pub use crate::algorithms::moments::{
    mean, population_std_dev, population_variance, std_dev, variance,
};
pub use crate::algorithms::proportion::{bernoulli_proportion, bernoulli_standard_error};
pub use crate::algorithms::ridge::{fit_ridge, fit_ridge_streaming};
pub use crate::errors::stats_error::{StatsError, StatsErrorEnum};
pub use crate::types::entropy_config::EntropyConfig;
pub use crate::types::log_base::LogBase;
pub use crate::types::logistic_config::LogisticConfig;
pub use crate::types::logistic_fit::LogisticFit;
pub use crate::types::mean_accumulator::MeanAccumulator;
pub use crate::types::normalisation::Normalisation;
pub use crate::types::penalisation::Penalisation;
pub use crate::types::ridge_config::RidgeConfig;
pub use crate::types::ridge_fit::RidgeFit;
pub use crate::types::zero_policy::ZeroPolicy;
