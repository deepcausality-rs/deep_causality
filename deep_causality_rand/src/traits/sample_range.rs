/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Rng;

pub trait SampleRange<T> {
    fn sample_single<R: Rng + ?Sized>(
        self,
        rng: &mut R,
    ) -> Result<T, crate::errors::rng_error::RngError>;

    fn is_empty(&self) -> bool;
}
