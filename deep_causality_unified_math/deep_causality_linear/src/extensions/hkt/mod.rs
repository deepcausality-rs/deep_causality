/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Higher-kinded witnesses, one per container.
//!
//! Uniform composition across the mathematical crates is why `deep_causality_haft` exists, and these
//! containers are the ones a caller composes with `CausalTensor` and `CausalTensorTrain`. A
//! container that implemented some of the trait set and not the rest would compose in some pipelines
//! and not others, which is worse than being uniformly absent.
//!
//! `CsrMatrixWitness` already implements `HKT`, `Functor`, `Foldable`, `Pure`, `Applicative`,
//! `Monad`, `CoMonad` and `Adjunction`. The three new witnesses match it, or document at the impl
//! site which member they cannot support and why.
//!
//! # `PackedGf2` has no witness, and that is the documented shortfall
//!
//! The other three are generic in their element type, so `Type<T>` projects to a container of `T`.
//! `PackedGf2` is not: its element type is fixed to [`Gf2`](deep_causality_num::Gf2) by its storage,
//! which is one bit per entry. There is no `PackedGf2<T>` for an arbitrary `T` to project to, so
//! `HKT` cannot be stated for it — `fmap` with a function returning `f64` would have nowhere to put
//! the result.
//!
//! A caller who wants to map over an 𝔽₂ matrix unpacks to `DenseMatrix<Gf2>`, which has a witness,
//! and the conversion is explicit. This is a consequence of the packing decision rather than an
//! oversight, and it is recorded here rather than left for someone to rediscover.

pub mod csr_matrix_witness;
pub mod dense_matrix_witness;
pub mod dense_vector_witness;
