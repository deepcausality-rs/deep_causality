/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

// This is primarily a marker/umbrella module if you want a generic 'Group' trait
// that abstracts over the operation, but usually in Rust we stick to
// AddGroup/MulGroup for clarity.
//
// However, for completeness of the hierarchy:

use crate::AddGroup;

pub trait Group: AddGroup {}

// Blanket Implementation for all types that impl Add
impl<T: AddGroup> Group for T {}
