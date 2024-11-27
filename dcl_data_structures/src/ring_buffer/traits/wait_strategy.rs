// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use crate::ring_buffer::prelude::{AtomicSequenceOrdered, Sequence};
use std::borrow::Borrow;

/// A trait defining a wait strategy.
///
/// A wait strategy determines how a sequencer blocks and unblocks.
/// Implementations of this trait can be used to define different
/// waiting strategies.
///
/// The `wait_for` method is called when a sequencer needs to wait.
/// The caller should implement a loop that calls `wait_for` until
/// the method returns `Some(_)` or the alert status of the sequencer
/// changes.
///
/// The `signal` method is called when the sequencer is alerted.
///
/// The `new` method is used to create a new instance of the wait
/// strategy.
pub trait WaitStrategy: Send + Sync {
    fn new() -> Self;
    fn wait_for<F: Fn() -> bool, S: Borrow<AtomicSequenceOrdered>>(
        &self,
        sequence: Sequence,
        dependencies: &[S],
        check_alert: F,
    ) -> Option<Sequence>;
    fn signal(&self);
}
