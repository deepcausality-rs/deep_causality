/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

// This trait can be placed in a new module, e.g., `crate::processing`

use crate::prelude::{
    Causaloid, Context, ContextuableGraph, Datable, Generatable, GenerativeOutput, Identifiable,
    ModelValidationError, SpaceTemporal, Spatial, Symbolic, Temporal,
};
use std::hash::Hash;

/// A trait for types that can process the output of a `Generatable` instance.
///
/// It defines the required state for the processing (a destination for the Causaloid
/// and Context) and provides a default implementation for the processing logic itself,
/// making it highly reusable.
#[allow(clippy::type_complexity)]
pub trait GenerativeProcessor<D, S, T, ST, SYM, VS, VT, G>
where
    D: Default + Datable + Copy + Clone + Hash + Eq + PartialEq,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
    G: Generatable<D, S, T, ST, SYM, VS, VT, G>,
{
    /// Provides mutable access to the destination for the generated Causaloid.
    /// This is a required method for the trait implementor.
    fn get_causaloid_dest(&mut self) -> &mut Option<Causaloid<D, S, T, ST, SYM, VS, VT>>;

    /// Provides mutable access to the destination for the generated Context.
    /// This is a required method for the trait implementor.
    fn get_context_dest(&mut self) -> &mut Option<Context<D, S, T, ST, SYM, VS, VT>>;

    /// Processes a single `GenerativeOutput` command, mutating the state provided
    /// by the getter methods.
    ///
    /// This method has a default implementation, providing reusable processing logic
    /// to any type that implements this trait.
    fn process_output(
        &mut self,
        output: GenerativeOutput<D, S, T, ST, SYM, VS, VT, G>,
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
                let context = self
                    .get_context_dest()
                    .as_mut()
                    .ok_or(ModelValidationError::TargetContextNotFound { id })?;

                if context.id() != id {
                    return Err(ModelValidationError::TargetContextNotFound { id });
                }

                if let Some(name) = new_name {
                    context.set_name(name);
                }

                Ok(())
            }

            GenerativeOutput::DeleteContext { id } => {
                let context_dest = self.get_context_dest();
                if let Some(context) = context_dest.as_ref() {
                    if context.id() == id {
                        *context_dest = None; // Set the Option to None
                        return Ok(());
                    }
                }
                Err(ModelValidationError::TargetContextNotFound { id })
            }

            GenerativeOutput::AddContextoidToContext {
                context_id,
                contextoid,
            } => {
                let context = self
                    .get_context_dest()
                    .as_mut()
                    .ok_or(ModelValidationError::TargetContextNotFound { id: context_id })?;

                if context.id() != context_id {
                    return Err(ModelValidationError::TargetContextNotFound { id: context_id });
                }

                context.add_node(contextoid);
                Ok(())
            }

            GenerativeOutput::UpdateContextoidInContext {
                context_id,
                existing_contextoid,
                new_contextoid,
            } => {
                let context = self
                    .get_context_dest()
                    .as_mut()
                    .ok_or(ModelValidationError::TargetContextNotFound { id: context_id })?;

                if context.id() != context_id {
                    return Err(ModelValidationError::TargetContextNotFound { id: context_id });
                }

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
                let context = self
                    .get_context_dest()
                    .as_mut()
                    .ok_or(ModelValidationError::TargetContextNotFound { id: context_id })?;

                if context.id() != context_id {
                    return Err(ModelValidationError::TargetContextNotFound { id: context_id });
                }

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

            // Explicitly handle unimplemented or unsupported variants
            GenerativeOutput::CreateExtraContext { .. } => {
                Err(ModelValidationError::UnsupportedOperation {
                    operation: "CreateExtraContext is not implemented yet".to_string(),
                })
            }
            GenerativeOutput::Evolve(_) => Err(ModelValidationError::UnsupportedOperation {
                operation:
                    "The Evolve variant is not supported by the default GenerativeProcessor."
                        .to_string(),
            }),
        }
    }
}
