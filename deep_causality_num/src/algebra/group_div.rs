/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

// Often redundant if MulGroup exists, but sometimes used to enforce
// Euclidean Division or Remainder logic for Integers.
use crate::MulGroup;

pub trait DivGroup: MulGroup {}

// Blanket Implementation
impl<T: MulGroup> DivGroup for T {}
