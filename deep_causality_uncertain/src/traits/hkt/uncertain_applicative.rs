/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::ProbabilisticType;
use deep_causality_haft::HKT;

pub trait UncertainApplicative<F: HKT> {
    fn pure<T>(value: T) -> F::Type<T>
    where
        T: ProbabilisticType;

    fn apply<A, B, Func>(f_ab: F::Type<Func>, f_a: F::Type<A>) -> F::Type<B>
    where
        Func: FnMut(A) -> B,
        A: ProbabilisticType,
        B: ProbabilisticType;
}
