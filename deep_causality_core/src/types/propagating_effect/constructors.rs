use crate::types::monad_types::causal_monad::CausalMonad;
use crate::{
    CausalEffectLog, CausalPropagatingEffect, CausalityError, ContextoidId, EffectValue,
    IdentificationValue, NumericValue, PropagatingEffect,
};
use alloc::boxed::Box;
use core::fmt::Debug;
use deep_causality_haft::MonadEffect3;
#[cfg(feature = "std")]
use std::collections::HashMap;

// Constructors
impl<Value, Error, Log> CausalPropagatingEffect<Value, Error, Log>
where
    Value: Default + Clone + Debug,
    Error: Default + Clone + Debug,
    Log: Default + Clone + deep_causality_haft::LogAppend,
{
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
    /// A `CausalPropagatingEffect` instance with the specified error.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality::{CausalPropagatingEffect, CausalityError, CausalEffectLog};
    /// use core::fmt::Debug;
    ///
    /// #[derive(Default, Clone, Debug)]
    /// struct MyLog;
    /// impl deep_causality_haft::LogAppend for MyLog {
    ///    fn append(&mut self, other: &mut Self) {}
    /// }
    ///
    /// let error_effect = CausalPropagatingEffect::<(), CausalityError, CausalEffectLog>::from_error(CausalityError::new("Something went wrong".to_string()));
    /// assert!(error_effect.is_err());
    /// ```
    pub fn from_error(
        err: CausalityError,
    ) -> CausalPropagatingEffect<Value, CausalityError, CausalEffectLog> {
        CausalPropagatingEffect {
            value: EffectValue::None,
            error: Some(err),
            logs: CausalEffectLog::new(),
        }
    }

    /// Creates a new `CausalPropagatingEffect` with `EffectValue::None`, no error, and no logs.
    ///
    /// This is useful for representing an effect that carries no specific value or outcome,
    /// and is not associated with any error or log entries.
    ///
    /// # Returns
    ///
    /// A `CausalPropagatingEffect` instance with `EffectValue::None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality::{CausalPropagatingEffect, EffectValue, CausalityError, CausalEffectLog};
    /// use core::fmt::Debug;
    ///
    /// #[derive(Default, Clone, Debug)]
    /// struct MyLog;
    /// impl deep_causality_haft::LogAppend for MyLog {
    ///    fn append(&mut self, other: &mut Self) {}
    /// }
    ///
    /// let none_effect = CausalPropagatingEffect::<(), CausalityError, CausalEffectLog>::none();
    /// assert!(matches!(none_effect.value, EffectValue::None));
    /// assert!(!none_effect.is_err());
    /// ```
    pub fn none() -> CausalPropagatingEffect<Value, CausalityError, CausalEffectLog> {
        CausalPropagatingEffect {
            value: EffectValue::None,
            error: None,
            logs: CausalEffectLog::new(),
        }
    }

    /// Creates a new `CausalPropagatingEffect` from a given `EffectValue`.
    ///
    /// This is a generic constructor that wraps any `EffectValue` into a `CausalPropagatingEffect`
    /// with no error and no logs.
    ///
    /// # Arguments
    ///
    /// * `effect_value` - The `EffectValue` to wrap.
    ///
    /// # Returns
    ///
    /// A `CausalPropagatingEffect` instance containing the given `EffectValue`.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality::{CausalPropagatingEffect, EffectValue, CausalityError, CausalEffectLog};
    ///
    /// let effect = CausalPropagatingEffect::<bool, CausalityError, CausalEffectLog>::from_effect_value(EffectValue::Value(true));
    /// assert!(matches!(effect.value, EffectValue::Value(true)));
    /// ```
    pub fn from_effect_value(
        effect_value: EffectValue<Value>,
    ) -> CausalPropagatingEffect<Value, CausalityError, CausalEffectLog> {
        CausalPropagatingEffect {
            value: effect_value,
            error: None,
            logs: CausalEffectLog::new(),
        }
    }

    /// Creates a new `CausalPropagatingEffect` from a given `EffectValue` and `CausalEffectLog`.
    ///
    /// This constructor allows initializing a `CausalPropagatingEffect` with both a value
    /// and associated logs, but no error.
    ///
    /// # Arguments
    ///
    /// * `value` - The `EffectValue` to wrap.
    /// * `logs` - The `CausalEffectLog` containing any causal effect logs.
    ///
    /// # Returns
    ///
    /// A `CausalPropagatingEffect` instance containing the given `EffectValue` and logs.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality::{CausalPropagatingEffect, EffectValue, CausalEffectLog, CausalityError};
    ///
    /// let logs = CausalEffectLog::new();
    /// let effect = CausalPropagatingEffect::<bool, CausalityError, CausalEffectLog>::from_effect_value_with_log(EffectValue::Value(true), logs);
    /// assert!(matches!(effect.value, EffectValue::Value(true)));
    /// assert!(!effect.is_err());
    /// ```
    pub fn from_effect_value_with_log(
        value: EffectValue<Value>,
        logs: CausalEffectLog,
    ) -> CausalPropagatingEffect<Value, CausalityError, CausalEffectLog> {
        CausalPropagatingEffect {
            value,
            error: None,
            logs,
        }
    }

    /// Creates a new `CausalPropagatingEffect` of the `Map` variant from an existing `HashMap`.
    ///
    /// # Arguments
    ///
    /// * `map` - A `HashMap` containing `IdentificationValue` keys and boxed `CausalPropagatingEffect` values.
    ///
    /// # Returns
    ///
    /// A `CausalPropagatingEffect` instance initialized with the given map.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality::{CausalPropagatingEffect, EffectValue, CausalityError, CausalEffectLog, IdentificationValue};
    /// use alloc::boxed::Box;
    /// use std::collections::HashMap;
    ///
    /// let mut map = HashMap::new();
    /// map.insert(1, Box::new(CausalPropagatingEffect::from_boolean(true)));
    /// let effect = CausalPropagatingEffect::<(), CausalityError, CausalEffectLog>::from_map(map);
    /// assert!(matches!(effect.value, EffectValue::Map(_)));
    /// ```
    #[cfg(feature = "std")]
    pub fn from_map(
        map: HashMap<
            IdentificationValue,
            Box<PropagatingEffect<Value, CausalityError, CausalEffectLog>>,
        >,
    ) -> CausalPropagatingEffect<Value, CausalityError, CausalEffectLog> {
        CausalPropagatingEffect {
            value: EffectValue::Map(map),
            error: None,
            logs: CausalEffectLog::new(),
        }
    }

    /// Creates a new `CausalPropagatingEffect` of the `RelayTo` variant.
    ///
    /// This variant is used to dispatch a command that directs the reasoning engine to
    /// dynamically jump to a specific causaloid within the graph, passing an effect as input.
    ///
    /// # Arguments
    ///
    /// * `id` - The `usize` index of the target causaloid.
    /// * `effect` - A `Box<CausalPropagatingEffect>` representing the effect to be passed as input to the target causaloid.
    ///
    /// # Returns
    ///
    /// A `CausalPropagatingEffect` instance containing the relay command.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality::{CausalPropagatingEffect, EffectValue, CausalityError, CausalEffectLog};
    /// use alloc::boxed::Box;
    ///
    /// let effect_to_relay = CausalPropagatingEffect::from_boolean(false);
    /// let effect = CausalPropagatingEffect::<(), CausalityError, CausalEffectLog>::from_relay_to(1, Box::new(effect_to_relay));
    /// assert!(matches!(effect.value, EffectValue::RelayTo(_, _)));
    /// ```
    pub fn from_relay_to(
        id: usize,
        effect: Box<PropagatingEffect<Value, CausalityError, CausalEffectLog>>,
    ) -> CausalPropagatingEffect<Value, CausalityError, CausalEffectLog> {
        CausalPropagatingEffect {
            value: EffectValue::RelayTo(id, effect),
            error: None,
            logs: CausalEffectLog::new(),
        }
    }
}

