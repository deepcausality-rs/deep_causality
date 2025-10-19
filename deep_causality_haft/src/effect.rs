/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

// ----------------------------------------------------
// Effect Traits (Arity 3)
// ----------------------------------------------------

use crate::{HKT, HKT3, HKT4, HKT5};

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

// ----------------------------------------------------
// Effect Traits (Arity 4)
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

// ----------------------------------------------------
// Effect Traits (Arity 5)
// ----------------------------------------------------

/// Effect5: The Bridge Trait for Arity 5 Type Constructors.
pub trait Effect5 {
    type Fixed1;
    type Fixed2;
    type Fixed3;
    type Fixed4;

    /// The concrete witness type that implements HKT5 with the four fixed types.
    type HktWitness: HKT5<Self::Fixed1, Self::Fixed2, Self::Fixed3, Self::Fixed4> + HKT;
}
