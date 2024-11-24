// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

mod spinlock_wait_strategy;
mod blocking_wait_strategy;

// Re exports
pub use spinlock_wait_strategy::*;
pub use blocking_wait_strategy::*;