/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Applicative, Foldable, Functor, HKT, Monad};

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

// Implementation of Applicative for OptionWitness
impl Applicative<OptionWitness> for OptionWitness {
    fn pure<T>(value: T) -> <OptionWitness as HKT>::Type<T> {
        Some(value)
    }

    fn apply<A, B, Func>(
        f_ab: <OptionWitness as HKT>::Type<Func>,
        f_a: <OptionWitness as HKT>::Type<A>,
    ) -> <OptionWitness as HKT>::Type<B>
    where
        Func: FnMut(A) -> B,
    {
        f_ab.and_then(|f| f_a.map(f))
    }
}

// Implementation of Foldable for OptionWitness
impl Foldable<OptionWitness> for OptionWitness {
    fn fold<A, B, Func>(fa: Option<A>, init: B, mut f: Func) -> B
    where
        Func: FnMut(B, A) -> B,
    {
        match fa {
            Some(a) => f(init, a),
            None => init,
        }
    }
}

// Implementation of Monad for OptionWitness
impl Monad<OptionWitness> for OptionWitness {
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
