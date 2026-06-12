/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Feature-dependent thread-safety bound (the workspace `parallel` pattern).
//!
//! With the `parallel` feature off the trait is vacuous, so the serial API
//! carries no extra bounds. With the feature on it requires `Send + Sync`,
//! which is what the Rayon fan-out inside the N-dimensional plans needs.

/// Vacuous marker without the `parallel` feature.
#[cfg(not(feature = "parallel"))]
pub trait MaybeParallel {}

#[cfg(not(feature = "parallel"))]
impl<T> MaybeParallel for T {}

/// `Send + Sync` alias under the `parallel` feature.
#[cfg(feature = "parallel")]
pub trait MaybeParallel: Send + Sync {}

#[cfg(feature = "parallel")]
impl<T: Send + Sync> MaybeParallel for T {}
