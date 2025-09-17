/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

#[derive(Debug, Clone)]
pub struct Iter<D, R, T> {
    pub distr: D,
    pub rng: R,
    pub phantom: core::marker::PhantomData<T>,
}
