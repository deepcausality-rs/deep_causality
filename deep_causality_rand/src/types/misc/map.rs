/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

pub struct Map<D, F, T, S> {
    pub distr: D,
    pub func: F,
    pub phantom: core::marker::PhantomData<(T, S)>,
}
