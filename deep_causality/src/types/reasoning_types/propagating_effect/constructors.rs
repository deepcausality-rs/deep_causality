/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{
    CausalMonad, CausalityError, ComplexTensor, ContextId, ContextoidId, EffectValue,
    IdentificationValue, NumericValue, NumericalValue, PropagatingEffect,
};
use deep_causality_haft::MonadEffect3;
use deep_causality_num::{Complex, Quaternion};
use deep_causality_tensor::CausalTensor;
use deep_causality_uncertain::{
    MaybeUncertainBool, MaybeUncertainF64, UncertainBool, UncertainF64,
};
use std::collections::HashMap;

// Constructors
impl PropagatingEffect {
    /// Creates a new `PropagatingEffect` from a given `EffectValue`.
    ///
    /// This is a generic constructor that wraps any `EffectValue` into a `PropagatingEffect`
    /// with no error and no logs.
    ///
    /// # Arguments
    ///
    /// * `effect_value` - The `EffectValue` to wrap.
    ///
    /// # Returns
    ///
    /// A `PropagatingEffect` instance containing the given `EffectValue`.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality::{PropagatingEffect, EffectValue};
    ///
    /// let effect = PropagatingEffect::from_effect_value(EffectValue::Deterministic(true));
    /// assert!(matches!(effect.value, EffectValue::Deterministic(true)));
    /// ```
    pub fn from_effect_value(effect_value: EffectValue) -> Self {
        CausalMonad::pure(effect_value)
    }

    /// Creates a new `PropagatingEffect` of the `Deterministic` variant.
    ///
    /// # Arguments
    ///
    /// * `deterministic` - A boolean value representing the deterministic effect.
    ///
    /// # Returns
    ///
    /// A `PropagatingEffect::Deterministic` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality::PropagatingEffect;
    /// use deep_causality::EffectValue;
    ///
    /// let effect = PropagatingEffect::from_deterministic(true);
    /// assert!(matches!(effect.value, EffectValue::Deterministic(true)));
    /// ```
    pub fn from_deterministic(deterministic: bool) -> Self {
        CausalMonad::pure(EffectValue::Deterministic(deterministic))
    }

    /// Creates a new `PropagatingEffect` of the `Numerical` variant.
    ///
    /// # Arguments
    ///
    /// * `numerical` - A `NumericalValue` representing the numerical effect.
    ///
    /// # Returns
    ///
    /// A `PropagatingEffect::Numerical` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality::{PropagatingEffect, NumericValue, EffectValue};
    ///
    /// let effect = PropagatingEffect::from_numerical(123.45);
    /// assert!(matches!(effect.value, EffectValue::Number(NumericValue::F64(123.45))));
    /// ```
    pub fn from_numerical(numerical: NumericalValue) -> Self {
        CausalMonad::pure(EffectValue::Number(NumericValue::F64(numerical)))
    }

    /// Creates a new `PropagatingEffect` of the `Number` variant from a `NumericValue`.
    ///
    /// # Arguments
    ///
    /// * `numeric` - A `NumericValue` representing the numerical effect.
    ///
    /// # Returns
    ///
    /// A `PropagatingEffect` instance containing the `NumericValue`.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality::{PropagatingEffect, NumericValue, EffectValue};
    ///
    /// let effect = PropagatingEffect::from_numeric(NumericValue::F64(123.45));
    /// assert!(matches!(effect.value, EffectValue::Number(NumericValue::F64(123.45))));
    /// ```
    pub fn from_numeric(numeric: NumericValue) -> Self {
        CausalMonad::pure(EffectValue::Number(numeric))
    }

