/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Causable, CausalityError, NumericalValue};

/// Provides default implementations for reasoning over collections of `Causable` items.
///
/// Any collection type that implements the basic accessor methods (`len`, `is_empty`,
/// `to_vec`, `get_all_items`) will automatically gain a suite of useful default
/// methods for inspecting the collective state of its `Causable` elements.
pub trait CausableReasoning<T>
where
    T: Causable,
{
    //
    // These methods must be implemented by the collection type.
    //

    /// Returns the total number of `Causable` items in the collection.
    fn len(&self) -> usize;

    /// Checks if the collection of `Causable` items is empty.
    fn is_empty(&self) -> bool;

    /// Creates a new vector containing the `Causable` items from the collection.
    fn to_vec(&self) -> Vec<T>;

    /// Returns a vector of references to all `Causable` items in the collection.
    /// This is the primary accessor used by the trait's default methods.
    fn get_all_items(&self) -> Vec<&T>;

    //
    // Default implementations for all other methods are provided below.
    //

    /// Checks if all causes in the collection are active.
    ///
    /// Iterates through all items and returns `Ok(false)` if any item's `is_active()`
    /// method returns `Ok(false)`. Returns `Ok(true)` if the collection is empty.
    /// Propagates any `Err` from `is_active`.
    fn get_all_causes_true(&self) -> Result<bool, CausalityError> {
        for cause in self.get_all_items() {
            if !cause.is_active()? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Returns a vector containing references to all active causes.
    /// Propagates any `Err` from `is_active`.
    fn get_all_active_causes(&self) -> Result<Vec<&T>, CausalityError> {
        let mut active_causes = Vec::new();
        for cause in self.get_all_items() {
            if cause.is_active()? {
                active_causes.push(cause);
            }
        }
        Ok(active_causes)
    }

    /// Returns a vector containing references to all inactive causes.
    /// Propagates any `Err` from `is_active`.
    fn get_all_inactive_causes(&self) -> Result<Vec<&T>, CausalityError> {
        let mut inactive_causes = Vec::new();
        for cause in self.get_all_items() {
            if !cause.is_active()? {
                inactive_causes.push(cause);
            }
        }
        Ok(inactive_causes)
    }

    /// Returns the number of active causes as a `NumericalValue`.
    /// Propagates any `Err` from `is_active`.
    fn number_active(&self) -> Result<NumericalValue, CausalityError> {
        let mut count = 0;
        for c in self.get_all_items() {
            if c.is_active()? {
                count += 1;
            }
        }
        Ok(count as NumericalValue)
    }

    /// Calculates the percentage of active causes.
    ///
    /// Returns `Ok(0.0)` if the collection is empty to avoid division by zero.
    /// Propagates any `Err` from `number_active`.
    fn percent_active(&self) -> Result<NumericalValue, CausalityError> {
        let total = self.len() as NumericalValue;
        if total == 0.0 {
            return Ok(0.0);
        }
        let count = self.number_active()?;
        Ok((count / total) * 100.0)
    }

    /// Generates an explanation by concatenating the `explain()` text of all causes.
    ///
    /// Each explanation is formatted and separated by newlines.
    /// It gracefully handles errors from individual `explain` calls by inserting
    /// a placeholder error message.
    fn explain(&self) -> String {
        let mut explanation = String::new();
        for cause in self.get_all_items() {
            let cause_explanation = match cause.explain() {
                Ok(s) => s,
                Err(e) => format!("[Error explaining cause {} ('{}')]", cause.id(), e),
            };

            explanation.push('\n');
            explanation.push_str(format!(" * {cause_explanation}").as_str());
            explanation.push('\n');
        }
        explanation
    }
}
