/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The parameter-study combinator: an order-preserving, `Result`-collecting map over case
//! inputs, deterministically parallel under the `parallel` feature.
//!
//! A study is a sweep: back pressures, airspeeds, matrix rows, noise draws. The output type
//! is fully generic on purpose; a body may return a row array, a [`Report`](super::Report),
//! or a domain struct, and a study that runs no march at all (a pointwise table) uses the
//! same combinator. Side effects (printing, file writes) belong after the sweep, not inside
//! the body: under the `parallel` feature bodies run concurrently and their output would
//! interleave. Results are bit-identical to the sequential run either way.

use deep_causality_par::{MaybeParallel, scoped_map};

/// Map `f` over `items` in input order, collecting into one `Result`. The first error in
/// input order wins; under the `parallel` feature every body still runs (there is no
/// cancellation), but the returned error is the earliest failing case's.
pub fn sweep<T, U, E, F>(items: &[T], f: F) -> Result<Vec<U>, E>
where
    T: MaybeParallel,
    U: MaybeParallel,
    E: MaybeParallel,
    F: Fn(&T) -> Result<U, E> + MaybeParallel,
{
    scoped_map(items, f).into_iter().collect()
}
