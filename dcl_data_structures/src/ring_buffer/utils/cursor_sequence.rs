// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use crate::ring_buffer::sequence::sequence::{AtomicSequence, Sequence};
use std::borrow::Borrow;

pub fn min_cursor_sequence<S: Borrow<AtomicSequence>>(sequences: &[S]) -> Sequence {
    sequences
        .iter()
        .map(|s| s.borrow().get())
        .min()
        .unwrap_or_default()
}
