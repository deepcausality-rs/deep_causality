/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::errors::{ActionError, UpdateError};
use crate::prelude::{CausalAction, CausalState, Datable, NumericalValue, Symbolic};
use crate::traits::contextuable::space_temporal::SpaceTemporal;
use crate::traits::contextuable::spatial::Spatial;
use crate::traits::contextuable::temporal::Temporal;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, RwLock};

pub mod csm_action;
pub mod csm_state;

/// A tuple consisting of a causal state and an associated causal action.
///
/// This is used to represent the result of state-action reasoning steps.
pub type StateAction<D, S, T, ST, SYM, VS, VT> =
    (CausalState<D, S, T, ST, SYM, VS, VT>, CausalAction);
pub type CSMMap<D, S, T, ST, SYM, VS, VT> = HashMap<usize, StateAction<D, S, T, ST, SYM, VS, VT>>;

pub type CSMStateActions<D, S, T, ST, SYM, VS, VT> = [StateAction<D, S, T, ST, SYM, VS, VT>];

/// # Causal State Machine (CSM)
///
/// A Causal State Machine (CSM) is a structure that manages relationships between causal states and actions.
/// It provides a mechanism for evaluating states based on input data and triggering associated actions
/// when specific conditions are met.
///
/// ## Purpose
///
/// The CSM is designed to model systems where different states can trigger specific actions when
/// certain conditions are met. It's particularly useful for:
///
/// - Event-driven systems where actions should be triggered based on state changes
/// - Monitoring systems that need to respond to specific conditions
/// - Control systems that need to take actions based on sensor readings
/// - Any system where cause-effect relationships need to be modeled and evaluated
///
/// ## How It Works
///
/// 1. The CSM maintains a collection of causal states paired with actions
/// 2. Each causal state contains a causaloid that defines when the state should trigger its action
/// 3. When data is fed into the CSM, it evaluates the relevant states
/// 4. If a state's conditions are met (evaluated to true), the associated action is triggered
///
/// ## Usage
///
/// The CSM is typically used by:
///
/// 1. Creating causal states with appropriate causaloids that define trigger conditions
/// 2. Creating actions that should be executed when states are triggered
/// 3. Initializing a CSM with state-action pairs
/// 4. Feeding data into the CSM for evaluation
///
/// See the example in `examples/csm/src/main.rs` for a practical implementation.
pub struct CSM<D, S, T, ST, SYM, VS, VT>
where
    D: Datable + Clone,
    S: Spatial<VS> + Clone + Debug,
    T: Temporal<VT> + Clone + Debug,
    ST: SpaceTemporal<VS, VT> + Clone + Debug,
    SYM: Symbolic + Clone + Debug,
    VS: Clone + Debug,
    VT: Clone + Debug,
{
    state_actions: Arc<RwLock<CSMMap<D, S, T, ST, SYM, VS, VT>>>,
}

