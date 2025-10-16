/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

/// A zero-sized type used purely as a marker/placeholder when implementing
/// the HKT traits for concrete types.
///
/// Example: Instead of implementing HKT for `Option`, you implement it for
/// `Option<Placeholder>`.
pub struct Placeholder;

// ----------------------------------------------------
// HKT Trait for Arity 1: Kind * -> *
// ----------------------------------------------------

/// Trait for a Higher-Kinded Type (HKT) with one type parameter.
///
/// This is implemented by a concrete "witness" type which serves as a token
/// to refer to the type constructor (e.g., Option<Placeholder> witnesses Option).
pub trait HKT {
    /// The Generic Associated Type (GAT) that represents the type constructor.
    /// The `<T>` is the "hole" in the type constructor (e.g., Option<T>).
    type Type<T>;
}

// ----------------------------------------------------
// HKT Trait for Arity 2: Kind *, * -> *
// ----------------------------------------------------

/// Trait for an HKT with two type parameters.
///
/// It is generic over the first type parameter to be "fixed" (`F`).
/// This allows the trait to represent a type that is partially applied.
///
/// Example: Result<T, E> is Kind *, * -> *. To make it HKT2, we fix E (F).
pub trait HKT2<F> {
    /// The GAT that represents the remaining type constructor.
    /// The resulting kind is * -> * (one hole <T> remaining).
    ///
    /// Example: If the implementing type is Result<(), F> and F is i32,
    /// then Type<T> is Result<T, i32>.
    type Type<T>;
}

// ----------------------------------------------------
// HKT Trait for Arity 3: Kind *, *, * -> *
// ----------------------------------------------------

/// Trait for an HKT with three type parameters.
///
/// It is generic over the first two type parameters to be "fixed" (`F1` and `F2`).
/// This is essential for your DSL's Error and Warning/Log types.
///
/// Example: Your DiscoveryResult<T, E, W> is Kind *, *, * -> *.
pub trait HKT3<F1, F2> {
    /// The GAT that represents the remaining type constructor.
    /// The resulting kind is * -> * (one hole <T> remaining).
    ///
    /// Example: If the implementing type is DiscoveryResult<(), F1, F2>,
    /// then Type<T> is DiscoveryResult<T, F1, F2>.
    type Type<T>;
}

// ----------------------------------------------------
// Manual HKT Implementations
// ----------------------------------------------------

// Witness for Option
pub struct OptionWitness;

impl HKT for OptionWitness {
    type Type<T> = Option<T>;
}

// Witness for Result<T, E> where E is fixed
pub struct ResultWitness<E>(Placeholder, E);

impl<E> HKT2<E> for ResultWitness<E> {
    type Type<T> = Result<T, E>;
}

impl<E> HKT for ResultWitness<E> {
    type Type<T> = Result<T, E>;
}
