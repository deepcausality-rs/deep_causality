/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Applicative, Foldable, Functor, HKT, Monad};

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

// Implementation of Applicative for VecWitness
impl Applicative<VecWitness> for VecWitness {
    fn pure<T>(value: T) -> <VecWitness as HKT>::Type<T> {
        vec![value]
    }

    fn apply<A, B, Func>(
        f_ab: <VecWitness as HKT>::Type<Func>,
        f_a: <VecWitness as HKT>::Type<A>,
    ) -> <VecWitness as HKT>::Type<B>
    where
        Func: FnMut(A) -> B,
        A: Clone,
    {
        f_ab.into_iter()
            .flat_map(|mut f_val| {
                f_a.iter()
                    .map(move |a_val| f_val(a_val.clone()))
                    .collect::<Vec<B>>()
            }) // Clone a_val for FnMut
            .collect()
    }
}

// Implementation of Foldable for VecWitness
impl Foldable<VecWitness> for VecWitness {
    fn fold<A, B, Func>(fa: <VecWitness as HKT>::Type<A>, init: B, f: Func) -> B
    where
        <VecWitness as HKT>::Type<A>: IntoIterator<Item = A>,
        Func: FnMut(B, A) -> B,
    {
        fa.into_iter().fold(init, f)
    }
}

// Implementation of Monad for VecWitness
impl Monad<VecWitness> for VecWitness {
    fn bind<A, B, Func>(m_a: <VecWitness as HKT>::Type<A>, f: Func) -> <VecWitness as HKT>::Type<B>
    where
        Func: FnMut(A) -> <VecWitness as HKT>::Type<B>,
    {
        m_a.into_iter().flat_map(f).collect()
    }
}