impl CausalPropagatingEffect<bool, CausalityError, CausalEffectLog> {
    /// Creates a new `CausalPropagatingEffect` of type `bool`.
    ///
    /// # Arguments
    ///
    /// * `boolean` - A boolean value.
    ///
    /// # Returns
    ///
    /// A `CausalPropagatingEffect` instance containing the boolean value.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality::{CausalPropagatingEffect, EffectValue, CausalityError, CausalEffectLog};
    ///
    /// let effect = CausalPropagatingEffect::from_boolean(true);
    /// assert!(matches!(effect.value, EffectValue::Value(true)));
    /// ```
    pub fn from_boolean(boolean: bool) -> Self {
        CausalMonad::pure(boolean)
    }
}

impl CausalPropagatingEffect<f64, CausalityError, CausalEffectLog> {
    /// Creates a new `CausalPropagatingEffect` of type `f64`.
    ///
    /// # Arguments
    ///
    /// * `numerical` - A `f64` value representing the numerical effect.
    ///
    /// # Returns
    ///
    /// A `CausalPropagatingEffect` instance containing the `f64` value.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality::{CausalPropagatingEffect, EffectValue, CausalityError, CausalEffectLog};
    ///
    /// let effect = CausalPropagatingEffect::from_f64(123.45);
    /// assert!(matches!(effect.value, EffectValue::Value(123.45)));
    /// ```
    pub fn from_f64(numerical: f64) -> Self {
        CausalMonad::pure(numerical)
    }
}

impl CausalPropagatingEffect<NumericValue, CausalityError, CausalEffectLog> {
    /// Creates a new `CausalPropagatingEffect` from a `NumericValue`.
    ///
    /// # Arguments
    ///
    /// * `numeric` - A `NumericValue` representing the numerical effect.
    ///
    /// # Returns
    ///
    /// A `CausalPropagatingEffect` instance containing the `NumericValue`.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality::{CausalPropagatingEffect, NumericValue, EffectValue, CausalityError, CausalEffectLog};
    ///
    /// let effect = CausalPropagatingEffect::from_numeric(NumericValue::F64(123.45));
    /// assert!(matches!(effect.value, EffectValue::Value(NumericValue::F64(123.45))));
    /// ```
    pub fn from_numeric(numeric: NumericValue) -> Self {
        CausalMonad::pure(numeric)
    }

    /// Creates a new `CausalPropagatingEffect` of the `ContextualLink` variant.
    ///
    /// # Arguments
    ///
    /// * `context_id` - The `ContextId` of the context.
    /// * `contextoid_id` - The `ContextoidId` of the linked contextoid.
    ///
    /// # Returns
    ///
    /// A `CausalPropagatingEffect` instance containing the contextual link.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality::{CausalPropagatingEffect, EffectValue, CausalityError, CausalEffectLog};
    ///
    /// let effect = CausalPropagatingEffect::<(), CausalityError, CausalEffectLog>::from_contextual_link(23, 42);
    /// assert!(matches!(effect.value, EffectValue::ContextualLink(_,_)));
    /// ```
    pub fn from_contextual_link(context_id: ContextoidId, contextoid_id: ContextoidId) -> Self {
        CausalPropagatingEffect {
            value: EffectValue::ContextualLink(context_id, contextoid_id),
            error: None,
            logs: CausalEffectLog::new(),
        }
    }
}
