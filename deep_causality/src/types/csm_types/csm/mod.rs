/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

mod eval;

use crate::{CSMMap, CausalAction, CausalState, EffectEthos, StateAction, TeloidTag, UpdateError};
use crate::{Datable, SpaceTemporal, Spatial, Symbolic, Temporal};
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, RwLock};

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
#[allow(clippy::type_complexity)]
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
    effect_ethos: Option<(EffectEthos<D, S, T, ST, SYM, VS, VT>, Vec<TeloidTag>)>,
}

#[allow(clippy::type_complexity)]
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
    pub fn new(
        state_actions: &[(&CausalState<D, S, T, ST, SYM, VS, VT>, &CausalAction)],
        effect_ethos: Option<(EffectEthos<D, S, T, ST, SYM, VS, VT>, &[TeloidTag])>,
    ) -> Self {
        let mut map = HashMap::with_capacity(state_actions.len());

        for (state, action) in state_actions {
            map.insert(*state.id(), ((*state).clone(), (*action).clone()));
        }

        if let Some((ethos, _)) = &effect_ethos {
            if !ethos.is_verified() {
                panic!("EffectEthos must be verified before being used in a CSM.");
            }
        }

        let ethos = effect_ethos.map(|(e, t)| (e, t.to_vec()));

        Self {
            state_actions: Arc::new(RwLock::new(map)),
            effect_ethos: ethos,
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
        if self.state_actions.read().unwrap().contains_key(&idx) {
            return Err(UpdateError(format!("State {idx} already exists.")));
        }

        // Insert the new state/action at the idx position
        self.state_actions
            .write()
            .unwrap()
            .insert(idx, state_action);

        Ok(())
    }

    /// Removes a state action at the index position id.
    /// Returns UpdateError if the index does not exist.
    pub fn remove_single_state(&self, id: usize) -> Result<(), UpdateError> {
        let mut binding = self.state_actions.write().unwrap();

        if binding.remove(&id).is_none() {
            return Err(UpdateError(format!(
                "State {id} does not exist and cannot be removed"
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
        if !self.state_actions.read().unwrap().contains_key(&idx) {
            return Err(UpdateError(format!(
                "State {idx} does not exist. Add it first before updating."
            )));
        }

        // Update state/action at the idx position
        self.state_actions
            .write()
            .unwrap()
            .insert(idx, state_action);

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
        *self.state_actions.write().unwrap() = state_map;
        Ok(())
    }
}
