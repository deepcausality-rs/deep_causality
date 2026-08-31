/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! HKT witnesses for the Cayley-Dickson number types.
//!
//! # What these witnesses do and do not implement
//!
//! `Functor` and `Foldable`, and nothing else.
//!
//! `Pure::pure` receives one value by value, with no `Clone` and no `Default` in its signature, so
//! it can fill exactly one slot. A variable-arity container answers that by producing a one-element
//! container, which is what `CausalTensor` and `CausalMultiVector` do. `Complex`, `Quaternion` and
//! `Octonion` are **fixed-arity products**: every field has to be filled, and a two-, four- or
//! eight-slot value cannot be built from a single move. `Applicative` declares
//! `Applicative<F>: Functor<F> + Pure<F>`, so it is out of reach for the same reason.
//!
//! `Monad` would need `Pure` as well, and a product functor has no canonical `bind` in any case.
//! `CoMonad::extract` could return the scalar part, but `extend` has no canonical cursor to walk
//! over a product, so it is left out rather than guessed at.
//!
//! # Why `NoConstraint`
//!
//! These operations move components; they never compute with them. `fmap` maps `Complex<A>` to
//! `Complex<B>` for unrelated `A` and `B`, so a complex of labels maps as readily as a complex of
//! `f64`. The arithmetic lives on impls that name `RealField` themselves, which the compiler
//! enforces and no witness can bypass.

pub mod hkt_complex;
pub mod hkt_octonion;
pub mod hkt_quaternion;

pub use hkt_complex::ComplexWitness;
pub use hkt_octonion::OctonionWitness;
pub use hkt_quaternion::QuaternionWitness;
