/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::collections::HashMap;

use crate::errors::CausalityError;
use crate::{Identifiable, IdentificationValue, NumericalValue};

/// The Causable trait defines the core behavior for causal reasoning.
///
/// It requires implementing the Identifiable trait.
///
/// # Trait Methods
///
/// * `explain` - Returns an explanation of the cause as a String.
/// * `is_active` - Returns true if this cause is currently active.
/// * `is_singleton` - Returns true if this cause acts on a single data point.
/// * `verify_single_cause` - Verifies this cause against a single data point.
/// * `verify_all_causes` - Verifies this cause against multiple data points.
///
/// `verify_single_cause` and `verify_all_causes` return a Result indicating
/// if the cause was validated or not.
///
/// # Examples
///
/// ```
/// use deep_causality::{Causable, Identifiable, IdentificationValue, NumericalValue, CausalityError};
/// use std::collections::HashMap;
///
/// struct MyCause {
///     id: IdentificationValue,
///     active: bool,
///     singleton: bool,
/// }
///
/// impl Identifiable for MyCause {
///     fn id(&self) -> IdentificationValue {
///         self.id
///     }
/// }
///
/// impl Causable for MyCause {
///     fn explain(&self) -> Result<String, CausalityError> {
///         Ok(format!("This is cause {}", self.id))
///     }
///
///     fn is_active(&self) -> bool {
///         self.active
///     }
///
///     fn is_singleton(&self) -> bool {
///         self.singleton
///     }
///
///     fn verify_single_cause(&self, obs: &NumericalValue) -> Result<bool, CausalityError> {
///         Ok(*obs > 0.0)
///     }
///
///     fn verify_all_causes(&self, data: &[NumericalValue], data_index: Option<&HashMap<IdentificationValue, IdentificationValue>>) -> Result<bool, CausalityError> {
///         Ok(data.iter().all(|&x| x > 0.0))
///     }
/// }
/// ```
pub trait Causable: Identifiable {
    /// Generates a human-readable explanation of the causaloid's current state.
    ///
    /// The nature of the explanation depends on the `CausaloidType`:
    /// - For a `Singleton`, it describes whether the causaloid is active.
    /// - For a `Collection`, it aggregates the explanations of the causaloids within it.
    /// - For a `Graph`, it explains all the causal paths that have been evaluated.
    ///
    /// # Returns
    ///
    /// - `Ok(String)` containing the explanation if the causaloid is active.
    /// - `Err(CausalityError)` if the causaloid has not been evaluated yet (i.e., is not active).
    fn explain(&self) -> Result<String, CausalityError>;

    /// Checks if the causaloid is currently considered active.
    ///
    /// The definition of "active" varies by `CausaloidType`:
    /// - For a `Singleton`, it's active if its causal function evaluated to `true`.
    /// - For a `Collection` or `Graph`, it's active if at least one of its contained
    ///   causaloids is active.
    ///
    /// # Returns
    ///
    /// `true` if the causaloid is active, `false` otherwise.
    fn is_active(&self) -> bool;

    /// Determines if the causaloid represents a single, indivisible causal unit.
    ///
    /// This method is crucial for dispatching to the correct verification logic.
    /// If this returns `true`, `verify_single_cause` should be used.
    /// If it returns `false`, `verify_all_causes` should be used.
    ///
    /// # Returns
    ///
    /// `true` if the `CausaloidType` is `Singleton`, `false` otherwise.
    fn is_singleton(&self) -> bool;

    /// Verifies a `Singleton` causaloid against a single numerical observation.
    ///
    /// This method should only be called when `is_singleton()` returns `true`. It executes
    /// the associated causal function (either contextual or non-contextual) against the
    /// provided observation and updates the causaloid's internal `active` state based on the result.
    ///
    /// # Arguments
    ///
    /// * `obs` - A reference to the `NumericalValue` to be evaluated.
    ///
    /// # Returns
    ///
    /// - `Ok(true)` if the causal condition is met.
    /// - `Ok(false)` if the causal condition is not met.
    /// - `Err(CausalityError)` if the required causal function is missing.
    fn verify_single_cause(&self, obs: &NumericalValue) -> Result<bool, CausalityError>;

    /// Verifies a `Collection` or `Graph` causaloid against a slice of numerical data.
    ///
    /// This method should be called when `is_singleton()` returns `false`. It reasons
    /// over the entire collection or graph of underlying causaloids.
    ///
    /// # Arguments
    ///
    /// * `data` - A slice of `NumericalValue` representing the dataset to verify against.
    /// * `data_index` - An optional `HashMap` that maps causaloid IDs to their corresponding
    ///   index in the `data` slice. This is primarily used for `Graph` types to ensure
    ///   the correct data point is passed to the correct causaloid node.
    ///
    /// # Returns
    ///
    /// - `Ok(true)` if the aggregate causal conditions of the collection/graph are met.
    /// - `Ok(false)` if the aggregate causal conditions are not met.
    /// - `Err(CausalityError)` if called on a `Singleton` causaloid or if the underlying
    ///   collection/graph is not properly initialized.
    fn verify_all_causes(
        &self,
        data: &[NumericalValue],
        data_index: Option<&HashMap<IdentificationValue, IdentificationValue>>,
    ) -> Result<bool, CausalityError>;
}

