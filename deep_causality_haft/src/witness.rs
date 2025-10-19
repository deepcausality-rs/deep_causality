/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */


// ----------------------------------------------------
// Manual HKT Implementations
// ----------------------------------------------------

use crate::{Placeholder, HKT, HKT2};

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
