/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{
    ActionError, CSM, CausalState, CsmError, Datable, DeonticExplainable, DeonticInferable,
    PropagatingEffect, ProposedAction, SpaceTemporal, Spatial, Symbolic, TeloidModal, Temporal,
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
    /// If the state evaluates to `PropagatingEffect::Deterministic(true)`, the associated action is fired.
    ///
    /// # Errors
    /// Returns `CsmError` if the state does not exist, evaluation fails, the effect is not
    /// deterministic, or the action fails to fire.
    pub fn eval_single_state(&self, id: usize, data: &PropagatingEffect) -> Result<(), CsmError> {
        let binding = self.state_actions.read().unwrap();

        let (state, action) = binding.get(&id).ok_or_else(|| {
            CsmError::Action(
                format!("State {id} does not exist. Add it first before evaluating.").into(),
            )
        })?;

        let effect = state.eval_with_data(data)?;

        match &effect {
            PropagatingEffect::Deterministic(true) => {
                if let Some((ethos, tags)) = &self.effect_ethos {
                    if let Some(context) = state.context() {
                        let action_name = format!(
                            "proposed action for CSM State: {} version: {}",
                            state.id(),
                            state.version()
                        );

                        let proposed_action =
                            self.create_proposed_action(action_name, state, &effect)?;
                        let verdict = ethos.evaluate_action(&proposed_action, context, tags)?;

                        if verdict.outcome() == TeloidModal::Impermissible {
                            let explanation = ethos.explain_verdict(&verdict)?;
                            return Err(CsmError::Forbidden(explanation));
                        } else {
                            action.fire()?;
                        }
                    } else {
                        return Err(CsmError::Action(ActionError::new(
                            "Cannot evaluate action with ethos because state context is missing."
                                .into(),
                        )));
                    }
                } else {
                    action.fire()?;
                }
                Ok(())
            }
            PropagatingEffect::Deterministic(false) => {
                // State evaluated to false, do nothing.
                Ok(())
            }
            _ => {
                // The effect was not deterministic, which is an invalid state for a CSM.
                Err(CsmError::Action(ActionError(format!(
                    "CSM[eval]: Invalid non-deterministic effect '{:?}' for state {}",
                    effect,
                    state.id()
                ))))
            }
        }
    }

    /// Evaluates all causal states in the CSM using their internal data.
    /// For each state that evaluates to `PropagatingEffect::Deterministic(true)`, the associated action is fired.
    ///
    /// # Errors
    /// Returns `CsmError` if any state evaluation fails, produces a non-deterministic effect,
    /// or any triggered action fails to fire.
    pub fn eval_all_states(&self) -> Result<(), CsmError> {
        let binding = self.state_actions.read().unwrap();

        for (_id, (state, action)) in binding.iter() {
            let effect = state.eval()?;

            match &effect {
                PropagatingEffect::Deterministic(true) => {
                    if let Some((ethos, tags)) = &self.effect_ethos {
                        if let Some(context) = state.context() {
                            let action_name = format!(
                                "proposed action for CSM State: {} version: {}",
                                state.id(),
                                state.version()
                            );

                            let proposed_action =
                                self.create_proposed_action(action_name, state, &effect)?;
                            let verdict = ethos.evaluate_action(&proposed_action, context, tags)?;

                            if verdict.outcome() == TeloidModal::Impermissible {
                                let explanation = ethos.explain_verdict(&verdict)?;
                                return Err(CsmError::Forbidden(explanation));
                            } else {
                                action.fire()?;
                            }
                        } else {
                            return Err(CsmError::Action(ActionError::new(
                                "Cannot evaluate action with ethos because state context is missing."
                                    .into(),
                            )));
                        }
                    } else {
                        action.fire()?
                    }
                }
                PropagatingEffect::Deterministic(false) => {
                    // State evaluated to false, do nothing, continue loop.
                }
                _ => {
                    // The effect was not deterministic, which is an invalid state for a CSM.
                    return Err(CsmError::Action(ActionError(format!(
                        "CSM[eval]: Invalid non-deterministic effect '{:?}' for state {}",
                        effect,
                        state.id()
                    ))));
                }
            }
        }

        Ok(())
    }

    fn create_proposed_action(
        &self,
        action_name: String,
        state: &CausalState<D, S, T, ST, SYM, VS, VT>,
        effect: &PropagatingEffect,
    ) -> Result<ProposedAction, CsmError> {
        let mut params = HashMap::new();
        params.insert("trigger_effect".to_string(), effect.clone().into());

        Ok(ProposedAction::new(state.id() as u64, action_name, params))
    }
}
