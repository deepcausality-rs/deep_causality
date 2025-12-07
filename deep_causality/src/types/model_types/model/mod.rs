/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
mod getters;
mod identifiable;
mod transferable;

use crate::traits::contextuable::space_temporal::SpaceTemporal;
use crate::traits::contextuable::spatial::Spatial;
use crate::traits::contextuable::temporal::Temporal;
use crate::{Assumption, Causaloid, Context, Datable, Identifiable, Symbolic};
use std::fmt::Debug;
use std::sync::{Arc, RwLock};

#[allow(clippy::type_complexity)]
#[derive(Debug)]
pub struct Model<I, O, D, S, T, ST, SYM, VS, VT>
where
    I: Default,
    O: Default + Debug,
    D: Datable + Clone,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
{
    id: u64,
    author: String,
    description: String,
    assumptions: Option<Arc<Vec<Assumption>>>,
    /// The root causaloid of the model, encapsulating the main causal logic.
    ///
    /// The `Causaloid` is the core component that defines the causal reasoning mechanism
    /// of the model. It can be a singleton, a collection, or a complex graph of
    /// causal relationships.
    causaloid: Arc<Causaloid<I, O, (), Arc<RwLock<Context<D, S, T, ST, SYM, VS, VT>>>>>,
    context: Option<Arc<RwLock<Context<D, S, T, ST, SYM, VS, VT>>>>,
}

#[allow(clippy::type_complexity)]
impl<I, O, D, S, T, ST, SYM, VS, VT> Model<I, O, D, S, T, ST, SYM, VS, VT>
where
    I: Default,
    O: Default + Debug,
    D: Datable + Clone,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
{
    pub fn new(
        id: u64,
        author: &str,
        description: &str,
        assumptions: Option<Arc<Vec<Assumption>>>,
        causaloid: Arc<Causaloid<I, O, (), Arc<RwLock<Context<D, S, T, ST, SYM, VS, VT>>>>>,
        context: Option<Arc<RwLock<Context<D, S, T, ST, SYM, VS, VT>>>>,
    ) -> Self {
        Self {
            id,
            author: author.to_string(),
            description: description.to_string(),
            assumptions,
            causaloid,
            context,
        }
    }
}

#[allow(clippy::type_complexity)]
impl<I, O, D, S, T, ST, SYM, VS, VT> Model<I, O, D, S, T, ST, SYM, VS, VT>
where
    I: Default + Clone,
    O: Default + Debug + Clone,
    D: Datable + Copy + Clone + PartialEq,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
{
    /// Evolves the model by applying a sequence of operations defined in an `OpTree`.
    ///
    /// This method uses the HKT-based `Interpreter` to execute the operations
    /// and returns a new `Model` instance reflecting the changes, along with a log of modifications.
    pub fn evolve(
        &self,
        op_tree: &crate::OpTree<I, O, D, S, T, ST, SYM, VS, VT>,
    ) -> Result<(Self, crate::ModificationLog), crate::ModelValidationError>
    where
        D: Copy + PartialEq + std::fmt::Debug,
        S: std::fmt::Debug,
        T: std::fmt::Debug,
        ST: std::fmt::Debug,
        SYM: std::fmt::Debug,
        VS: std::fmt::Debug,
        VT: std::fmt::Debug,
    {
        // 1. Initialize the interpreter state with the current model's components.
        //    We clone the Arc-wrapped components to share them with the new state.
        //    The interpreter will handle cloning the inner data if necessary for modifications.
        let mut state = crate::CausalSystemState::new();

        // Populate state with current context if it exists
        if let Some(ctx_arc) = &self.context {
            // We need to lock to get the ID.
            // Note: In a real scenario, we might want to deep clone the context
            // if we want full immutability history, but here we are evolving *this* model.
            // However, the Interpreter expects to *own* the data it modifies or at least
            // have mutable access.
            // For this implementation, we will assume the Interpreter works on the *same*
            // underlying data structures (interior mutability via RwLock) or creates new ones.
            //
            // The `CausalSystemState` uses `HashMap`s. We need to map the current model's
            // context into the state.
            let ctx =
                ctx_arc
                    .read()
                    .map_err(|_| crate::ModelValidationError::InterpreterError {
                        reason: "Failed to read context lock".to_string(),
                    })?;
            state.contexts.insert(ctx.id(), ctx.clone());
        }

        // Populate state with current causaloid
        state
            .causaloids
            .insert(self.causaloid.id(), (*self.causaloid).clone());

        // 2. Create the Interpreter.
        let interpreter = crate::Interpreter::new();

        // 3. Execute the OpTree.
        let effect = interpreter.execute(op_tree, state);

        if let Some(err) = effect.error {
            return Err(err);
        }

        let final_state = effect.value.ok_or_else(|| {
            // This case should ideally not be reached if error is always Some on failure.
            crate::ModelValidationError::InterpreterError {
                reason: "Unknown error: execution failed but no specific error was provided"
                    .to_string(),
            }
        })?;

        let logs = effect.logs;

        // 4. Reconstruct the Model from the final state.
        //    We need to retrieve the (potentially modified) causaloid and context.
        //    Since the ID of the model's main components might not have changed,
        //    we look them up by the original IDs.

        let new_causaloid = final_state
            .causaloids
            .get(&self.causaloid.id())
            .cloned()
            .ok_or_else(|| crate::ModelValidationError::InterpreterError {
                reason: "Main causaloid lost during evolution".to_string(),
            })?;

        let new_context = if let Some(old_ctx) = &self.context {
            let old_id = old_ctx
                .read()
                .map_err(|_| crate::ModelValidationError::InterpreterError {
                    reason: "Context read lock poisoned".to_string(),
                })?
                .id();
            final_state.contexts.get(&old_id).cloned()
        } else {
            None
        };

        Ok((
            Self::new(
                self.id,
                &self.author,
                &self.description,
                self.assumptions.clone(),
                Arc::new(new_causaloid),
                new_context.map(|c| Arc::new(RwLock::new(c))),
            ),
            logs,
        ))
    }
}
