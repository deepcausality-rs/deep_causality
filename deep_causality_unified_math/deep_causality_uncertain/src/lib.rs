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
//! # Ensembles, and how they compose
//!
//! `n` draws of one quantity go into a container the **caller** names:
//! [`Uncertain::materialize`] takes a witness type parameter and hands back `W::Type<R>` — a
//! `DenseVector<R>`, a rank-1 `CausalTensor<R>`, a `Vec<R>`. This crate declares no ensemble type
//! of its own and names neither container crate; the ensemble arrives already carrying every
//! categorical structure its witness provides.
//!
//! **Correlated quantities pair by index.** Two ensembles drawn from one session at the *same*
//! indices — [`Uncertain::materialize_at`] rather than [`Uncertain::materialize`], which advances —
//! are correlated: draw `i` of one belongs with draw `i` of the other, because both were drawn at
//! sample `i`. Combine them with a **positional zip**: `Semigroupal::zip_with` on
//! `ZipDenseVectorWitness` or `ZipTensorWitness`, and `DiagonalTraversable::sequence_zip` to turn a
//! structure of ensembles inside out. Neither zip witness has `Pure` — the unit of a positional zip
//! is the infinite repeat — so `sequence_zip` takes its accumulator as a parameter, which for
//! sampling is right: the ensemble size is declared rather than inferred.
//!
//! **`Traversable::sequence` is the hazard, not the tool.** It forms the *cartesian product* across
//! quantities, which for sampling is almost never what is meant: four quantities at 50 draws each
//! give 50⁴ = 6 250 000 combinations rather than 50 correlated tuples. Every combination is
//! individually well-formed, so the count is the only symptom. Reach for the diagonal.
//!
//! ## The memory an ensemble costs, and the scalar that changes it
//!
//! An ensemble per cell of a field is bounded by *cells × draws × width*, and that product reaches
//! the ceiling before any one factor looks large. The **width** is the factor the caller chooses,
//! and it is the reason the scalar being a parameter is worth something here rather than merely
//! tidy:
//!
//! | cells                  | draws | `f64` (8 B) | `f32` (4 B) | `BFloat16` (2 B) |
//! |------------------------|-------|-------------|-------------|------------------|
//! | 256³ = 16 777 216      | 1000  | 134 GB      | 67 GB       | **34 GB**        |
//! | 512³ = 134 217 728     | 1000  | 1.07 TB     | 537 GB      | **268 GB**       |
//! | 256³                   | 32    | 4.3 GB      | 2.1 GB      | **1.1 GB**       |
//!
//! **`BFloat16` is the interesting column, because it trades range for little loss.** It carries 8
//! exponent bits with f32's bias — so `MIN_POSITIVE` is 2⁻¹²⁶ in *both*, and its `MAX` of 3.3895e38
//! is within 0.4% of f32's 3.4028e38, short only because the significand is. What it trades is
//! precision alone: epsilon 7.8125e-3 against f32's 1.1921e-7. So wherever a quantity's **numerical
//! range** fits inside f32 — which is the usual case, and is what overflow and underflow actually
//! test — `BFloat16` stands in for `f32` at half the memory and for `f64` at a quarter the memory.
//!
//! For an *ensemble* that trade is better still, because the draws already carry sampling noise
//! much larger than the rounding error. A `BFloat16` half-ulp is a relative error of 2⁻⁸ = 0.39%; the
//! Monte-Carlo standard error of an `n`-draw mean is 1/√n of the quantity's own spread — 3.2% at
//! n = 1000, 1.0% at n = 10 000. The two meet at **n = 65 536**. Below that the ensemble's own
//! statistical error dominates its storage rounding, so carrying the draws at `BFloat16` costs
//! nothing in terms of error, but cuts memory usage significantly.
//!
//! This is about *storing* draws: reducing them is a separate question,
//! and a left-to-right sum at `BFloat16` stagnates badly enough to return 32.75
//! for a mean of 100 — which is why every reduction in `deep_causality_stats` sums as a balanced
//! tree. And a spread estimated from `BFloat16` draws loses more than a mean does, because a
//! deviation is a difference of similar quantities; measured, `std_dev` of N(100, 5) recovers about
//! 5.03 and of N(0, 1) about 1.008, so roughly a percent rather than the mean's exactness.
//!
//! **The shape that stays bounded** inverts the loop regardless of scalar: materialise the
//! uncertain *inputs* — a handful of quantities, not a field of them — traverse them diagonally,
//! and evaluate the field once per draw, reducing as you go. Memory is then one field plus one
//! accumulator, independent of the ensemble size.
//!
//! # Addressed draws
//!
//! A draw is a function of three numbers — the [`SampleSession`]'s seed, the sample index, and the
//! leaf's [ordinal](LeafOrdinals) — and of nothing else. Nothing is stored between calls, and two
//! graphs sharing a leaf agree about that leaf at the same index.

mod algos;
mod errors;
pub mod extensions;
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
