/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use super::kind::{HKT, HKT3, HKT4};

// ----------------------------------------------------
// Standard Functional Traits (Arity 1)
// ----------------------------------------------------

/// Functor: Abstracts over the ability to map a function over a type constructor.
///
/// Generic over the HKT witness `F`.
pub trait Functor<F: HKT> {
    /// Applies a function `f` to the value inside the container `m_a`.
    fn fmap<A, B, Func>(m_a: F::Type<A>, f: Func) -> F::Type<B>
    where
        Func: FnOnce(A) -> B;
}

// ----------------------------------------------------
// Monad Trait (Arity 2)
// ----------------------------------------------------

/// Monad: Abstracts over the ability to sequence computations within a type constructor.
///
/// Generic over the HKT witness `F`.
pub trait Monad<F: HKT>: Functor<F> {
    /// Lifts a value into the minimal monadic context. (e.g., Some(value), Ok(value)).
    fn pure<T>(value: T) -> F::Type<T>;

    /// Chains a computation from an effectful value, flattening the result.
    /// This is the core sequencing operation.
    fn bind<A, B, Func>(m_a: F::Type<A>, f: Func) -> F::Type<B>
    where
        // The function must return a new effectful type (F::Type<B>)
        Func: FnOnce(A) -> F::Type<B>;
}

// ----------------------------------------------------
// Advanced Functional Traits (Arity 3)
// ----------------------------------------------------

/// Effect3: The Bridge Trait for Arity 3 Type Constructors.
///
/// This trait is implemented by a user-defined **System Witness** (e.g., `MyEffect`)
/// to partially apply (fix) two of the three generic parameters of the HKT3 type.
pub trait Effect3 {
    /// The fixed type for the first parameter (e.g., the Error type E).
    type Fixed1;

    /// The fixed type for the second parameter (e.g., the Warning/Log type W).
    type Fixed2;

    /// The concrete witness type that implements HKT3 with the two fixed types.
    /// It MUST implement HKT so we can pass it to Functor/Monad functions.
    type HktWitness: HKT3<Self::Fixed1, Self::Fixed2> + HKT;
}

/// Monadic logic for the Arity 3 type after it has been partially applied.
///
/// Generic over the Effect3 witness `E`. This is your type-encoded effect system.
pub trait MonadEffect3<E: Effect3>
where
    E::HktWitness: Functor<E::HktWitness> + Sized,
{
    /// Lifts a pure value into the DSL's Effect container.
    fn pure<T>(value: T) -> <E::HktWitness as HKT3<E::Fixed1, E::Fixed2>>::Type<T>;

    /// The core sequencing operation
    fn bind<T, U, Func>(
        effect: <E::HktWitness as HKT3<E::Fixed1, E::Fixed2>>::Type<T>,
        f: Func,
    ) -> <E::HktWitness as HKT3<E::Fixed1, E::Fixed2>>::Type<U>
    where
        Func: FnOnce(T) -> <E::HktWitness as HKT3<E::Fixed1, E::Fixed2>>::Type<U>;
}

// ----------------------------------------------------
// Advanced Functional Traits (Arity 4)
// ----------------------------------------------------

/// Effect4: The Bridge Trait for Arity 4 Type Constructors.I
pub trait Effect4 {
    /// The fixed type for the first parameter.
    type Fixed1;

    /// The fixed type for the second parameter.
    type Fixed2;

    /// The fixed type for the third parameter.
    type Fixed3;

    /// The concrete witness type that implements HKT4 with the three fixed types.
    /// It MUST implement HKT so we can pass it to Functor/Monad functions.
    type HktWitness: HKT4<Self::Fixed1, Self::Fixed2, Self::Fixed3> + HKT;
}

/// Monadic logic for the Arity 4 type after it has been partially applied.
///
/// Generic over the Effect4 witness `E`.
pub trait MonadEffect4<E: Effect4>
where
    E::HktWitness: Functor<E::HktWitness> + Sized,
{
    /// Lifts a pure value into the DSL's Effect container.
    fn pure<T>(value: T) -> <E::HktWitness as HKT4<E::Fixed1, E::Fixed2, E::Fixed3>>::Type<T>;

    /// The core sequencing operation
    fn bind<T, U, Func>(
        effect: <E::HktWitness as HKT4<E::Fixed1, E::Fixed2, E::Fixed3>>::Type<T>,
        f: Func,
    ) -> <E::HktWitness as HKT4<E::Fixed1, E::Fixed2, E::Fixed3>>::Type<U>
    where
        Func: FnOnce(T) -> <E::HktWitness as HKT4<E::Fixed1, E::Fixed2, E::Fixed3>>::Type<U>;
}
