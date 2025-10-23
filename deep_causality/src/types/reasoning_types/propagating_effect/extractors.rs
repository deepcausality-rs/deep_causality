/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{ComplexTensor, ContextId, ContextoidId, NumericalValue, PropagatingEffect};
use deep_causality_tensor::CausalTensor;
use deep_causality_uncertain::{MaybeUncertain, Uncertain};

// Extractors
impl PropagatingEffect {
    /// Attempts to extract a `bool` value from the `PropagatingEffect`.
    ///
    /// Returns `Some(bool)` if the effect is of the `Deterministic` variant, otherwise returns `None`.
    ///
    /// # Returns
    ///
    /// An `Option` containing the boolean value if the effect is `Deterministic`, or `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality::PropagatingEffect;
    ///
    /// let effect = PropagatingEffect::Deterministic(true);
    /// assert_eq!(effect.as_bool(), Some(true));
    ///
    /// let effect = PropagatingEffect::Numerical(1.0);
    /// assert_eq!(effect.as_bool(), None);
    /// ```
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            PropagatingEffect::Deterministic(b) => Some(*b),
            _ => None,
        }
    }

    /// Attempts to extract a `NumericalValue` from the `PropagatingEffect`.
    ///
    /// Returns `Some(NumericalValue)` if the effect is of the `Numerical` variant, otherwise returns `None`.
    ///
    /// # Returns
    ///
    /// An `Option` containing the numerical value if the effect is `Numerical`, or `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality::PropagatingEffect;
    ///
    /// let effect = PropagatingEffect::Numerical(123.45);
    /// assert_eq!(effect.as_numerical(), Some(123.45));
    ///
    /// let effect = PropagatingEffect::Deterministic(true);
    /// assert_eq!(effect.as_numerical(), None);
    /// ```
    pub fn as_numerical(&self) -> Option<NumericalValue> {
        match self {
            PropagatingEffect::Numerical(p) => Some(*p),
            _ => None,
        }
    }

    /// Attempts to extract a `NumericalValue` representing a probability from the `PropagatingEffect`.
    ///
    /// Returns `Some(NumericalValue)` if the effect is of the `Probabilistic` variant, otherwise returns `None`.
    ///
    /// # Returns
    ///
    /// An `Option` containing the probability value if the effect is `Probabilistic`, or `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality::PropagatingEffect;
    ///
    /// let effect = PropagatingEffect::Probabilistic(0.75);
    /// assert_eq!(effect.as_probability(), Some(0.75));
    ///
    /// let effect = PropagatingEffect::Deterministic(true);
    /// assert_eq!(effect.as_probability(), None);
    /// ```
    pub fn as_probability(&self) -> Option<NumericalValue> {
        match self {
            PropagatingEffect::Probabilistic(p) => Some(*p),
            _ => None,
        }
    }

    /// Attempts to extract a `CausalTensor<f64>` from the `PropagatingEffect`.
    ///
    /// Returns `Some(CausalTensor<f64>)` if the effect is of the `Tensor` variant, otherwise returns `None`.
    /// The tensor is cloned to avoid ownership issues.
    ///
    /// # Returns
    ///
    /// An `Option` containing the `CausalTensor<f64>` if the effect is `Tensor`, or `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality::PropagatingEffect;
    /// use deep_causality_tensor::CausalTensor;
    ///
    /// let tensor = CausalTensor::new(vec![1.0, 2.0], vec![2]).unwrap();
    /// let effect = PropagatingEffect::Tensor(tensor.clone());
    /// assert_eq!(effect.as_tensor().map(|t| t.data().clone()), Some(vec![1.0, 2.0]));
    ///
    /// let effect = PropagatingEffect::Numerical(1.0);
    /// assert_eq!(effect.as_tensor(), None);
    /// ```
    pub fn as_tensor(&self) -> Option<CausalTensor<f64>> {
        match self {
            PropagatingEffect::Tensor(p) => Some(p.clone()),
            _ => None,
        }
    }

    /// Attempts to extract a `ComplexTensor` from the `PropagatingEffect`.
    ///
    /// Returns `Some(ComplexTensor)` if the effect is of the `ComplexTensor` variant, otherwise returns `None`.
    /// The tensor is cloned to avoid ownership issues.
    ///
    /// # Returns
    ///
    /// An `Option` containing the `ComplexTensor` if the effect is `ComplexTensor`, or `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality::PropagatingEffect;
    /// use deep_causality_num::Complex;
    /// use deep_causality_tensor::CausalTensor;
    ///
    /// fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let complex_tensor = CausalTensor::new(vec![Complex::new(1.0, 2.0)], vec![1])?;
    ///     let effect = PropagatingEffect::ComplexTensor(complex_tensor.clone());
    ///     assert_eq!(effect.as_complex_tensor().map(|t| t.data().clone()), Some(vec![Complex::new(1.0, 2.0)]));
    ///
    ///     let effect = PropagatingEffect::Numerical(1.0);
    ///     assert_eq!(effect.as_complex_tensor(), None);
    ///     Ok(())
    /// }
    /// ```
    pub fn as_complex_tensor(&self) -> Option<ComplexTensor> {
        match self {
            PropagatingEffect::ComplexTensor(p) => Some(p.clone()),
            _ => None,
        }
    }

    /// Attempts to extract an `Uncertain<bool>` value from the `PropagatingEffect`.
    ///
    /// Returns `Some(Uncertain<bool>)` if the effect is of the `UncertainBool` variant, otherwise returns `None`.
    /// The uncertain boolean is cloned to avoid ownership issues.
    ///
    /// # Returns
    ///
    /// An `Option` containing the `Uncertain<bool>` if the effect is `UncertainBool`, or `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality::PropagatingEffect;
    /// use deep_causality_uncertain::{UncertainBool, UncertainError};
    ///
    /// fn main() -> Result<(), UncertainError> {
    ///     let uncertain_bool = UncertainBool::point(true);
    ///     let effect = PropagatingEffect::UncertainBool(uncertain_bool.clone());
    ///     assert_eq!(effect.as_uncertain_bool().map(|u| u.sample().unwrap()), Some(true));
    ///
    ///     let effect = PropagatingEffect::Numerical(1.0);
    ///     assert_eq!(effect.as_uncertain_bool(), None);
    ///     Ok(())
    /// }
    /// ```
    pub fn as_uncertain_bool(&self) -> Option<Uncertain<bool>> {
        match self {
            PropagatingEffect::UncertainBool(b) => Some(b.clone()),
            _ => None,
        }
    }

    /// Attempts to extract an `Uncertain<f64>` value from the `PropagatingEffect`.
    ///
    /// Returns `Some(Uncertain<f64>)` if the effect is of the `UncertainFloat` variant, otherwise returns `None`.
    /// The uncertain float is cloned to avoid ownership issues.
    ///
    /// # Returns
    ///
    /// An `Option` containing the `Uncertain<f64>` if the effect is `UncertainFloat`, or `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality::PropagatingEffect;
    /// use deep_causality_uncertain::{UncertainF64, UncertainError};
    ///
    /// fn main() -> Result<(), UncertainError> {
    ///     let uncertain_float = UncertainF64::point(1.0);
    ///     let effect = PropagatingEffect::UncertainFloat(uncertain_float.clone());
    ///     assert_eq!(effect.as_uncertain_float().map(|u| u.sample().unwrap()), Some(1.0));
    ///
    ///     let effect = PropagatingEffect::Deterministic(true);
    ///     assert_eq!(effect.as_uncertain_float(), None);
    ///     Ok(())
    /// }
    /// ```
    pub fn as_uncertain_float(&self) -> Option<Uncertain<f64>> {
        match self {
            PropagatingEffect::UncertainFloat(b) => Some(b.clone()),
            _ => None,
        }
    }

    /// Attempts to extract a `MaybeUncertain<bool>` value from the `PropagatingEffect`.
    ///
    /// Returns `Some(MaybeUncertain<bool>)` if the effect is of the `MaybeUncertainBool` variant, otherwise returns `None`.
    /// The maybe uncertain boolean is cloned to avoid ownership issues.
    ///
    /// # Returns
    ///
    /// An `Option` containing the `MaybeUncertain<bool>` if the effect is `MaybeUncertainBool`, or `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality::PropagatingEffect;
    /// use deep_causality_uncertain::{MaybeUncertainBool, UncertainError};
    ///
    /// fn main() -> Result<(), UncertainError> {
    ///     let maybe_uncertain_bool = MaybeUncertainBool::from_value(true);
    ///     let effect = PropagatingEffect::MaybeUncertainBool(maybe_uncertain_bool.clone());
    ///     assert_eq!(effect.as_maybe_uncertain_bool().map(|m| m.sample().unwrap()), Some(Some(true)));
    ///
    ///     let effect = PropagatingEffect::Numerical(1.0);
    ///     assert_eq!(effect.as_maybe_uncertain_bool(), None);
    ///     Ok(())
    /// }
    /// ```
    pub fn as_maybe_uncertain_bool(&self) -> Option<MaybeUncertain<bool>> {
        match self {
            PropagatingEffect::MaybeUncertainBool(b) => Some(b.clone()),
            _ => None,
        }
    }

    /// Attempts to extract a `MaybeUncertain<f64>` value from the `PropagatingEffect`.
    ///
    /// Returns `Some(MaybeUncertain<f64>)` if the effect is of the `MaybeUncertainFloat` variant, otherwise returns `None`.
    /// The maybe uncertain float is cloned to avoid ownership issues.
    ///
    /// # Returns
    ///
    /// An `Option` containing the `MaybeUncertain<f64>` if the effect is `MaybeUncertainFloat`, or `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality::PropagatingEffect;
    /// use deep_causality_uncertain::{MaybeUncertainF64, UncertainError};
    ///
    /// fn main() -> Result<(), UncertainError> {
    ///     let maybe_uncertain_float = MaybeUncertainF64::from_value(1.0);
    ///     let effect = PropagatingEffect::MaybeUncertainFloat(maybe_uncertain_float.clone());
    ///     assert_eq!(effect.as_maybe_uncertain_float().map(|m| m.sample().unwrap()), Some(Some(1.0)));
    ///
    ///     let effect = PropagatingEffect::Deterministic(true);
    ///     assert_eq!(effect.as_maybe_uncertain_float(), None);
    ///     Ok(())
    /// }
    /// ```
    pub fn as_maybe_uncertain_float(&self) -> Option<MaybeUncertain<f64>> {
        match self {
            PropagatingEffect::MaybeUncertainFloat(b) => Some(b.clone()),
            _ => None,
        }
    }

    /// Attempts to extract a `(ContextId, ContextoidId)` tuple from the `PropagatingEffect`.
    ///
    /// Returns `Some((ContextId, ContextoidId))` if the effect is of the `ContextualLink` variant, otherwise returns `None`.
    ///
    /// # Returns
    ///
    /// An `Option` containing the `(ContextId, ContextoidId)` tuple if the effect is `ContextualLink`, or `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality::PropagatingEffect;
    /// use deep_causality::{ContextId, ContextoidId};
    ///
    /// let context_id = 1u64;
    /// let contextoid_id = 2u64;
    /// let effect = PropagatingEffect::ContextualLink(context_id, contextoid_id);
    /// assert_eq!(effect.as_contextual_link(), Some((1u64, 2u64)));
    ///
    /// let effect = PropagatingEffect::Numerical(1.0);
    /// assert_eq!(effect.as_contextual_link(), None);
    /// ```
    pub fn as_contextual_link(&self) -> Option<(ContextId, ContextoidId)> {
        match self {
            PropagatingEffect::ContextualLink(context_id, contextoid_id) => {
                Some((*context_id, *contextoid_id))
            }
            _ => None,
        }
    }
}
