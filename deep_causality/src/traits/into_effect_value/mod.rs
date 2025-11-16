//! This module defines the `PropagatingEffect` trait.

/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{CausalityError, EffectValue, PropagatingValue};

/// Defines the contract for any type that can be losslessly converted to and from the
/// `EffectValue` enum.
///
/// This trait is the core mechanism for safe type conversion within the causal system,
/// enabling a compile-time, generic-based approach for causal functions and causaloids.
pub trait IntoEffectValue: PropagatingValue + Clone {
    /// Converts the implementing type into an `EffectValue` enum variant.
    ///
    /// This conversion should be lossless, meaning all information from the original type
    /// is preserved within the `EffectValue`.
    fn into_effect_value(self) -> EffectValue;

    /// Attempts to convert an `EffectValue` enum variant back into the implementing type.
    ///
    /// This conversion is fallible and returns a `Result` to indicate success or a
    /// `CausalityError` if the `EffectValue` does not contain the expected type.
    fn try_from_effect_value(ev: EffectValue) -> Result<Self, CausalityError>
    where
        Self: Sized;
}
