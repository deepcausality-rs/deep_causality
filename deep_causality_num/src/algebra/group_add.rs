/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::Zero;
use core::ops::{Add, Sub};

pub trait AddGroup: Add<Output = Self> + Sub<Output = Self> + Zero + Clone {}

// Blanket Implementation for all types that impl Add, Sub, and have zero
impl<T> AddGroup for T where T: Add<Output = T> + Sub<Output = T> + Zero + Clone {}