/// The CausableReasoning trait provides default implementations for reasoning over collections of Causable items.
///
/// It requires the generic type T to implement the Causable trait.
///
/// The trait provides default methods for:
///
/// - Getting active/inactive causes
/// - Counting active causes
/// - Calculating percentage of active causes
/// - Explaining all causes
/// - Verifying causes against data
///
/// The `reason_all_causes` method verifies all causes in the collection against the provided data,
/// using the cause's `is_singleton` method to determine whether to call `verify_single_cause` or
/// `verify_all_causes`.
///
/// An index is emulated for the data to enable singleton cause verification.
///
pub trait CausableReasoning<T>
where
    T: Causable,
{
    //
    // These methods can be generated by compiler macros.
    //
    /// Returns the total number of `Causable` items in the collection.
    ///
    /// This is a required method for the trait implementor, often fulfilled
    /// by a derive macro or a straightforward implementation on a collection type.
    fn len(&self) -> usize;

    /// Checks if the collection of `Causable` items is empty.
    ///
    /// # Returns
    ///
    /// `true` if the collection contains no items (`len() == 0`), `false` otherwise.
    ///
    /// This is a required method for the trait implementor.
    fn is_empty(&self) -> bool;

    /// Creates a new vector containing the `Causable` items from the collection.
    ///
    /// This method typically involves cloning the items to create an owned `Vec<T>`.
    ///
    /// # Returns
    ///
    /// A `Vec<T>` containing all the items from the collection.
    ///
    /// This is a required method for the trait implementor.
    fn to_vec(&self) -> Vec<T>;

    /// Returns a vector of references to all `Causable` items in the collection.
    ///
    /// This method provides non-owning, read-only access to the items and is
    /// heavily used by the default implementations of other reasoning methods
    /// in this trait (e.g., `get_all_active_causes`, `number_active`).
    ///
    /// # Returns
    ///
    /// A `Vec<&T>` containing references to all items.
    ///
    /// This is a required method for the trait implementor.
    fn get_all_items(&self) -> Vec<&T>;

    //
    // Default implementations for all other methods are provided below.
    //

    /// Checks if all causes in the collection are active.
    ///
    /// Iterates through all causes via `get_all_items()` and returns false
    /// if any cause's `is_active()` method returns false.
    ///
    /// If all causes are active, returns true.
    ///
    fn get_all_causes_true(&self) -> bool {
        for cause in self.get_all_items() {
            if !cause.is_active() {
                return false;
            }
        }

        true
    }

    /// Returns a vector containing references to all active causes.
    ///
    /// Gets all causes via `get_all_items()`, filters to keep only those where
    /// `is_active()` returns true, and collects into a vector.
    ///
    fn get_all_active_causes(&self) -> Vec<&T> {
        self.get_all_items()
            .into_iter()
            .filter(|cause| cause.is_active())
            .collect()
    }

    /// Returns a vector containing references to all inactive causes.
    ///
    /// Gets all causes via `get_all_items()`, filters to keep only those where
    /// `is_active()` returns false, and collects into a vector.
    ///
    fn get_all_inactive_causes(&self) -> Vec<&T> {
        self.get_all_items()
            .into_iter()
            .filter(|cause| !cause.is_active())
            .collect()
    }

    /// Returns the number of active causes.
    ///
    /// Gets all causes via `get_all_items()`, filters to keep only active ones,
    /// counts them, and returns the count as a NumericalValue.
    ///
    fn number_active(&self) -> NumericalValue {
        self.get_all_items()
            .iter()
            .filter(|c| c.is_active())
            .count() as NumericalValue
    }

    /// Calculates the percentage of active causes.
    ///
    /// Gets the number of active causes via `number_active()`.
    /// Gets the total number of causes via `len()`.
    /// Divides the active count by the total.
    /// Multiplies by 100 to get a percentage.
    /// Returns the result as a NumericalValue.
    ///
    fn percent_active(&self) -> NumericalValue {
        let count = self.number_active();
        let total = self.len() as NumericalValue;
        (count / total) * (100 as NumericalValue)
    }

    /// Verifies all causes in the collection against the provided data.
    ///
    /// Returns an error if the collection is empty.
    ///
    /// Iterates through all causes, using the cause's `is_singleton()` method
    /// to determine whether to call `verify_single_cause()` or `verify_all_causes()`.
    ///
    /// For singleton causes, the data index is emulated to enable lookup by index.
    ///
    /// If any cause fails verification, returns Ok(false).
    ///
    /// If all causes pass verification, returns Ok(true).
    ///
    fn reason_all_causes(&self, data: &[NumericalValue]) -> Result<bool, CausalityError> {
        if self.is_empty() {
            return Err(CausalityError("Causality collection is empty".into()));
        }

        // Emulate the data index using an enumerated iterator
        // assuming that values in the map have the same order as the data.
        for (i, cause) in self.get_all_items().iter().enumerate() {
            let valid = if cause.is_singleton() {
                cause.verify_single_cause(data.get(i).expect("failed to get value"))?
            } else {
                cause.verify_all_causes(data, None)?
            };

            if !valid {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Generates an explanation by concatenating the explain() text of all causes.
    ///
    /// Calls explain() on each cause and unwraps the result.
    /// Concatenates the explanations by inserting newlines between each one.
    ///
    /// Returns the concatenated explanation string.
    ///
    fn explain(&self) -> String {
        let mut explanation = String::new();
        for cause in self.get_all_items() {
            explanation.push('\n');
            explanation.push_str(format!(" * {}", cause.explain().unwrap()).as_str());
            explanation.push('\n');
        }
        explanation
    }
}
