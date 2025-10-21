/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{CausalityError, IdentificationValue, NumericalValue, PropagatingEffect};
use std::collections::HashMap;

// Map-specific methods
impl PropagatingEffect {
    /// Creates a new empty Effect Map.
    pub fn new_map() -> Self {
        PropagatingEffect::Map(HashMap::new())
    }

    /// Inserts a key-value pair into an Effect Map.
    /// Panics if the Effect is not a Map variant.
    pub fn insert(&mut self, key: IdentificationValue, value: PropagatingEffect) {
        if let PropagatingEffect::Map(map) = self {
            map.insert(key, Box::new(value));
        } else {
            panic!("Cannot insert into PropagatingEffect that is not a Map variant");
        }
    }

    /// Retrieves a numerical value from an Effect::Map by key.
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

    /// Retrieves a deterministic boolean value from an Effect::Map by key.
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
