/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Rng;

pub trait Fill {
    fn fill<R: Rng + ?Sized>(&mut self, rng: &mut R);
}
