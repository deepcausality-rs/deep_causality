/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Rng, RngError};

pub trait SampleRange<T> {
    fn sample_single<R: Rng + ?Sized>(self, rng: &mut R) -> Result<T, RngError>;

    fn is_empty(&self) -> bool;
}
