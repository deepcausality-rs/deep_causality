// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

pub mod atomic_sequence_ordered;
pub mod atomic_sequence_relaxed;

/// Type alias for sequence numbers in the ring buffer.
/// Uses u64 to provide a large range of sequence numbers before wrapping.
pub type Sequence = u64;
