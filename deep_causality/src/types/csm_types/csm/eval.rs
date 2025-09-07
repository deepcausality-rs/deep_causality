/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{
    ActionError, CSM, CausalState, CausalityError, CsmError, Datable, DeonticExplainable,
    DeonticInferable, PropagatingEffect, ProposedAction, SpaceTemporal, Spatial, Symbolic,
    TeloidModal, Temporal,
};
use std::collections::HashMap;
use std::fmt::Debug;

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
    /// Evaluates a single causal state at the index position id.
    /// If the state evaluates to an active effect, the associated action is fired.
    /// An active effect can be Deterministic(true) or an Uncertain type that passes
    /// its hypothesis test.
    ///
    /// # Errors
    /// Returns `CsmError` if the state does not exist, evaluation fails,
    /// or the action fails to fire.
    /// Also returns an error if the causaloid returns a probabilistic effect,
    /// which cannot be evaluated in a single state context.
    pub fn eval_single_state(&self, id: usize, data: &PropagatingEffect) -> Result<(), CsmError> {
        let binding = self.state_actions.read().unwrap();

        let (state, action) = binding.get(&id).ok_or_else(|| {
            CsmError::Action(ActionError(
                format!("State {id} does not exist. Add it first before evaluating."),
            ))
        })?;

        let effect = state.eval_with_data(data)?;

        if effect.is_probabilistic() {
            return Err(CsmError::Causal(CausalityError(
                "Probabilistic effect cannot trigger actions directly in single state evaluation."
                    .into(),
            )));
        }

        self.evaluate_and_fire_action(state, action, &effect)
    }

    /// Evaluates all causal states in the CSM using their internal data.
    /// For each state that evaluates to an active effect, the associated action is fired.
    ///
    /// # Errors
    /// Returns `CsmError` if any state evaluation or action firing fails.
    pub fn eval_all_states(&self) -> Result<(), CsmError> {
        let binding = self.state_actions.read().unwrap();

        for (_id, (state, action)) in binding.iter() {
            let effect = state.eval()?;
            self.evaluate_and_fire_action(state, action, &effect)?;
        }

        Ok(())
    }

    /// Centralized logic to evaluate an effect and fire an action if conditions are met.
    fn evaluate_and_fire_action(
        &self,
        state: &CausalState<D, S, T, ST, SYM, VS, VT>,
        action: &crate::CausalAction,
        effect: &PropagatingEffect,
    ) -> Result<(), CsmError> {
        let is_active = match effect {
            PropagatingEffect::Deterministic(val) => *val,
            PropagatingEffect::UncertainBool(uncertain_bool) => {
                let result = if let Some(params) = state.uncertain_parameter() {
                    uncertain_bool.probability_exceeds(
                        params.threshold(),
                        params.confidence(),
                        params.epsilon(),
                        params.max_samples(),
                    )
                } else {
                    uncertain_bool.implicit_conditional()
                };

                match result {
                    Ok(active) => active,
                    Err(e) => {
                        return Err(CsmError::Causal(CausalityError(format!(
                            "Failed to evaluate uncertain boolean: {}",
                            e
                        ))));
                    }
                }
            }
            PropagatingEffect::UncertainFloat(uncertain_float) => {
                if let Some(params) = state.uncertain_parameter() {
                    let comparison_result = uncertain_float.greater_than(params.threshold());
                    match comparison_result.probability_exceeds(
                        0.5, // When comparing against a threshold, the probability check is > 0.5
                        params.confidence(),
                        params.epsilon(),
                        params.max_samples(),
                    ) {
                        Ok(active) => active,
                        Err(e) => {
                            return Err(CsmError::Causal(CausalityError(format!(
                                "Failed to evaluate uncertain float: {}",
                                e
                            ))));
                        }
                    }
                } else {
                    return Err(CsmError::Causal(CausalityError(
                        "UncertainFloat effect requires UncertainParameter on CausalState".into(),
                    )));
                }
            }
            // Other effect types are considered inactive for triggering actions.
            _ => false,
        };

        if is_active {
            self.fire_action_with_ethos_check(state, action, effect)?;
        }

        Ok(())
    }

    /// Helper to perform the EffectEthos check and fire an action.
    fn fire_action_with_ethos_check(
        &self,
        state: &CausalState<D, S, T, ST, SYM, VS, VT>,
        action: &crate::CausalAction,
        effect: &PropagatingEffect,
    ) -> Result<(), CsmError> {
        if let Some((ethos, tags)) = &self.effect_ethos {
            if let Some(context) = state.context() {
                let proposed_action = self.create_proposed_action(state, effect)?;
                let verdict = ethos.evaluate_action(&proposed_action, context, tags)?;

                if verdict.outcome() == TeloidModal::Impermissible {
                    let explanation = ethos.explain_verdict(&verdict)?;
                    return Err(CsmError::Forbidden(explanation));
                }
            } else {
                return Err(CsmError::Action(ActionError::new(
                    "Cannot evaluate action with ethos because state context is missing.".into(),
                )));
            }
        }

        action.fire()?;
        Ok(())
    }

    /// Creates a `ProposedAction` from a `CausalState` and its triggering effect.
    fn create_proposed_action(
        &self,
        state: &CausalState<D, S, T, ST, SYM, VS, VT>,
        effect: &PropagatingEffect,
    ) -> Result<ProposedAction, CsmError> {
        let mut params = HashMap::new();
        params.insert("trigger_effect".to_string(), effect.clone().into());
        let action_name = format!(
            "proposed action for CSM State: {} version: {}",
            state.id(),
            state.version()
        );

        Ok(ProposedAction::new(state.id() as u64, action_name, params))
    }
}
