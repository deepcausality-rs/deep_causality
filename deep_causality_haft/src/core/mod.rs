/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Core Higher-Kinded Type (HKT) Machinery.
//!
//! This module provides the foundational traits and types required to emulate Higher-Kinded Types (HKTs)
//! in Rust, a language that does not natively support them. It uses the "Generic Associated Types (GAT) Pattern"
//! to achieve this.
//!
//! # The Problem: Missing HKTs in Rust
//!
//! In languages like Haskell, you can write a trait (typeclass) that is generic over a type constructor `F<_>`:
//!
//! ```haskell
//! class Functor f where
//!   fmap :: (a -> b) -> f a -> f b
//! ```
//!
//! In Rust, generic parameters must be concrete types (of kind `*`), not type constructors (of kind `* -> *`).
//! You cannot write `trait Functor<F<A>>`.
//!
//! # The Solution: The GAT Pattern
//!
//! We define a "Witness" trait (`HKT`) that acts as a proxy for the type constructor. This witness trait
//! contains a Generic Associated Type (GAT) `Type<T>` which projects the witness back to the concrete type.
//!
//! ```rust,ignore
//! pub trait HKT {
//!     type Type<T>; // The GAT
//! }
//!
//! struct OptionWitness; // The Witness
//! impl HKT for OptionWitness {
//!     type Type<T> = Option<T>; // The Projection
//! }
//! ```
//!
//! This allows us to write generic traits like `Functor` over the witness `F`:
//!
//! ```rust,ignore
//! trait Functor<F: HKT> {
//!     fn fmap<A, B>(fa: F::Type<A>, f: Fn(A) -> B) -> F::Type<B>;
//! }
//! ```
//!
//! # Module Contents
//!
//! *   [`hkt`]: Defines the standard `HKT` trait (Arity 1) and fixed-parameter HKTs (`HKT2`, `HKT3`, etc.)
//!     where all but one parameter are fixed.
//! *   [`hkt_unbound`]: Defines "Unbound" HKT traits (`HKT2Unbound`, `HKT3Unbound`, etc.) where
//!     multiple parameters remain generic. This corresponds to multi-parameter type constructors
//!     like `Result<E, T>` (Bifunctor) or `(A, B, C)` (Trifunctor).
//!
//! # Arity Support
//!
//! This crate supports HKTs up to Arity 6 (for complex effect systems).
//!
//! *   **Arity 1**: `HKT` (e.g., `Option<T>`, `Vec<T>`)
//! *   **Arity 2**: `HKT2` (Fixed), `HKT2Unbound` (e.g., `Result<E, T>`)
//! *   **Arity 3**: `HKT3` (Fixed), `HKT3Unbound` (e.g., `(A, B, C)`)
//! *   **Arity 4**: `HKT4` (Fixed), `HKT4Unbound`
//! *   **Arity 5**: `HKT5` (Fixed), `HKT5Unbound`
//! *   **Arity 6**: `HKT6Unbound`

pub mod hkt;
pub mod hkt_unbound;
