// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use crate::ring_buffer::prelude::*;
use std::borrow::Borrow;
use std::sync::{Condvar, Mutex};

pub struct BlockingWaitStrategy {
    guard: Mutex<()>,
    cvar: Condvar,
}

impl WaitStrategy for BlockingWaitStrategy {
    fn new() -> Self {
        Self {
            cvar: Condvar::new(),
            guard: Mutex::new(()),
        }
    }

    fn wait_for<F: Fn() -> bool, S: Borrow<AtomicSequenceOrdered>>(
        &self,
        sequence: Sequence,
        dependencies: &[S],
        check_alert: F,
    ) -> Option<Sequence> {
        loop {
            let blocked = self.guard.lock().unwrap();
            if check_alert() {
                return None;
            }

            let available = get_min_cursor_sequence(dependencies);
            if available >= sequence {
                return Some(available);
            } else {
                let _guard = self.cvar.wait(blocked).unwrap();
            }
        }
    }

    fn signal(&self) {
        let _guard = self.guard.lock().unwrap();
        self.cvar.notify_all();
        drop(_guard);
    }
}
