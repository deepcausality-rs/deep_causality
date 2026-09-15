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
//! # Why the distributions live here
//!
//! A distribution is mathematics. `Normal`, `Exponential`, `Cauchy` and the rest are defined by
//! their densities and their moments, and every one of those is a statistic — which is what this
//! crate is for. Putting them beside the densities is what makes importance sampling, sequential
//! Monte Carlo and MCMC diagnostics short to write: each needs a sampler and a density in the same
//! place, and separating them by a crate boundary is what made them long.
//!
//! The entropy crate below keeps what is genuinely entropy: generators, the raw machine word, the
//! Boolean draw, the Sobol sequence and the range sampler. It makes no claim about how a number is
//! distributed beyond "uniform over the bits", and it does not name a density anywhere. The
//! dependency runs `stats -> rand`, downhill, so nothing circles back.
//!
//! # A lazy sampler is Arrow-shaped, not witness-shaped
//!
//! A carrier that *stores* a sampling closure — draw-on-demand rather than draw-now — cannot take
//! the container traits in `deep_causality_haft`, and the reason is worth recording so it is not
//! rediscovered as a gap.
//!
//! Those traits carry no `'static` bound: zero occurrences across `Functor`, `Pure`, `Applicative`,
//! `Monad`, `Traversable` and `LaxMonoidal`. That absence is deliberate. A functor receives a
//! function, applies it and drops it, so the function never outlives the call. `Profunctor`, which
//! does store its functions, carries `'static` on every parameter.
//!
//! A stored closure needs bounds the container traits do not provide, and an implementation cannot
//! add them — `error[E0276]: impl has stricter requirements than trait`. The container traits are
//! for data. A lazy sampler is a program, and `haft`'s home for programs is the `Arrow` layer.
//!
//! An ensemble of *realised* draws has no such problem: it is a `Vec` with a witness, which
//! `CausalTensor` already is, and drawing into one is an ordinary generic function needing no
//! witness of its own.
//!
//! # Precision
//!
//! Every function is generic over its scalar under the algebra tower's bounds. Two of the
//! implementations this crate absorbs compute in `f64` behind a generic signature, which silently
//! discards the caller's precision; the point of the bound is that every scalar gets its own.
//!
//! Three signatures name a concrete float, and each says why at the function. The unit coordinate
//! of an inverse-CDF transform is `f64` because it is a position on `[0, 1)` rather than a value in
//! the caller's scalar, and rounding it into a narrow scalar before the transform is destructive at
//! the endpoints — see [`standard_normal_inverse_cdf_at`]. Beside it,
//! [`standard_normal_inverse_cdf`] and [`standard_normal_inverse_cdf_f106`] are the same transform
//! at one precision each, kept so that values recorded against them do not move.
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
pub mod traits;
pub mod types;
pub mod utils;
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

// ---------------------------------------------------------------------------------------------
// Distributions
// ---------------------------------------------------------------------------------------------
//
// The mathematics of a random quantity, beside the densities that describe it. The machine words
// these consume come from `deep_causality_rand`; what they mean is a statement about a real field.

pub use crate::errors::bernoulli_error::BernoulliDistributionError;
pub use crate::errors::normal_error::NormalDistributionError;
pub use crate::types::distr::bernoulli::Bernoulli;
pub use crate::types::distr::categorical::Categorical;
pub use crate::types::distr::cauchy::Cauchy;
pub use crate::types::distr::exponential::Exponential;
pub use crate::types::distr::log_normal::LogNormal;
pub use crate::types::distr::normal::Normal;
pub use crate::types::distr::normal::standard_normal::StandardNormal;
pub use crate::types::distr::poisson::{MAX_ITERATIONS, MAX_RATE, Poisson};
pub use crate::types::distr::uniform_int::UniformInt;
pub use crate::types::distr::unit_interval::standard_uniform::StandardUniform;
pub use crate::types::distr::weibull::Weibull;
pub use crate::types::range::{Open01, OpenClosed01};
pub use crate::utils::inverse_cdf::{
    bernoulli_inverse_cdf, standard_normal_inverse_cdf, standard_normal_inverse_cdf_at,
    standard_normal_inverse_cdf_f106, uniform_inverse_cdf,
};

// The generator bridge, re-exported **by name** so a crate that wants only distributions needs no
// second dependency to spell the bound. Named, never a glob: a `pub use` is an alias to the same
// trait item, so a bound written against either path is satisfied by the same implementations,
// whereas a second trait of the same shape would be a different item entirely.

pub use crate::traits::random_ext::RandomExt;
pub use deep_causality_rand::{Distribution, RandScalar, Rng, RngCore, Xoshiro256, rng};
