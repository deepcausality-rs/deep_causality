/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{Diff, DifferentiableArrow, Scalar};
use deep_causality_haft::Arrow;
use deep_causality_num::Dual;

impl<A, R> Arrow for Diff<A, R>
where
    A: DifferentiableArrow,
    R: Scalar,
{
    type In = Dual<R>;
    type Out = Dual<R>;

    #[inline]
    fn run(&self, input: Dual<R>) -> Dual<R> {
        self.0.run(input)
    }
}
