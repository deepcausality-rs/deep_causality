/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_macros::{Constructor, Getters};

use crate::ActionError;

/// A `CausalAction` represents an executable action that can be triggered in response to causal conditions.
///
/// In a causal state machine (CSM), actions are paired with causal states. When a causal state's
/// conditions are met (evaluated to true), the associated action is fired.
///
/// # Purpose
/// `CausalAction` encapsulates a function that performs a specific task, such as:
/// - Triggering alerts or alarms
/// - Logging events
/// - Sending notifications
/// - Executing control operations
///
/// # Usage
/// `CausalAction` is typically used in conjunction with `CausalState` in a state-action pair
/// within a causal state machine (CSM). The CSM evaluates states and, when conditions are met,
/// fires the associated actions.
///
/// # Example
/// ```
/// use deep_causality::{ActionError, CausalAction};
///
/// fn get_alert_action() -> CausalAction {
///     let func = || {
///         println!("Alert triggered!");
///         Ok(())
///     };
///     let descr = "Action that triggers an alert";
///     let version = 1;
///
///     CausalAction::new(func, descr, version)
/// }
/// ```
#[allow(clippy::type_complexity)]
#[derive(Getters, Constructor, Clone, Debug)]
pub struct CausalAction {
    // The function to execute when the action is fired.
    // This function should return `Ok(())` on success or an `ActionError` on failure.
    action: fn() -> Result<(), ActionError>,
    // A description of what the action does.
    descr: &'static str,
    // The version number of the action, useful for tracking changes or updates.
    version: usize,
}

impl CausalAction {
    /// Executes the action function encapsulated by this `CausalAction`.
    ///
    /// This method is typically called by a causal state machine (CSM) when
    /// the associated causal state's conditions are met.
    ///
    /// # Returns
    /// - `Ok(())` if the action executes successfully
    /// - `Err(ActionError)` if the action fails
    ///
    /// # Example
    /// ```
    /// use deep_causality::{ActionError, CausalAction};
    ///
    /// // Create a CausalAction
    /// let action = CausalAction::new(
    ///     || { println!("Action executed!"); Ok(()) },
    ///     "Example action",
    ///     1
    /// );
    ///
    /// // Fire the action
    /// match action.fire() {
    ///     Ok(()) => println!("Action completed successfully"),
    ///     Err(e) => println!("Action failed: {}", e),
    /// }
    /// ```
    pub fn fire(&self) -> Result<(), ActionError> {
        (self.action)()
    }
}
