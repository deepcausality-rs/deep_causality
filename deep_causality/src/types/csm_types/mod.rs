// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::Hash;
use std::ops::{Add, Mul, Sub};

use crate::errors::{ActionError, UpdateError};
use crate::prelude::{
    CausalAction, CausalState, Datable, NumericalValue, SpaceTemporal, Spatial, Temporable,
};

pub mod csm_action;
pub mod csm_state;

pub type CSMMap<'l, D, S, T, ST, V> =
    HashMap<usize, (&'l CausalState<'l, D, S, T, ST, V>, &'l CausalAction)>;
pub type CSMStateActions<'l, D, S, T, ST, V> =
    [(&'l CausalState<'l, D, S, T, ST, V>, &'l CausalAction)];

pub struct CSM<'l, D, S, T, ST, V>
where
    D: Datable + Clone + Copy,
    S: Spatial<V> + Clone + Copy,
    T: Temporable<V> + Clone + Copy,
    ST: SpaceTemporal<V> + Clone + Copy,
    V: Default
        + Copy
        + Clone
        + Hash
        + Eq
        + PartialEq
        + Add<V, Output = V>
        + Sub<V, Output = V>
        + Mul<V, Output = V>,
{
    state_actions: RefCell<CSMMap<'l, D, S, T, ST, V>>,
}

impl<'l, D, S, T, ST, V> CSM<'l, D, S, T, ST, V>
where
    D: Datable + Clone + Copy,
    S: Spatial<V> + Clone + Copy,
    T: Temporable<V> + Clone + Copy,
    ST: SpaceTemporal<V> + Clone + Copy,
    V: Default
        + Copy
        + Clone
        + Hash
        + Eq
        + PartialEq
        + Add<V, Output = V>
        + Sub<V, Output = V>
        + Mul<V, Output = V>,
{
    /// Constructs a new CSM.
    pub fn new(state_actions: &'l CSMStateActions<'l, D, S, T, ST, V>) -> Self {
        // Generate a new HashMap from the collection.
        let mut state_map: CSMMap<'l, D, S, T, ST, V> = HashMap::with_capacity(state_actions.len());
        for (state, action) in state_actions {
            state_map.insert(*state.id(), (state, action));
        }

        Self {
            state_actions: RefCell::new(state_map),
        }
    }

    /// Returns the number of elements in the CSM.
    pub fn len(&self) -> usize {
        self.state_actions.borrow().len()
    }

    /// Returns true if the CSM contains no elements.
    pub fn is_empty(&self) -> bool {
        self.state_actions.borrow().is_empty()
    }
}

impl<'l, D, S, T, ST, V> CSM<'l, D, S, T, ST, V>
where
    D: Datable + Clone + Copy,
    S: Spatial<V> + Clone + Copy,
    T: Temporable<V> + Clone + Copy,
    ST: SpaceTemporal<V> + Clone + Copy,
    V: Default
        + Copy
        + Clone
        + Hash
        + Eq
        + PartialEq
        + Add<V, Output = V>
        + Sub<V, Output = V>
        + Mul<V, Output = V>,
{
    /// Inserts a new state action at the index position idx.
    /// Returns UpdateError if the index already exists.
    pub fn add_single_state(
        &self,
        idx: usize,
        state_action: (&'l CausalState<'l, D, S, T, ST, V>, &'l CausalAction),
    ) -> Result<(), UpdateError> {
        // Check if the key exists, if so return error
        if self.state_actions.borrow().get(&idx).is_some() {
            return Err(UpdateError(format!("State {} already exists.", idx)));
        }

        // Insert the new state/action at the idx position
        self.state_actions.borrow_mut().insert(idx, state_action);

        Ok(())
    }

    /// Removes a state action at the index position idx.
    /// Returns UpdateError if the index does not exists.
    pub fn remove_single_state(&self, id: usize) -> Result<(), UpdateError> {
        // Need binding to prevent dropped tmp value warnings
        let mut binding = self.state_actions.borrow_mut();

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
}

impl<'l, D, S, T, ST, V> CSM<'l, D, S, T, ST, V>
where
    D: Datable + Clone + Copy,
    S: Spatial<V> + Clone + Copy,
    T: Temporable<V> + Clone + Copy,
    ST: SpaceTemporal<V> + Clone + Copy,
    V: Default
        + Copy
        + Clone
        + Hash
        + Eq
        + PartialEq
        + Add<V, Output = V>
        + Sub<V, Output = V>
        + Mul<V, Output = V>,
{
    /// Evaluates a single causal state at the index position idx.
    /// Returns ActionError if the evaluation failed.
    pub fn eval_single_state(&self, id: usize, data: NumericalValue) -> Result<(), ActionError> {
        // Need binding to prevent dropped tmp value warnings
        let binding = self.state_actions.borrow();

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
                "CSM[eval]: Error evaluating causal state: {}",
                state
            )));
        }

        // Unpack the bool result that triggers the action
        let trigger =
            eval.expect("CSM[eval]: Failed to unwrap evaluation result from causal state}");

        // If the state evaluated to true, fire the associated action.
        if trigger && action.fire().is_err() {
            return Err(ActionError(format!(
                "CSM[eval]: Failed to fire action associated with causal state {}",
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
        state_action: (&'l CausalState<'l, D, S, T, ST, V>, &'l CausalAction),
    ) -> Result<(), UpdateError> {
        // Check if the key exists, if not return error
        if self.state_actions.borrow().get(&idx).is_none() {
            return Err(UpdateError(format!(
                "State {} does not exists. Add it first before evaluating",
                idx
            )));
        }

        // Update state/action at the idx position
        self.state_actions.borrow_mut().insert(idx, state_action);

        Ok(())
    }
}

impl<'l, D, S, T, ST, V> CSM<'l, D, S, T, ST, V>
where
    D: Datable + Clone + Copy,
    S: Spatial<V> + Clone + Copy,
    T: Temporable<V> + Clone + Copy,
    ST: SpaceTemporal<V> + Clone + Copy,
    V: Default
        + Copy
        + Clone
        + Hash
        + Eq
        + PartialEq
        + Add<V, Output = V>
        + Sub<V, Output = V>
        + Mul<V, Output = V>,
{
    /// Evaluates all causal states in the CSM.
    /// Returns ActionError if the evaluation failed.
    pub fn eval_all_states(&self) -> Result<(), ActionError> {
        for (_, (state, action)) in self.state_actions.borrow().iter() {
            let eval = state.eval();

            // check if the causal state evaluation returned an error
            if eval.is_err() {
                return Err(ActionError(format!(
                    "CSM[eval]: Error evaluating causal state: {}",
                    state
                )));
            }

            // Unpack the bool result
            let trigger =
                eval.expect("CSM[eval]: Failed to unwrap evaluation result from causal state}");

            // If the state evaluated to true, fire the associated action.
            if trigger && action.fire().is_err() {
                return Err(ActionError(format!(
                    "CSM[eval]: Failed to fire action associated with causal state {}",
                    state
                )));
            }
        }

        Ok(())
    }

    /// Updates all causal state with a new state collection.
    /// Note, this operation erases all previous states in the CSM by generating a new collection.
    /// Returns UpdateError if the update operation failed.
    pub fn update_all_states(&self, state_actions: &'l CSMStateActions<'l, D, S, T, ST, V>) {
        // Generate a new HashMap from the collection
        let mut state_map: CSMMap<'l, D, S, T, ST, V> = HashMap::with_capacity(state_actions.len());
        for (state, action) in state_actions {
            state_map.insert(*state.id(), (state, action));
        }

        // Replace the existing map with the newly generated one.
        *self.state_actions.borrow_mut() = state_map
    }
}
