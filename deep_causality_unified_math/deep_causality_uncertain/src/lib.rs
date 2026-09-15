/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Uncertain values as lazy computation graphs, with the scalar as a parameter.
//!
//! # Precision is a parameter
//!
//! Every type here is generic in `R: RandScalar` — `RealField + FromPrimitive`, blanket-implemented
//! in `deep_causality_rand` and re-exported here, so a crate writing a bound over an uncertain
//! value needs no second dependency to spell it. A scalar joins by satisfying the algebra and by nothing else, so a
//! type added to `deep_causality_num` works here with no line changed in this crate. Nothing in
//! `src` names a concrete scalar outside the alias module and a display boundary.
//!
//! `Real` would not do. A probability is a ratio of two counts, and `Real` in
//! `deep_causality_algebra` is a commutative ring with an order and carries no `Div`; division
//! arrives with `Field`. `RealField` is the weakest structure that can state a frequency.
//!
//! # Two carriers over one graph
//!
//! The graph is `ConstTree<Node<R>>`, and one evaluation of a node yields a [`Sample<R>`]: a real
//! at the graph's scalar, or a truth value. Two carriers read it:
//!
//! - [`Uncertain<R>`] — a real quantity. Arithmetic, comparisons, Monte-Carlo statistics.
//! - [`UncertainBool<R>`] — a truth value. Logical operators, the SPRT gate, probability estimates.
//!
//! `Uncertain<bool>` does not exist. A Bernoulli leaf, a comparison and a logical combination all
//! read a truth value off a tree whose leaves are real, so `bool` is not something the tree can be
//! parameterised by; and the Boolean carrier keeps `R` because the tree beneath it holds `R`.
//!
//! The constraint that settles this is `core.verdict.closure`. [`Verdict`](deep_causality_algebra::Verdict)
//! is instanced twice over two different algebras — the Boolean class on the Boolean carrier
//! (`meet = &`, `join = |`, `complement = !`) and the MV class on `[0, 1]` on the real carrier
//! (`min`, `max`, `1 − p`). One type cannot hold both, and dropping either breaks `Aggregatable`
//! downstream. As two blanket instances over two distinct local types they are coherent.
//!
//! # Addressed draws
//!
//! A draw is a function of three numbers — the [`SampleSession`]'s seed, the sample index, and the
//! leaf's [ordinal](LeafOrdinals) — and of nothing else. Nothing is stored between calls, and two
//! graphs sharing a leaf agree about that leaf at the same index.

mod algos;
mod errors;
mod traits;
mod types;
mod utils;

// Algos
pub use crate::algos::hypothesis::sprt_eval;
// Errors
pub use crate::errors::UncertainError;
// Traits
pub use crate::traits::sampler::Sampler;
// The scalar bound, re-exported so a downstream bound needs one dependency rather than two.
// It is exactly the algebra: nothing is added to it here, because nothing in the graph asks for
// more — no node stores a trait object, so no `'static` reaches the scalar.
pub use deep_causality_rand::RandScalar;
// Types
pub use crate::types::computation::node::Node;
pub use crate::types::computation::operator::arithmetic_operator::ArithmeticOperator;
pub use crate::types::computation::operator::comparison_operator::ComparisonOperator;
pub use crate::types::computation::operator::logical_operator::LogicalOperator;
pub use crate::types::computation::sample::Sample;
pub use crate::types::distribution::DistributionEnum;
pub use crate::types::distribution_parameters::BernoulliParams;
pub use crate::types::distribution_parameters::NormalDistributionParams;
pub use crate::types::distribution_parameters::UniformDistributionParams;
pub use crate::types::leaf_ordinals::LeafOrdinals;
pub use crate::types::sample_session::SampleSession;
pub use crate::types::sampler::qmc_sampler::QmcSampler;
pub use crate::types::sampler::sequential_sampler::SequentialSampler;
pub use crate::types::uncertain::Uncertain;
pub use crate::types::uncertain_bool::UncertainBool;
pub use crate::types::uncertain_maybe::MaybeUncertain;
// Utils
pub(crate) use crate::utils::ratio::ratio;
pub use crate::utils::seed_mix::draw_seed;
