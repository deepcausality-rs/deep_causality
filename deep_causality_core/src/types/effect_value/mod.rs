/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! This module defines the CausalValue enum.

use crate::PropagatingEffect;
use crate::alias::{ContextoidId, IdentificationValue};
use crate::errors::CausalityError;
use crate::traits::propagating_value::PropagatingValue;
use crate::EffectLog;

#[cfg(feature = "std")]
use std::collections::HashMap;

use alloc::boxed::Box;
use core::fmt::Debug;

mod display;
mod from;
mod partial_eq;
mod predicates;

/// Represents the payload of a propagating effect.
///
/// This enum encapsulates various types of effect data that can be propagated
/// through the causal effect system. It is generic over type `T` to allow
/// flexibility in the value type.
#[derive(Debug, Clone, Default)]
pub enum EffectValue<T> {
    /// Represents the absence of a signal or evidence.
    #[default]
    None,
    /// Represents a value of type T
    Value(T),
    /// A link to a complex, structured result in a Contextoid. As an output, this
    /// can be interpreted by a reasoning engine as a command to fetch data.
    ContextualLink(ContextoidId, ContextoidId),
    /// A dispatch command that directs the reasoning engine to dynamically jump to a specific
    /// causaloid within the graph. The `usize` is the target causaloid's index, and the `Box<PropagatingEffect>`
    /// is the effect to be passed as input to that target causaloid. This enables adaptive reasoning.
    RelayTo(
        usize,
        Box<PropagatingEffect<T, CausalityError, EffectLog>>,
    ),
    /// A collection of named values, allowing for complex, structured data passing.
    #[cfg(feature = "std")]
    Map(HashMap<IdentificationValue, Box<PropagatingEffect<T, CausalityError, EffectLog>>>),
    /// A container for any external, user-defined type that implements the `PropagatingValue` trait.
    /// This enables the causal system to be extended with custom data types.
    External(Box<dyn PropagatingValue>),
}
