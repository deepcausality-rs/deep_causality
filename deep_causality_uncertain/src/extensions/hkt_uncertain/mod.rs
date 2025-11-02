/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Uncertain;
use crate::{ProbabilisticType, UncertainApplicative, UncertainFunctor, UncertainMonad};
use deep_causality_haft::HKT;

pub struct UncertainWitness {}

impl HKT for UncertainWitness {
    type Type<T> = Uncertain<T>;
}

impl UncertainApplicative<UncertainWitness> for UncertainWitness {
    fn pure<T>(value: T) -> <UncertainWitness as HKT>::Type<T>
    where
        T: ProbabilisticType,
    {
        unimplemented!()
    }

    fn apply<A, B, Func>(
        f_ab: <UncertainWitness as HKT>::Type<Func>,
        f_a: <UncertainWitness as HKT>::Type<A>,
    ) -> <UncertainWitness as HKT>::Type<B>
    where
        Func: FnMut(A) -> B,
        A: ProbabilisticType,
        B: ProbabilisticType,
    {
        unimplemented!()
    }
}

impl UncertainFunctor<UncertainWitness> for UncertainWitness {
    fn fmap<A, B, Func>(
        m_a: <UncertainWitness as HKT>::Type<A>,
        f: Func,
    ) -> <UncertainWitness as HKT>::Type<B>
    where
        Func: FnMut(A) -> B,
        A: ProbabilisticType,
        B: ProbabilisticType,
    {
        unimplemented!()
    }
}

impl UncertainMonad<UncertainWitness> for UncertainWitness {
    fn bind<A, B, Func>(
        m_a: <UncertainWitness as HKT>::Type<A>,
        f: Func,
    ) -> <UncertainWitness as HKT>::Type<B>
    where
        Func: FnMut(A) -> <UncertainWitness as HKT>::Type<B>,
        A: ProbabilisticType,
        B: ProbabilisticType,
    {
        unimplemented!()
    }
}