    /// Creates a new `PropagatingEffect` of the `Probabilistic` variant.
    ///
    /// # Arguments
    ///
    /// * `numerical` - A `NumericalValue` representing the probabilistic effect (e.g., a probability score).
    ///
    /// # Returns
    ///
    /// A `PropagatingEffect::Probabilistic` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality::{PropagatingEffect, EffectValue};
    ///
    /// let effect = PropagatingEffect::from_probabilistic(0.75);
    /// assert!(matches!(effect.value, EffectValue::Probabilistic(0.75)));
    /// ```
    pub fn from_probabilistic(numerical: NumericalValue) -> Self {
        CausalMonad::pure(EffectValue::Probabilistic(numerical))
    }
    /// Creates a new `PropagatingEffect` of the `Tensor` variant.
    ///
    /// # Arguments
    ///
    /// * `tensor` - A `CausalTensor<f64>` representing the tensor effect.
    ///
    /// # Returns
    ///
    /// A `PropagatingEffect::Tensor` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality::{PropagatingEffect, EffectValue};
    /// use deep_causality_tensor::CausalTensor;
    ///
    /// fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let tensor = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3])?;
    ///     let effect = PropagatingEffect::from_tensor(tensor.clone());
    ///     assert!(matches!(effect.value, EffectValue::Tensor(_)));
    ///     Ok(())
    /// }
    /// ```
    pub fn from_tensor(tensor: CausalTensor<f64>) -> Self {
        CausalMonad::pure(EffectValue::Tensor(tensor))
    }

    /// Creates a new `PropagatingEffect` of the `Complex` variant.
    ///
    /// # Arguments
    ///
    /// * `complex` - A `Complex<f64>` representing the complex number effect.
    ///
    /// # Returns
    ///
    /// A `PropagatingEffect` instance containing the `Complex<f64>`.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality::{PropagatingEffect, EffectValue};
    /// use deep_causality_num::Complex;
    ///
    /// let complex = Complex::new(1.0, 2.0);
    /// let effect = PropagatingEffect::from_complex(complex);
    /// assert!(matches!(effect.value, EffectValue::Complex(_)));
    /// ```
    pub fn from_complex(complex: Complex<f64>) -> Self {
        CausalMonad::pure(EffectValue::Complex(complex))
    }

    /// Creates a new `PropagatingEffect` of the `ComplexTensor` variant.
    ///
    /// # Arguments
    ///
    /// * `complex_tensor` - A `ComplexTensor` representing the complex tensor effect.
    ///
    /// # Returns
    ///
    /// A `PropagatingEffect::ComplexTensor` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality::{PropagatingEffect, EffectValue};
    /// use deep_causality_num::Complex;
    /// use deep_causality_tensor::CausalTensor;
    ///
    /// fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let complex_tensor = CausalTensor::new(vec![Complex::new(1.0, 2.0)], vec![1])?;
    ///     let effect = PropagatingEffect::from_complex_tensor(complex_tensor.clone());
    ///     assert!(matches!(effect.value, EffectValue::ComplexTensor(_)));
    ///     Ok(())
    /// }
    /// ```
    pub fn from_complex_tensor(complex_tensor: ComplexTensor) -> Self {
        CausalMonad::pure(EffectValue::ComplexTensor(complex_tensor))
    }

    /// Creates a new `PropagatingEffect` of the `Quaternion` variant.
    ///
    /// # Arguments
    ///
    /// * `quaternion` - A `Quaternion<f64>` representing the quaternion effect.
    ///
    /// # Returns
    ///
    /// A `PropagatingEffect` instance containing the `Quaternion<f64>`.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality::{PropagatingEffect, EffectValue};
    /// use deep_causality_num::Quaternion;
    ///
    /// let quaternion = Quaternion::new(1.0, 2.0, 3.0, 4.0);
    /// let effect = PropagatingEffect::from_quaternion(quaternion);
    /// assert!(matches!(effect.value, EffectValue::Quaternion(_)));
    /// ```
    pub fn from_quaternion(quaternion: Quaternion<f64>) -> Self {
        CausalMonad::pure(EffectValue::Quaternion(quaternion))
    }

