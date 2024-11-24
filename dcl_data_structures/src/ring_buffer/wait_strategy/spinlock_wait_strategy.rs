// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use std::{
    borrow::Borrow,
};
use crate::ring_buffer::prelude::*;

pub struct SpinLoopWaitStrategy;

impl WaitStrategy for SpinLoopWaitStrategy {
    fn new() -> Self {
        SpinLoopWaitStrategy {}
    }

    fn wait_for<F: Fn() -> bool, S: Borrow<AtomicSequence>>(
        &self,
        sequence: Sequence,
        dependencies: &[S],
        check_alert: F,
    ) -> Option<Sequence> {
        loop {
            let available = min_cursor_sequence(dependencies);
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