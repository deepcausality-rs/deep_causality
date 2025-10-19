/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Functor, Monad, HKT, HKT2, OptionWitness, ResultWitness};

// Manual implementation of Functor for OptionWitness
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

// Manual implementation of Monad for OptionWitness
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

// Manual implementation of Functor for ResultWitness
impl<E> Functor<ResultWitness<E>> for ResultWitness<E>
where
    E: 'static,
{
    fn fmap<A, B, Func>(
        m_a: <ResultWitness<E> as HKT2<E>>::Type<A>,
        f: Func,
    ) -> <ResultWitness<E> as HKT2<E>>::Type<B>
    where
        Func: FnOnce(A) -> B,
    {
        m_a.map(f)
    }
}

// Manual implementation of Monad for ResultWitness
impl<E> Monad<ResultWitness<E>> for ResultWitness<E>
where
    E: 'static,
{
    fn pure<T>(value: T) -> <ResultWitness<E> as HKT2<E>>::Type<T> {
        Ok(value)
    }

    fn bind<A, B, Func>(
        m_a: <ResultWitness<E> as HKT2<E>>::Type<A>,
        f: Func,
    ) -> <ResultWitness<E> as HKT2<E>>::Type<B>
    where
        Func: FnOnce(A) -> <ResultWitness<E> as HKT2<E>>::Type<B>,
    {
        m_a.and_then(f)
    }
}
