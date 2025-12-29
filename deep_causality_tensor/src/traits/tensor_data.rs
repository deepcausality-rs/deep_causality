/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! TensorData marker trait for allowed tensor element types.

use deep_causality_num::Field;

/// Marker trait for allowed tensor data types.
///
/// This trait constrains the types that can be used as tensor elements.
/// It requires:
/// - `Field` (from `deep_causality_num`) for full arithmetic operations
/// - `Default` for tensor initialization (zeros, etc.)
/// - `PartialOrd` for comparison operations (max, min, etc.)
/// - `Copy + Send + Sync` for efficient memory handling and thread safety
///
/// # Blanket Implementation
///
/// Any type satisfying the bounds automatically implements `TensorData`.
pub trait TensorData:
    Field + Copy + Default + PartialOrd + Send + Sync + From<u32> + 'static
{
}

// Blanket implementation for all qualifying types
impl<T> TensorData for T where
    T: Field + Copy + Default + PartialOrd + Send + Sync + From<u32> + 'static
{
}
