/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::EffectLog;
use crate::errors::causality_error::CausalityError;
use crate::types::causal_effect_propagation_process::hkt::CausalEffectPropagationProcessWitness;
use core::marker::PhantomData;
use deep_causality_haft::Effect5;

/// The monadic system definition for Causal Effect Propagation.
///
/// `CausalSystem` acts as the "Type Constructor" or registry for the `Effect5` trait,
/// binding together the specific types used in the DeepCausality monadic system:
/// *   **Fixed1 (State)**: S
/// *   **Fixed2 (Context)**: C
/// *   **Fixed3 (Error)**: CausalityError
/// *   **Fixed4 (Log)**: EffectLog
///
/// It does not hold data itself (`PhantomData`) but is essential for the higher-kinded
/// type machinery that allows `CausalMonad` to function generically.
pub struct CausalSystem<S, C>(PhantomData<(S, C)>);

impl<S, C> Effect5 for CausalSystem<S, C>
where
    S: Clone + Default,
    C: Clone,
{
    type Fixed1 = S;
    type Fixed2 = C;
    type Fixed3 = CausalityError;
    type Fixed4 = EffectLog;

    type HktWitness = CausalEffectPropagationProcessWitness<
        Self::Fixed1,
        Self::Fixed2,
        Self::Fixed3,
        Self::Fixed4,
    >;
}
