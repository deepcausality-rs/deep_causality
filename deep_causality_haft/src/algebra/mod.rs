/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Algebraic Functional Traits.
//!
//! This module contains the core abstract behaviors (traits) derived from Category Theory and functional programming.
//! These traits abstract over the *structure* and *manipulation* of data types, allowing for generic algorithms
//! that work across different containers and computation contexts.
//!
//! # Categories of Traits
//!
//! ## Mapping (Functors)
//! Traits that deal with transforming values inside a context.
//!
//! *   [`Functor`](crate::algebra::functor::Functor): Maps a function `A -> B` over a structure `F<A>` to produce `F<B>`.
//! *   [`Bifunctor`](crate::algebra::bifunctor::Bifunctor): Maps over two types simultaneously (e.g., `Result<A, B>`).
//! *   [`Profunctor`](crate::algebra::profunctor::Profunctor): Contravariant in the first argument, covariant in the second (e.g., functions `A -> B`).
//! *   [`RiemannMap`](crate::algebra::riemann_map::RiemannMap): High-arity mapping for geometric structures (Curvature, Scattering).
//!
//! ## Monadic (Computation & Context)
//! Traits that model computations, side effects, and context dependency.
//!
//! *   [`Monad`](crate::algebra::monad::Monad): Sequences computations (`bind`/`flat_map`).
//! *   [`Applicative`](crate::algebra::applicative::Applicative): Applies functions wrapped in a context to values wrapped in a context.
//! *   [`Comonad`](crate::algebra::comonad::CoMonad): Context-dependent computation (the dual of Monad). Extracts values and extends context.
//! *   [`ParametricMonad`](crate::algebra::parametric_monad::ParametricMonad): Indexed Monad where the state type changes during computation.
//! *   [`Promonad`](crate::algebra::promonad::Promonad): Profunctor Monad (Arrows), modeling input/output processes with fusion.
//!
//! ## Structural (folding & Traversal)
//! Traits that deal with the shape and aggregation of data structures.
//!
//! *   [`Foldable`](crate::algebra::foldable::Foldable): Reduces a structure to a single value (`fold`).
//! *   [`Traversable`](crate::algebra::traversable::Traversable): Traverses a structure with an effectful function (swaps layers, e.g., `Vec<Option<T>>` -> `Option<Vec<T>>`).
//! *   [`Adjunction`](crate::algebra::adjunction::Adjunction): A relationship between two functors (Left and Right adjoints).
//! *   [`CyberneticLoop`](crate::algebra::cybernetic_loop::CyberneticLoop): Models a 5-component feedback loop system.
//!
//! # Usage
//!
//! To use these traits, you typically need a type that implements the corresponding HKT witness (from `core` or `extensions`).
//!
//! ```rust
//! use deep_causality_haft::Functor;
//! use deep_causality_haft::VecWitness;
//!
//! let v = vec![1, 2, 3];
//! let v_mapped = VecWitness::fmap(v, |x| x * 2);
//! assert_eq!(v_mapped, vec![2, 4, 6]);
//! ```

pub mod adjunction;
pub mod applicative;
pub mod bifunctor;
pub mod comonad;
pub mod cybernetic_loop;
pub mod foldable;
pub mod functor;
pub mod monad;
pub mod parametric_monad;
pub mod profunctor;
pub mod promonad;
pub mod riemann_map;
pub mod traversable;
