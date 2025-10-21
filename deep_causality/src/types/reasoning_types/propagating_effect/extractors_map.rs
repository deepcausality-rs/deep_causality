/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{CausalityError, IdentificationValue, NumericalValue, PropagatingEffect};

// Map-specific methods
impl PropagatingEffect {
    /// Inserts a key-value pair into the `PropagatingEffect` if it is a `Map` variant.
    ///
    /// # Arguments
    ///
    /// * `key` - The `IdentificationValue` to be used as the key in the map.
    /// * `value` - The `PropagatingEffect` to be inserted as the value.
    ///
    /// # Panics
    ///
    /// Panics if the `PropagatingEffect` instance is not a `Map` variant, as insertion is only
    /// supported for map-like effects.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality::{PropagatingEffect, IdentificationValue};
    /// use std::collections::HashMap;
    ///
    /// let mut effect_map = PropagatingEffect::Map(HashMap::new());
    /// let key = IdentificationValue::from(1u64);
    /// let value = PropagatingEffect::Numerical(42.0);
    /// effect_map.insert(key.clone(), value.clone());
    ///
    /// if let PropagatingEffect::Map(map) = effect_map {
    ///     assert!(map.contains_key(&key));
    /// }
    /// ```
    pub fn insert(&mut self, key: IdentificationValue, value: PropagatingEffect) {
        if let PropagatingEffect::Map(map) = self {
            map.insert(key, Box::new(value));
        } else {
            panic!("Cannot insert into PropagatingEffect that is not a Map variant");
        }
    }

    /// Retrieves a numerical value from a `PropagatingEffect::Map` variant by its key.
    ///
    /// This method attempts to retrieve a `PropagatingEffect` associated with the given `key`
    /// from the internal map. If found, it then checks if the retrieved effect is of the
    /// `Numerical` variant and returns its `NumericalValue`.
    ///
    /// # Arguments
    ///
    /// * `key` - The `IdentificationValue` representing the key to look up in the map.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    /// - `Ok(NumericalValue)` if the key is found and its associated effect is `Numerical`.
    /// - `Err(CausalityError)` if the `PropagatingEffect` is not a `Map` variant,
    ///   if no effect is found for the given key, or if the found effect is not `Numerical`.
    ///
    /// # Errors
    ///
    /// Returns a `CausalityError` in the following cases:
    /// - The `PropagatingEffect` instance is not a `Map` variant.
    /// - No effect is found for the provided `key`.
    /// - The effect found for the `key` is not of the `Numerical` variant.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality::{PropagatingEffect, IdentificationValue};
    /// use std::collections::HashMap;
    ///
    /// let mut effect_map = PropagatingEffect::Map(HashMap::new());
    /// let key = IdentificationValue::from(1u64);
    /// let numerical_value = PropagatingEffect::Numerical(123.45);
    /// effect_map.insert(key.clone(), numerical_value);
    ///
    /// let result = effect_map.get_numerical_from_map(key.clone()).unwrap();
    /// assert_eq!(result, 123.45);
    ///
    /// let non_existent_key = IdentificationValue::from(2u64);
    /// assert!(effect_map.get_numerical_from_map(non_existent_key).is_err());
    /// ```
    pub fn get_numerical_from_map(
        &self,
        key: IdentificationValue,
    ) -> Result<NumericalValue, CausalityError> {
        if let PropagatingEffect::Map(map) = self {
            match map.get(&key) {
                Some(effect) => {
                    if let PropagatingEffect::Numerical(val) = **effect {
                        Ok(val)
                    } else {
                        Err(CausalityError(format!(
                            "Effect for key '{key}' is not of type Numerical"
                        )))
                    }
                }
                None => Err(CausalityError(format!("No effect found for key '{key}'"))),
            }
        } else {
            Err(CausalityError(
                "Cannot get value by key from PropagatingEffect that is not a Map variant".into(),
            ))
        }
    }

    /// Retrieves a deterministic boolean value from a `PropagatingEffect::Map` variant by its key.
    ///
    /// This method attempts to retrieve a `PropagatingEffect` associated with the given `key`
    /// from the internal map. If found, it then checks if the retrieved effect is of the
    /// `Deterministic` variant and returns its `bool` value.
    ///
    /// # Arguments
    ///
    /// * `key` - The `IdentificationValue` representing the key to look up in the map.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    /// - `Ok(bool)` if the key is found and its associated effect is `Deterministic`.
    /// - `Err(CausalityError)` if the `PropagatingEffect` is not a `Map` variant,
    ///   if no effect is found for the given key, or if the found effect is not `Deterministic`.
    ///
    /// # Errors
    ///
    /// Returns a `CausalityError` in the following cases:
    /// - The `PropagatingEffect` instance is not a `Map` variant.
    /// - No effect is found for the provided `key`.
    /// - The effect found for the `key` is not of the `Deterministic` variant.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality::{PropagatingEffect, IdentificationValue};
    /// use std::collections::HashMap;
    ///
    /// let mut effect_map = PropagatingEffect::Map(HashMap::new());
    /// let key = IdentificationValue::from(1u64);
    /// let boolean_value = PropagatingEffect::Deterministic(true);
    /// effect_map.insert(key.clone(), boolean_value);
    ///
    /// let result = effect_map.get_deterministic_from_map(key.clone()).unwrap();
    /// assert_eq!(result, true);
    ///
    /// let non_existent_key = IdentificationValue::from(2u64);
    /// assert!(effect_map.get_deterministic_from_map(non_existent_key).is_err());
    /// ```
    pub fn get_deterministic_from_map(
        &self,
        key: IdentificationValue,
    ) -> Result<bool, CausalityError> {
        if let PropagatingEffect::Map(map) = self {
            match map.get(&key) {
                Some(effect) => {
                    if let PropagatingEffect::Deterministic(val) = **effect {
                        Ok(val)
                    } else {
                        Err(CausalityError(format!(
                            "Effect for key '{key}' is not of type Deterministic"
                        )))
                    }
                }
                None => Err(CausalityError(format!("No effect found for key '{key}'"))),
            }
        } else {
            Err(CausalityError(
                "Cannot get value by key from PropagatingEffect that is not a Map variant".into(),
            ))
        }
    }
}
