/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! This module defines the CausalValue enum.

use crate::{ContextoidId, IdentificationValue, NumericValue, PropagatingEffect, PropagatingValue};
use deep_causality_num::{Complex, Quaternion};
use deep_causality_tensor::CausalTensor;
use deep_causality_uncertain::{
    MaybeUncertainBool, MaybeUncertainF64, UncertainBool, UncertainF64,
};
use std::collections::HashMap;
use std::fmt::Debug;

mod display;
mod extractors;
mod from;
mod partial_eq;
mod predicates;

/// Represents the payload of a propagating effect.
///
/// This enum encapsulates various types of effect data that can be propagated
/// through the causal effect system. It supports primitive types, strings,
/// and vectors of these types.
#[derive(Debug, Clone, Default)]
pub enum EffectValue {
    /// Represents the absence of a signal or evidence. Serves as the default.
    #[default]
    None,
    /// Represents a simple boolean value. This effect propagates like any other,
    /// and its interpretation (e.g., whether it prunes a traversal) is left to the
    /// consuming logic or explicit error handling within Causaloids.
    Deterministic(bool),
    /// Represents a standard numeric value i.e. int, uint, float.
    Number(NumericValue),
    /// Represents a numerical measurement
    Numerical(f64),
    /// Represents a quantitative outcome, such as a probability score or confidence level.
    Probabilistic(f64),
    /// Represents a Tensor via Causal Tensor.
    /// Note, when you import the  CausalTensorWitness from the deep_causality_tensor crate,
    /// you can apply monadic composition and monadic transformation to tensors.
    Tensor(CausalTensor<f64>),
    /// Represents a Complex Number  
    Complex(Complex<f64>),
    /// Represents a Tensor over complex numbers via Causal Tensor.
    /// Note, when you import the  CausalTensorWitness from the deep_causality_tensor crate,
    /// you can apply monadic composition and monadic transformation to complex tensors.
    ComplexTensor(CausalTensor<Complex<f64>>),
    /// Represents a Quaternion (4D Complex number)
    Quaternion(Quaternion<f64>),
    /// Represents a Tensor over quaternion numbers via Causal Tensor.
    QuaternionTensor(CausalTensor<Quaternion<f64>>),
    /// Represents a value with inherent uncertainty, modeled as a probability distribution.
    UncertainBool(UncertainBool),
    UncertainFloat(UncertainF64),
    /// Represents a value that is probabilistic present or absent with uncertainty when present
    MaybeUncertainBool(MaybeUncertainBool),
    MaybeUncertainFloat(MaybeUncertainF64),
    /// A link to a complex, structured result in a Contextoid. As an output, this
    /// can be interpreted by a reasoning engine as a command to fetch data.
    ContextualLink(ContextoidId),
    /// A dispatch command that directs the reasoning engine to dynamically jump to a specific
    /// causaloid within the graph. The `usize` is the target causaloid's index, and the `Box<CausalValue>`
    /// is the effect to be passed as input to that target causaloid. This enables adaptive reasoning.
    RelayTo(usize, Box<PropagatingEffect>),
    /// A container for any external, user-defined type that implements the `PropagatingValue` trait.
    /// This enables the causal system to be extended with custom data types.
    External(Box<dyn PropagatingValue>),

    /// A collection of named values, allowing for complex, structured data passing.
    Map(HashMap<IdentificationValue, Box<PropagatingEffect>>),
}
