/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
pub mod generative_processor;
pub mod generator;

use crate::errors::ModelGenerativeError;
use crate::prelude::{
    Context, Datable, GenerativeOutput, GenerativeTrigger, SpaceTemporal, Spatial, Symbolic,
    Temporal,
};
use std::hash::Hash;

// The user's Generative Enum must implement this trait.
// It is generic over itself to allow the Evolve variant to work.
#[allow(clippy::type_complexity)]
pub trait Generatable<D, S, T, ST, SYM, VS, VT, G>
where
    D: Default + Datable + Copy + Clone + Hash + Eq + PartialEq,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
    G: Generatable<D, S, T, ST, SYM, VS, VT, G> + Sized,
{
    fn generate(
        &mut self,
        trigger: &GenerativeTrigger<D>,
        // We need to pass the full context for informed decisions
        context: &Context<D, S, T, ST, SYM, VS, VT>,
    ) -> Result<GenerativeOutput<D, S, T, ST, SYM, VS, VT, G>, ModelGenerativeError>;
}
