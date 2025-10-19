/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Functor, HKT, Monad};

// Witness for Vec<T>
pub struct VecWitness;

impl HKT for VecWitness {
    type Type<T> = Vec<T>;
}

// Implementation of Functor for VecWitness
impl Functor<VecWitness> for VecWitness {
    fn fmap<A, B, Func>(m_a: <VecWitness as HKT>::Type<A>, f: Func) -> <VecWitness as HKT>::Type<B>
    where
        Func: FnMut(A) -> B,
    {
        m_a.into_iter().map(f).collect()
    }
}

// Implementation of Monad for VecWitness
impl Monad<VecWitness> for VecWitness {
    fn pure<T>(value: T) -> <VecWitness as HKT>::Type<T> {
        vec![value]
    }

    fn bind<A, B, Func>(m_a: <VecWitness as HKT>::Type<A>, f: Func) -> <VecWitness as HKT>::Type<B>
    where
        Func: FnMut(A) -> <VecWitness as HKT>::Type<B>,
    {
        m_a.into_iter().flat_map(f).collect()
    }
}
