/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{
    ActionError, CSM, CausalAction, CausalState, CsmError, CsmEvaluable, LogAddEntry,
    PropagatingEffect,
};
use std::fmt::Debug;

impl<I, O, C> CSM<I, O, C>
where
    I: Default + Clone + Debug + Send + Sync + 'static,
    O: CsmEvaluable + Default + Debug + Clone + Send + Sync + 'static,
    C: Clone + Debug + Send + Sync + 'static,
{
    /// Evaluates a single causal state at the index position id.
    /// If the state evaluates to an active effect, the associated action is fired.
    /// An active effect can be Deterministic(true) or an Uncertain type that passes
    /// its hypothesis test.
    ///
    /// Returns the evaluated effect. Its log holds the causaloid's entries followed by one entry
    /// from the machine that names the state and its version, whether the state was active, and
    /// the action fired with its version.
    ///
    /// # Errors
    /// Returns `CsmError` if the state does not exist, evaluation fails,
    /// or the action fails to fire.
    pub fn eval_single_state(
        &self,
        id: usize,
        data: &PropagatingEffect<I>,
    ) -> Result<PropagatingEffect<O>, CsmError> {
        let binding = self.state_actions.read().unwrap();

        let (state, action) = binding.get(&id).ok_or_else(|| {
            CsmError::Action(ActionError(format!(
                "State {id} does not exist. Add it first before evaluating."
            )))
        })?;

        let effect = state.eval_with_data(data)?;
        if let Err(err) = effect.outcome() {
            return Err(CsmError::Causal(err.clone()));
        }

        self.evaluate_and_fire_action(state, action, effect)
    }

    /// Evaluates all causal states in the CSM using their internal data, in ascending order of
    /// state id. For each state that evaluates to an active effect, the associated action is
    /// fired.
    ///
    /// Returns each state's id with its evaluated effect, in evaluation order. Each log ends with
    /// the machine's entry, as for [`eval_single_state`](Self::eval_single_state).
    ///
    /// # Errors
    /// Returns `CsmError` at the first state whose evaluation or action fails; later states are
    /// not evaluated.
    pub fn eval_all_states(&self) -> Result<Vec<(usize, PropagatingEffect<O>)>, CsmError> {
        let binding = self.state_actions.read().unwrap();

        let mut ids: Vec<usize> = binding.keys().copied().collect();
        ids.sort_unstable();

        ids.into_iter()
            .map(|id| {
                let (state, action) = &binding[&id];
                let effect = state.eval()?;
                if let Err(err) = effect.outcome() {
                    return Err(CsmError::Causal(err.clone()));
                }
                Ok((id, self.evaluate_and_fire_action(state, action, effect)?))
            })
            .collect()
    }

    /// Centralized logic to evaluate an effect, fire the action if the effect is active, and
    /// record the decision in the effect's log.
    fn evaluate_and_fire_action(
        &self,
        state: &CausalState<I, O, C>,
        action: &CausalAction,
        effect: PropagatingEffect<O>,
    ) -> Result<PropagatingEffect<O>, CsmError> {
        let is_active = match effect.value() {
            Some(val) => val
                .is_active(state.uncertain_parameter().as_ref())
                .map_err(CsmError::Causal)?,
            // Other effect kinds (RelayTo, errored carriers, etc.) are considered inactive for
            // triggering actions here.
            _ => false,
        };

        let entry = if is_active {
            action.fire()?;
            format!(
                "CSM state {} (version {}): active; fired '{}' (version {})",
                state.id(),
                state.version(),
                action.description(),
                action.version()
            )
        } else {
            format!(
                "CSM state {} (version {}): inactive",
                state.id(),
                state.version()
            )
        };

        let (outcome, effect_state, context, mut logs) = effect.into_parts();
        logs.add_entry(&entry);
        Ok(PropagatingEffect::new(outcome, effect_state, context, logs))
    }
}
