/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Effect3, Effect4, Effect5, Functor, HKT3, HKT4, HKT5};

// ----------------------------------------------------
// Monad Effect Traits (Arity 3)
// ----------------------------------------------------

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
        Func: FnMut(T) -> <E::HktWitness as HKT3<E::Fixed1, E::Fixed2>>::Type<U>;
}

// ----------------------------------------------------
// Monad Effect Traits (Arity 4)
// ----------------------------------------------------

/// Monadic logic for the Arity 4 type after it has been partially applied.
///
/// Generic over the Effect4 witness `E`.
pub trait MonadEffect4<E: Effect4>
where
    E::HktWitness: Functor<E::HktWitness> + Sized,
{
    /// Lifts a pure value into an Effect container.
    fn pure<T>(value: T) -> <E::HktWitness as HKT4<E::Fixed1, E::Fixed2, E::Fixed3>>::Type<T>;

    /// The core sequencing operation
    fn bind<T, U, Func>(
        effect: <E::HktWitness as HKT4<E::Fixed1, E::Fixed2, E::Fixed3>>::Type<T>,
        f: Func,
    ) -> <E::HktWitness as HKT4<E::Fixed1, E::Fixed2, E::Fixed3>>::Type<U>
    where
        Func: FnMut(T) -> <E::HktWitness as HKT4<E::Fixed1, E::Fixed2, E::Fixed3>>::Type<U>;
}

// ----------------------------------------------------
// Monad Effect Traits (Arity 5)
// ----------------------------------------------------

/// Monadic logic for the Arity 5 type after it has been partially applied.
///
/// Generic over the Effect5 witness `E`.
///
#[allow(clippy::type_complexity)]
pub trait MonadEffect5<E: Effect5>
where
    E::HktWitness: Functor<E::HktWitness> + Sized,
{
    /// Lifts a pure value into an Effect container.
    fn pure<T>(
        value: T,
    ) -> <E::HktWitness as HKT5<E::Fixed1, E::Fixed2, E::Fixed3, E::Fixed4>>::Type<T>;

    /// The core sequencing operation
    fn bind<T, U, Func>(
        effect: <E::HktWitness as HKT5<E::Fixed1, E::Fixed2, E::Fixed3, E::Fixed4>>::Type<T>,
        f: Func,
    ) -> <E::HktWitness as HKT5<E::Fixed1, E::Fixed2, E::Fixed3, E::Fixed4>>::Type<U>
    where
        Func: FnMut(
            T,
        )
            -> <E::HktWitness as HKT5<E::Fixed1, E::Fixed2, E::Fixed3, E::Fixed4>>::Type<U>;
}