    /// Creates a new `PropagatingEffect` of the `QuaternionTensor` variant.
    ///
    /// # Arguments
    ///
    /// * `quaternion_tensor` - A `CausalTensor<Quaternion<f64>>` representing the quaternion tensor effect.
    ///
    /// # Returns
    ///
    /// A `PropagatingEffect` instance containing the `CausalTensor<Quaternion<f64>>`.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality::{PropagatingEffect, EffectValue};
    /// use deep_causality_num::Quaternion;
    /// use deep_causality_tensor::CausalTensor;
    ///
    /// fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let quaternion_tensor = CausalTensor::new(vec![Quaternion::new(1.0, 2.0, 3.0, 4.0)], vec![1])?;
    ///     let effect = PropagatingEffect::from_quaternion_tensor(quaternion_tensor.clone());
    ///     assert!(matches!(effect.value, EffectValue::QuaternionTensor(_)));
    ///     Ok(())
    /// }
    /// ```
    pub fn from_quaternion_tensor(quaternion_tensor: CausalTensor<Quaternion<f64>>) -> Self {
        CausalMonad::pure(EffectValue::QuaternionTensor(quaternion_tensor))
    }

    /// Creates a new `PropagatingEffect` of the `UncertainBool` variant.
    ///
    /// # Arguments
    ///
    /// * `uncertain` - An `UncertainBool` value representing the uncertain boolean effect.
    ///
    /// # Returns
    ///
    /// A `PropagatingEffect::UncertainBool` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality::{PropagatingEffect, EffectValue};
    /// use deep_causality_uncertain::UncertainBool;
    ///
    /// let uncertain_bool = UncertainBool::point(true);
    /// let effect = PropagatingEffect::from_uncertain_bool(uncertain_bool.clone());
    /// assert!(matches!(effect.value, EffectValue::UncertainBool(_)));
    /// ```
    pub fn from_uncertain_bool(uncertain: UncertainBool) -> Self {
        CausalMonad::pure(EffectValue::UncertainBool(uncertain))
    }

    /// Creates a new `PropagatingEffect` of the `UncertainFloat` variant.
    ///
    /// # Arguments
    ///
    /// * `uncertain` - An `UncertainF64` value representing the uncertain float effect.
    ///
    /// # Returns
    ///
    /// A `PropagatingEffect::UncertainFloat` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality::{PropagatingEffect, EffectValue};
    /// use deep_causality_uncertain::UncertainF64;
    ///
    /// let uncertain_float = UncertainF64::point(1.0);
    /// let effect = PropagatingEffect::from_uncertain_float(uncertain_float.clone());
    /// assert!(matches!(effect.value, EffectValue::UncertainFloat(_)));
    /// ```
    pub fn from_uncertain_float(uncertain: UncertainF64) -> Self {
        CausalMonad::pure(EffectValue::UncertainFloat(uncertain))
    }

    /// Creates a new `PropagatingEffect` of the `MaybeUncertainBool` variant.
    ///
    /// # Arguments
    ///
    /// * `maybe_uncertain` - A `MaybeUncertainBool` value representing the possibly uncertain boolean effect.
    ///
    /// # Returns
    ///
    /// A `PropagatingEffect::MaybeUncertainBool` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality::{PropagatingEffect, EffectValue};
    /// use deep_causality_uncertain::MaybeUncertainBool;
    ///
    /// let maybe_uncertain_bool = MaybeUncertainBool::from_value(true);
    /// let effect = PropagatingEffect::from_maybe_uncertain_bool(maybe_uncertain_bool.clone());
    /// assert!(matches!(effect.value, EffectValue::MaybeUncertainBool(_)));
    /// ```
    pub fn from_maybe_uncertain_bool(maybe_uncertain_bool: MaybeUncertainBool) -> Self {
        CausalMonad::pure(EffectValue::MaybeUncertainBool(maybe_uncertain_bool))
    }

