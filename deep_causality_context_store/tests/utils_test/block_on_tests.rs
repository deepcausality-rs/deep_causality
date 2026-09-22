/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Expected values are the literals the futures resolve to.
//!
//! Corner cases (rows A to K): B a future ready on the first poll in
//! `test_a_ready_future_completes_in_one_poll`; D a future pending exactly once, the boundary
//! between one poll and two, in `test_a_pending_future_is_polled_again`, and one pending five
//! times in `test_a_future_pending_several_times_completes`, which pins that the loop is a loop
//! and not a fixed number of polls; every other row n/a.
use core::future::{Future, ready};
use core::pin::Pin;
use core::task::{Context, Poll};
use deep_causality_context_store::utils_test::block_on;

/// Pending on the first poll, ready on the second.
struct Twice {
    polled: bool,
}

impl Future for Twice {
    type Output = u8;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<u8> {
        if self.polled {
            Poll::Ready(3)
        } else {
            self.polled = true;
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

/// Pending `left` times, then ready with the number of polls it took.
struct Countdown {
    left: u8,
    polls: u8,
}

impl Future for Countdown {
    type Output = u8;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<u8> {
        self.polls += 1;
        if self.left == 0 {
            Poll::Ready(self.polls)
        } else {
            self.left -= 1;
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

#[test]
fn test_a_ready_future_completes_in_one_poll() {
    assert_eq!(block_on(ready(7)), 7);
}

#[test]
fn test_a_pending_future_is_polled_again() {
    assert_eq!(block_on(Twice { polled: false }), 3);
}

#[test]
fn test_a_future_pending_several_times_completes() {
    // Five pendings then ready: six polls in all, counted by the future itself.
    assert_eq!(block_on(Countdown { left: 5, polls: 0 }), 6);
}
