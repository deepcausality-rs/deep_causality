// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use crate::ring_buffer::prelude::*;
use std::borrow::Borrow;

pub struct SpinLoopWaitStrategy;

impl WaitStrategy for SpinLoopWaitStrategy {
    fn new() -> Self {
        SpinLoopWaitStrategy {}
    }

    fn wait_for<F: Fn() -> bool, S: Borrow<AtomicSequenceOrdered>>(
        &self,
        sequence: Sequence,
        dependencies: &[S],
        check_alert: F,
    ) -> Option<Sequence> {
        loop {
            let available = get_min_cursor_sequence(dependencies);
            if available >= sequence {
                return Some(available);
            }
            if check_alert() {
                return None;
            }
        }
    }

    fn signal(&self) {}
}
