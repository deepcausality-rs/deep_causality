/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Functor, HKT, Monad};

// Witness for Option
pub struct OptionWitness;

impl HKT for OptionWitness {
    type Type<T> = Option<T>;
}

// Implementation of Functor for OptionWitness
impl Functor<OptionWitness> for OptionWitness {
    fn fmap<A, B, Func>(
        m_a: <OptionWitness as HKT>::Type<A>,
        f: Func,
    ) -> <OptionWitness as HKT>::Type<B>
    where
        Func: FnOnce(A) -> B,
    {
        m_a.map(f)
    }
}

// Implementation of Monad for OptionWitness
impl Monad<OptionWitness> for OptionWitness {
    fn pure<T>(value: T) -> <OptionWitness as HKT>::Type<T> {
        Some(value)
    }

    fn bind<A, B, Func>(
        m_a: <OptionWitness as HKT>::Type<A>,
        f: Func,
    ) -> <OptionWitness as HKT>::Type<B>
    where
        Func: FnOnce(A) -> <OptionWitness as HKT>::Type<B>,
    {
        m_a.and_then(f)
    }
}
