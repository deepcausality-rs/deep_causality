/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::pin::pin;
use std::task::{Context, Poll, Waker};

/// Polls a future to completion on a waker that does nothing.
///
/// Every backend in this crate returns futures that are ready when returned, so one poll
/// completes them; a future that returns `Pending` is polled again at once. A future that waits
/// on something outside itself never completes here, which is why this is a test utility and not a
/// runtime.
pub fn block_on<F: Future>(future: F) -> F::Output {
    let mut future = pin!(future);
    let mut cx = Context::from_waker(Waker::noop());
    loop {
        if let Poll::Ready(output) = future.as_mut().poll(&mut cx) {
            return output;
        }
    }
}
