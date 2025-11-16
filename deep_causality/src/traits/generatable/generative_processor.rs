/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

// This trait can be placed in a new module, e.g., `crate::processing`

use crate::{
    Causaloid, Context, ContextId, ContextuableGraph, Datable, ExtendableContextuableGraph,
    Generatable, GenerativeOutput, Identifiable, IntoEffectValue, ModelValidationError,
    SpaceTemporal, Spatial, Symbolic, Temporal,
};
use std::hash::Hash;

/// A trait for types that can process the output of a `Generatable` instance.
///
/// It defines the required state for the processing (a destination for the Causaloid
/// and Context) and provides a default implementation for the processing logic itself,
/// making it highly reusable.
#[allow(clippy::type_complexity)]
pub trait GenerativeProcessor<I, O, D, S, T, ST, SYM, VS, VT, G>
where
    I: IntoEffectValue,
    O: IntoEffectValue,
    D: Default + Datable + Copy + Clone + Hash + Eq + PartialEq,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
    G: Generatable<I, O, D, S, T, ST, SYM, VS, VT, G>,
{
    /// Provides mutable access to the destination for the generated Causaloid.
    /// This is a required method for the trait implementor.
    fn get_causaloid_dest(&mut self) -> &mut Option<Causaloid<I, O, D, S, T, ST, SYM, VS, VT>>;

    /// Provides mutable access to the destination for the generated Context.
    /// This is a required method for the trait implementor.
    fn get_context_dest(&mut self) -> &mut Option<Context<D, S, T, ST, SYM, VS, VT>>;

    /// A helper method to get and verify the target context.
    /// This reduces boilerplate in the main processing logic.
    #[doc(hidden)]
    fn get_and_verify_context(
        &mut self,
        target_id: ContextId,
    ) -> Result<&mut Context<D, S, T, ST, SYM, VS, VT>, ModelValidationError> {
        let context = self
            .get_context_dest()
            .as_mut()
            .ok_or(ModelValidationError::TargetContextNotFound { id: target_id })?;

        if context.id() != target_id {
            Err(ModelValidationError::TargetContextNotFound { id: target_id })
        } else {
            Ok(context)
        }
    }

    /// Processes a single `GenerativeOutput` command, mutating the state provided
    /// by the getter methods.
    ///
    /// This method has a default implementation, providing reusable processing logic
    /// to any type that implements this trait.
    fn process_output(
        &mut self,
        output: GenerativeOutput<I, O, D, S, T, ST, SYM, VS, VT, G>,
    ) -> Result<(), ModelValidationError> {
        match output {
            GenerativeOutput::NoOp => Ok(()),

            GenerativeOutput::CreateCausaloid(id, causaloid) => {
                let causaloid_dest = self.get_causaloid_dest();
                if causaloid_dest.is_some() {
                    return Err(ModelValidationError::DuplicateCausaloidID { id });
                }
                *causaloid_dest = Some(causaloid);
                Ok(())
            }

            GenerativeOutput::UpdateCausaloid(id, causaloid) => {
                let causaloid_dest = self.get_causaloid_dest();
                if causaloid_dest.is_none() {
                    return Err(ModelValidationError::TargetCausaloidNotFound { id });
                }
                *causaloid_dest = Some(causaloid);
                Ok(())
            }

            GenerativeOutput::DeleteCausaloid(id) => {
                let causaloid_dest = self.get_causaloid_dest();
                if causaloid_dest.is_none() {
                    return Err(ModelValidationError::TargetCausaloidNotFound { id });
                }
                *causaloid_dest = None;
                Ok(())
            }

            GenerativeOutput::CreateBaseContext { id, name, capacity } => {
                let context_dest = self.get_context_dest();
                if context_dest.is_some() {
                    return Err(ModelValidationError::DuplicateContextId { id });
                }
                *context_dest = Some(Context::with_capacity(id, &name, capacity));
                Ok(())
            }

            GenerativeOutput::UpdateContext { id, new_name } => {
                let context = self.get_and_verify_context(id)?;
                if let Some(name) = new_name {
                    context.set_name(name);
                }
                Ok(())
            }

            GenerativeOutput::DeleteContext { id } => {
                // First, verify the context exists and has the correct ID.
                self.get_and_verify_context(id)?;
                // If verification passes, we can safely set the destination to None.
                *self.get_context_dest() = None;
                Ok(())
            }

            GenerativeOutput::CreateExtraContext {
                extra_context_id,
                capacity,
            } => {
                let context = self
                    .get_context_dest()
                    .as_mut()
                    // It's an error to create an extra context if the main one doesn't exist.
                    // Note: This assumes a new `BaseContextNotFound` error variant.
                    .ok_or(ModelValidationError::BaseContextNotFound)?;

                // Call the new method on the context.
                // We set `default` to false, as creating an extra context should not
                // automatically make it the active one.
                context
                    .extra_ctx_add_new_with_id(extra_context_id, capacity, false)
                    // Map the specific error from the context layer to the model validation layer.
                    // Note: This assumes a new `DuplicateExtraContextId` error variant.
                    .map_err(|_| ModelValidationError::DuplicateExtraContextId {
                        id: extra_context_id,
                    })
            }

            GenerativeOutput::AddContextoidToContext {
                context_id,
                contextoid,
            } => {
                let context = self.get_and_verify_context(context_id)?;
                match context.add_node(contextoid) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(ModelValidationError::AddContextoidError { err: e.to_string() }),
                }
            }

            GenerativeOutput::UpdateContextoidInContext {
                context_id,
                existing_contextoid,
                new_contextoid,
            } => {
                let context = self.get_and_verify_context(context_id)?;
                context
                    .update_node(existing_contextoid, new_contextoid)
                    .map_err(|_| ModelValidationError::TargetContextoidNotFound {
                        id: existing_contextoid,
                    })
            }

            GenerativeOutput::DeleteContextoidFromContext {
                context_id,
                contextoid_id,
            } => {
                let context = self.get_and_verify_context(context_id)?;
                context.remove_node(contextoid_id).map_err(|_| {
                    ModelValidationError::TargetContextoidNotFound { id: contextoid_id }
                })
            }

            GenerativeOutput::Composite(outputs) => {
                for out in outputs {
                    self.process_output(out)?;
                }
                Ok(())
            }

            GenerativeOutput::Evolve(_) => Err(ModelValidationError::UnsupportedOperation {
                operation:
                    "The Evolve variant is not supported by the default GenerativeProcessor."
                        .to_string(),
            }),
        }
    }
}
