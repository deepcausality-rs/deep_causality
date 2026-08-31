/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! HKT witnesses for the Cayley-Dickson number types.
//!
//! # What these witnesses implement, and what they still do not
//!
//! `Functor`, `Foldable`, and the lax monoidal stack: `Semigroupal`, `LaxMonoidal`,
//! `Convolutional` and `MonoidalApplicative`. Not `Pure`, not `Applicative`, not `Monad`, not
//! `CoMonad`.
//!
//! # Why `φ` is forced here, and `pure` is impossible
//!
//! Each of these types is `F(A) = A^S` for a finite index set `S`: two slots for `Complex`, four
//! for `Quaternion`, eight for `Octonion`. That single fact settles both halves.
//!
//! **The structure map is unique.** `F(())` is a singleton, so `unit` has exactly one inhabitant
//! and is forced. For `φ`, both `A^S × B^S` and `(A × B)^S` are representable at `(S, S)` in
//! `Set × Set`, so by Yoneda every natural `φ` is `φ(fa, fb)_s = (fa_{u(s)}, fb_{v(s)})` for a
//! fixed pair of endofunctions `u, v` of `S`. Right unitality forces `u = id` and left unitality
//! forces `v = id`. Componentwise pairing is therefore the *only* lawful `φ`, and associativity
//! follows. There is no choice for an implementer to get wrong, which is why these witnesses were
//! the first to adopt the stack.
//!
//! **`pure` remains impossible, and for the reason the stack exists.** The applicative this yields
//! is Reader on a finite index set, whose `pure` is the constant map `s ↦ a`. Filling `n` slots
//! from one value is the diagonal `Δ : A → A ⊗ A`, which in Rust is `Clone`. `Pure::pure` receives
//! one value by value and the trait declares no bound on it, so the body has no way to duplicate
//! it; an impl cannot add `Clone` of its own either, because that is stricter than the trait
//! (E0276). So a two-, four- or eight-slot value cannot be built from a single move:
//!
//! ```text
//! error[E0382]: use of moved value: `value`
//! ```
//!
//! `Applicative<F>: Functor<F> + Pure<F>` and `Monad` names `Pure` too, so both stay out of reach.
//! `MonoidalApplicative` reaches `apply` without them precisely because `zip_with` pairs slot with
//! slot and never invokes the diagonal. See `openspec/notes/archive/hkt_gat/monoidal-applicative.md`.
//!
//! `CoMonad::extract` could return the scalar part, but `extend` has no canonical cursor to walk
//! over a product, so it is left out rather than guessed at.
//!
//! # Why the element type carries no bound
//!
//! These operations move components; they never compute with them. `fmap` maps `Complex<A>` to
//! `Complex<B>` for unrelated `A` and `B`, so a complex of labels maps as readily as a complex of
//! `f64`, and `zip_with` needs no arithmetic either. The arithmetic lives on impls that name
//! `RealField` themselves, which the compiler enforces and no witness can bypass.
//!
pub mod hkt_complex;
pub mod hkt_octonion;
pub mod hkt_quaternion;

pub use hkt_complex::ComplexWitness;
pub use hkt_octonion::OctonionWitness;
pub use hkt_quaternion::QuaternionWitness;
