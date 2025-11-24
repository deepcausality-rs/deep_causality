/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

// ----------------------------------------------------
// Unbound Higher Kinded Types (HKT) Traits for Arity 2 - 5
// ----------------------------------------------------

/// Trait for a Higher-Kinded Type (HKT) with two unbound generic parameters (Arity 2).
///
/// Unlike `HKT2`, which fixes one parameter and leaves one free (`F<T>`),
/// `HKT2Unbound` leaves *both* parameters free (`F<A, B>`).
///
/// # Category Theory
/// Corresponds to a **Bifunctor** or a type constructor $F: \mathcal{C} \times \mathcal{D} \to \mathcal{E}$.
/// It maps a pair of objects $(A, B)$ to a new object $F(A, B)$.
///
/// # Examples
/// * `Result<A, B>`
/// * `(A, B)` (Tuple)
/// * `Either<A, B>`
pub trait HKT2Unbound {
    /// The Generic Associated Type representing F<A, B>
    type Type<A, B>;
}

/// Trait for a Higher-Kinded Type (HKT) with three unbound generic parameters (Arity 3).
///
/// # Category Theory
/// Corresponds to a **Trifunctor** or a type constructor $F: \mathcal{C} \times \mathcal{D} \times \mathcal{E} \to \mathcal{S}$.
///
/// # Examples
/// * `(A, B, C)` (Triple)
/// * `State<S_in, S_out, A>` (Parametric State)
pub trait HKT3Unbound {
    type Type<A, B, C>;
}

/// Trait for a Higher-Kinded Type (HKT) with four unbound generic parameters (Arity 4).
///
/// # Category Theory
/// Corresponds to a **Quadrifunctor** or a generic tensor of rank 4.
///
/// # Examples
/// * `(A, B, C, D)` (Quadruple)
/// * `RiemannTensor<A, B, C, D>`
pub trait HKT4Unbound {
    type Type<A, B, C, D>;
}

/// Trait for a Higher-Kinded Type (HKT) with five unbound generic parameters (Arity 5).
///
/// # Category Theory
/// Corresponds to a **Pentafunctor**.
///
/// # Examples
/// * `(A, B, C, D, E)` (Quintuple)
/// * `CyberneticLoop<S, B, C, A, E>`
pub trait HKT5Unbound {
    type Type<A, B, C, D, E>;
}

/// Trait for a Higher-Kinded Type (HKT) with six unbound generic parameters (Arity 6).
///
/// # Category Theory
/// Corresponds to a **Hexafunctor**.
///
/// # Examples
/// * `(A, B, C, D, E, F)` (Sextuple)
/// * `Effect5Unbound<Fixed1, Fixed2, Fixed3, S_in, S_out, A>`
pub trait HKT6Unbound {
    type Type<A, B, C, D, E, F>;
}