    /// Creates a new `PropagatingEffect` of the `MaybeUncertainFloat` variant.
    ///
    /// # Arguments
    ///
    /// * `maybe_uncertain` - A `MaybeUncertainF64` value representing the possibly uncertain float effect.
    ///
    /// # Returns
    ///
    /// A `PropagatingEffect::MaybeUncertainFloat` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality::{PropagatingEffect, EffectValue};
    /// use deep_causality_uncertain::MaybeUncertainF64;
    ///
    /// let maybe_uncertain_float = MaybeUncertainF64::from_value(1.0);
    /// let effect = PropagatingEffect::from_maybe_uncertain_float(maybe_uncertain_float.clone());
    /// assert!(matches!(effect.value, EffectValue::MaybeUncertainFloat(_)));
    /// ```
    pub fn from_maybe_uncertain_float(maybe_uncertain_float: MaybeUncertainF64) -> Self {
        CausalMonad::pure(EffectValue::MaybeUncertainFloat(maybe_uncertain_float))
    }

    /// Creates a new `PropagatingEffect` of the `ContextualLink` variant.
    ///
    /// # Arguments
    ///
    /// * `context_id` - The `ContextId` of the context.
    /// * `contextoid_id` - The `ContextoidId` of the linked contextoid.
    ///
    /// # Returns
    ///
    /// A `PropagatingEffect::ContextualLink` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality::{PropagatingEffect, ContextId, ContextoidId, EffectValue};
    ///
    /// let context_id = 1u64;
    /// let contextoid_id = 2u64;
    /// let effect = PropagatingEffect::from_contextual_link(context_id, contextoid_id);
    /// assert!(matches!(effect.value, EffectValue::ContextualLink(_, _)));
    /// ```
    pub fn from_contextual_link(context_id: ContextId, contextoid_id: ContextoidId) -> Self {
        CausalMonad::pure(EffectValue::ContextualLink(context_id, contextoid_id))
    }

    /// Creates a new `PropagatingEffect` of the `Map` variant from an existing `HashMap`.
    ///
    /// # Arguments
    ///
    /// * `map` - A `HashMap` containing `IdentificationValue` keys and boxed `PropagatingEffect` values.
    ///
    /// # Returns
    ///
    /// A `PropagatingEffect::Map` instance initialized with the given map.
    ///
    pub fn from_map(map: HashMap<IdentificationValue, Box<PropagatingEffect>>) -> Self {
        CausalMonad::pure(EffectValue::Map(map))
    }

    /// Creates a new `PropagatingEffect` of the `RelayTo` variant.
    ///
    /// This variant is used to dispatch a command that directs the reasoning engine to
    /// dynamically jump to a specific causaloid within the graph, passing an effect as input.
    ///
    /// # Arguments
    ///
    /// * `id` - The `usize` index of the target causaloid.
    /// * `effect` - A `Box<PropagatingEffect>` representing the effect to be passed as input to the target causaloid.
    ///
    /// # Returns
    ///
    /// A `PropagatingEffect::RelayTo` instance.
    ///
    pub fn from_relay_to(id: usize, effect: Box<PropagatingEffect>) -> Self {
        CausalMonad::pure(EffectValue::RelayTo(id, effect))
    }

    /// Creates a new `PropagatingEffect` that explicitly contains an error.
    ///
    /// This constructor is used when an operation results in a `CausalityError`,
    /// and the effect should propagate this error, short-circuiting further computations.
    /// The `value` field is set to `EffectValue::None` in this case.
    ///
    /// # Arguments
    ///
    /// * `err` - The `CausalityError` to be encapsulated in the effect.
    ///
    /// # Returns
    ///
    /// A `PropagatingEffect` instance with the specified error.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality::{PropagatingEffect, CausalityError};
    ///
    /// let error_effect = PropagatingEffect::from_error(CausalityError::new("Something went wrong".to_string()));
    /// assert!(error_effect.is_err());
    /// ```
    pub fn from_error(err: CausalityError) -> Self {
        Self {
            value: EffectValue::None,
            error: Some(err),
            logs: Vec::new(),
        }
    }
}
