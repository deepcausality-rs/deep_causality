/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{
    Causaloid, Context, Datable, Generatable, GenerativeProcessor, SpaceTemporal, Spatial,
    Symbolic, Temporal,
};
use std::hash::Hash;
use std::marker::PhantomData;

/// A temporary state holder used during the `Model::with_generator` construction process.
/// It implements the `GenerativeProcessor` trait to get the reusable logic.
#[allow(clippy::type_complexity)]
pub(crate) struct ModelBuilderProcessor<D, S, T, ST, SYM, VS, VT, G>
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
    causaloid: Option<Causaloid<D, S, T, ST, SYM, VS, VT>>,
    context: Option<Context<D, S, T, ST, SYM, VS, VT>>,
    ty: PhantomData<G>,
}

#[allow(clippy::type_complexity)]
impl<D, S, T, ST, SYM, VS, VT, G> ModelBuilderProcessor<D, S, T, ST, SYM, VS, VT, G>
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
    pub fn new() -> Self {
        Self {
            causaloid: None,
            context: None,
            ty: PhantomData,
        }
    }

    /// Consumes the processor and returns the generated parts.
    pub fn into_results(
        self,
    ) -> (
        Option<Causaloid<D, S, T, ST, SYM, VS, VT>>,
        Option<Context<D, S, T, ST, SYM, VS, VT>>,
    ) {
        (self.causaloid, self.context)
    }
}

// Implement the trait for the builder struct.
#[allow(clippy::type_complexity)]
impl<D, S, T, ST, SYM, VS, VT, G> GenerativeProcessor<D, S, T, ST, SYM, VS, VT, G>
    for ModelBuilderProcessor<D, S, T, ST, SYM, VS, VT, G>
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
    // Fulfill the contract by providing access to our fields.
    fn get_causaloid_dest(&mut self) -> &mut Option<Causaloid<D, S, T, ST, SYM, VS, VT>> {
        &mut self.causaloid
    }

    fn get_context_dest(&mut self) -> &mut Option<Context<D, S, T, ST, SYM, VS, VT>> {
        &mut self.context
    }
}
