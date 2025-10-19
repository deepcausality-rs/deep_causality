/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{Functor, HKT, HKT2, Monad, Placeholder};

// Witness for Result<T, E> where E is fixed
pub struct ResultWitness<E>(Placeholder, E);

impl<E> HKT2<E> for ResultWitness<E> {
    type Type<T> = Result<T, E>;
}

impl<E> HKT for ResultWitness<E> {
    type Type<T> = Result<T, E>;
}

// Implementation of Functor for ResultWitness
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

// Implementation of Monad for ResultWitness
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