impl<D, S, T, ST, SYM, VS, VT> CSM<D, S, T, ST, SYM, VS, VT>
where
    D: Datable + Clone + Debug,
    S: Spatial<VS> + Clone + Debug,
    T: Temporal<VT> + Clone + Debug,
    ST: SpaceTemporal<VS, VT> + Clone + Debug,
    SYM: Symbolic + Clone + Debug,
    VS: Clone + Debug,
    VT: Clone + Debug,
{
    /// Constructs a new CSM.
    pub fn new(state_actions: &[(&CausalState<D, S, T, ST, SYM, VS, VT>, &CausalAction)]) -> Self {
        let mut map = HashMap::with_capacity(state_actions.len());

        for (state, action) in state_actions {
            map.insert(*state.id(), ((*state).clone(), (*action).clone()));
        }

        Self {
            state_actions: Arc::new(RwLock::new(map)),
        }
    }

    /// Returns the number of elements in the CSM.
    pub fn len(&self) -> usize {
        self.state_actions.read().unwrap().len()
    }

    /// Returns true if the CSM contains no elements.
    pub fn is_empty(&self) -> bool {
        self.state_actions.read().unwrap().is_empty()
    }
    /// Inserts a new state action at the index position idx.
    /// Returns UpdateError if the index already exists.
    pub fn add_single_state(
        &self,
        idx: usize,
        state_action: StateAction<D, S, T, ST, SYM, VS, VT>,
    ) -> Result<(), UpdateError> {
        // Check if the key exists, if so return error
        if self.state_actions.read().unwrap().get(&idx).is_some() {
            return Err(UpdateError(format!("State {} already exists.", idx)));
        }

        // Insert the new state/action at the idx position
        self.state_actions
            .write()
            .unwrap()
            .insert(idx, state_action);

        Ok(())
    }

    /// Removes a state action at the index position idx.
    /// Returns UpdateError if the index does not exists.
    pub fn remove_single_state(&self, id: usize) -> Result<(), UpdateError> {
        // Need binding to prevent dropped tmp value warnings
        let mut binding = self.state_actions.write().unwrap();

        // Check if state actually exists in the HashMap
        let state_action = binding.get(&id);
        if state_action.is_none() {
            return Err(UpdateError(format!(
                "State {} does not exists and  cannot be removed",
                id
            )));
        }

        // remove the new state/action at the idx position
        binding.remove(&id);

        Ok(())
    }

    /// Evaluates a single causal state at the index position idx.
    /// Returns ActionError if the evaluation failed.
    pub fn eval_single_state(&self, id: usize, data: NumericalValue) -> Result<(), ActionError> {
        // Need binding to prevent dropped tmp value warnings
        let binding = self.state_actions.read().unwrap();

        // Check if state actually exists in the HashMap
        let state_action = binding.get(&id);
        if state_action.is_none() {
            return Err(ActionError(format!(
                "State {} does not exists. Add it first before evaluating",
                id
            )));
        }

        // State exists, extract it.
        let (state, action) = state_action.unwrap();

        // Apply data and evaluate causal state
        let eval = state.eval_with_data(&data);

        // Check if the causal state evaluation returned an error
        if eval.is_err() {
            return Err(ActionError(format!(
                "CSM[eval]: Error evaluating causal state: {:?}",
                state
            )));
        }

        // Unpack the bool result that triggers the action
        let trigger =
            eval.expect("CSM[eval]: Failed to unwrap evaluation result from causal state}");

        // If the state evaluated to true, fire the associated action.
        if trigger && action.fire().is_err() {
            return Err(ActionError(format!(
                "CSM[eval]: Failed to fire action associated with causal state {:?}",
                state
            )));
        }

        Ok(())
    }

    /// Updates a causal state with a new state at the index position idx.
    /// Returns UpdateError if the update operation failed.
    pub fn update_single_state(
        &self,
        idx: usize,
        state_action: StateAction<D, S, T, ST, SYM, VS, VT>,
    ) -> Result<(), UpdateError> {
        // Check if the key exists, if not return error
        if self.state_actions.read().unwrap().get(&idx).is_none() {
            return Err(UpdateError(format!(
                "State {} does not exists. Add it first before evaluating",
                idx
            )));
        }

        // Update state/action at the idx position
        self.state_actions
            .write()
            .unwrap()
            .insert(idx, state_action);

        Ok(())
    }

    /// Evaluates all causal states in the CSM.
    /// Returns ActionError if the evaluation failed.
    pub fn eval_all_states(&self) -> Result<(), ActionError> {
        for (_, (state, action)) in self.state_actions.read().unwrap().iter() {
            let eval = state.eval();

            // check if the causal state evaluation returned an error
            if eval.is_err() {
                return Err(ActionError(format!(
                    "CSM[eval]: Error evaluating causal state: {:?}",
                    state
                )));
            }

            // Unpack the bool result
            let trigger =
                eval.expect("CSM[eval]: Failed to unwrap evaluation result from causal state}");

            // If the state evaluated to true, fire the associated action.
            if trigger && action.fire().is_err() {
                return Err(ActionError(format!(
                    "CSM[eval]: Failed to fire action associated with causal state {:?}",
                    state
                )));
            }
        }

        Ok(())
    }

    /// Updates all causal state with a new state collection.
    /// Note, this operation erases all previous states in the CSM by generating a new collection.
    /// Returns UpdateError if the update operation failed.
    pub fn update_all_states(
        &self,
        state_actions: &[(&CausalState<D, S, T, ST, SYM, VS, VT>, &CausalAction)],
    ) -> Result<(), UpdateError> {
        let mut state_map = HashMap::with_capacity(state_actions.len());

        for (state, action) in state_actions {
            state_map.insert(*state.id(), ((*state).clone(), (*action).clone()));
        }
        
        // Replace the existing map with the newly generated one.
        match self.state_actions.write() {
            Ok(mut guard) => {
                *guard = state_map;
                Ok(())
            }
            Err(_) => Err(UpdateError("Failed to acquire write lock while updating CSM states".to_string())),
        }
    }
}
