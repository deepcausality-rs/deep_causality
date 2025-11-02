/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::ProbabilisticType;
use deep_causality_haft::HKT;

pub trait UncertainMonad<F: HKT> {
    fn bind<A, B, Func>(m_a: F::Type<A>, f: Func) -> F::Type<B>
    where
        Func: FnMut(A) -> F::Type<B>,
        A: ProbabilisticType,
        B: ProbabilisticType;
}
