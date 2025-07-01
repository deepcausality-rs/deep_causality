/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
mod getters;
mod identifiable;
mod model_builder_processor;

use crate::prelude::{
    Assumption, Causaloid, Context, Datable, Generatable, GenerativeProcessor, GenerativeTrigger,
    ModelBuildError, ModelValidationError, Symbolic,
};
use crate::traits::contextuable::space_temporal::SpaceTemporal;
use crate::traits::contextuable::spatial::Spatial;
use crate::traits::contextuable::temporal::Temporal;
use crate::types::model_types::model::model_builder_processor::ModelBuilderProcessor;
use std::hash::Hash;
use std::sync::Arc;

#[allow(clippy::type_complexity)]
#[derive(Debug)]
pub struct Model<D, S, T, ST, SYM, VS, VT>
where
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
    causaloid: Arc<Causaloid<D, S, T, ST, SYM, VS, VT>>,
    context: Option<Arc<Context<D, S, T, ST, SYM, VS, VT>>>,
}

#[allow(clippy::type_complexity)]
impl<D, S, T, ST, SYM, VS, VT> Model<D, S, T, ST, SYM, VS, VT>
where
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
        causaloid: Arc<Causaloid<D, S, T, ST, SYM, VS, VT>>,
        context: Option<Arc<Context<D, S, T, ST, SYM, VS, VT>>>,
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
impl<D, S, T, ST, SYM, VS, VT> Model<D, S, T, ST, SYM, VS, VT>
where
    D: Default + Datable + Copy + Clone + Hash + Eq + PartialEq,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
{
    /// Creates a new `Model` by running a one-shot generation process.
    ///
    /// This constructor uses a `GenerativeProcessor` internally to build the Causaloid
    /// and Context from the generator's output.
    #[allow(clippy::type_complexity)]
    pub fn with_generator<G>(
        id: u64,
        author: &str,
        description: &str,
        assumptions: Option<Arc<Vec<Assumption>>>,
        mut generator: G,
        trigger: &GenerativeTrigger<D>,
    ) -> Result<Self, ModelBuildError>
    where
        G: Generatable<D, S, T, ST, SYM, VS, VT, G>,
    {
        let initial_contexts = Context::with_capacity(0, "Base context", 120); // Empty world view for build
        let output = generator.generate(trigger, &initial_contexts)?;

        // 1. Create the processor.
        let mut processor = ModelBuilderProcessor::new();

        // 2. Use the reusable trait method to process the output.
        processor.process_output(output)?;

        // 3. Consume the processor to take ownership of the results.
        let (causaloid_opt, context_opt) = processor.into_results();

        // 4. Validate and extract the results.
        let mut final_causaloid =
            causaloid_opt.ok_or(ModelValidationError::MissingCreateCausaloid)?;

        // The context is also owned now.
        let final_context_arc = if let Some(context) = context_opt {
            // Create the Arc for the context.
            let context_arc = Arc::new(context);

            final_causaloid.set_context(Some(Arc::clone(&context_arc)));
            final_causaloid.set_has_context(true);

            // Return the Arc for the model's own field.
            Some(context_arc)
        } else {
            None
        };

        // 5. Construct the model.
        // All types now match correctly because we have owned values to wrap in Arcs.
        Ok(Self::new(
            id,
            author,
            description,
            assumptions,
            Arc::new(final_causaloid),
            final_context_arc,
        ))
    }
}
