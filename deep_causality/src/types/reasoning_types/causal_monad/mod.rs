/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{CausalEffectLog, CausalPropagatingEffect, CausalityError, PropagatingEffectWitness};
use deep_causality_haft::{Effect3, Functor, HKT3, MonadEffect3};

/// The System Witness for the Causal Effect System.
///
/// This struct implements `Effect3` to fix the error and log types
/// for the `CausalPropagatingEffect` within the `deep_causality` crate.
pub struct CausalEffectSystem;

impl Effect3 for CausalEffectSystem {
    type Fixed1 = CausalityError;
    type Fixed2 = CausalEffectLog;
    type HktWitness = PropagatingEffectWitness<Self::Fixed1, Self::Fixed2>;
}

/// The CausalMonad provides the core monadic operations (`pure` and `bind`)
/// for the `CausalEffectSystem`.
///
/// It handles the sequencing of computations, error propagation, and log accumulation
/// within the `CausalPropagatingEffect` context.
pub struct CausalMonad;

impl MonadEffect3<CausalEffectSystem> for CausalMonad
where
    <CausalEffectSystem as Effect3>::HktWitness:
        Functor<<CausalEffectSystem as Effect3>::HktWitness> + Sized,
{
    fn pure<T>(
        value: T,
    ) -> <<CausalEffectSystem as Effect3>::HktWitness as HKT3<
        <CausalEffectSystem as Effect3>::Fixed1,
        <CausalEffectSystem as Effect3>::Fixed2,
    >>::Type<T> {
        CausalPropagatingEffect {
            value,
            error: None,
            logs: CausalEffectLog::new(),
        }
    }

    fn bind<T, U, Func>(
        effect: <<CausalEffectSystem as Effect3>::HktWitness as HKT3<
            <CausalEffectSystem as Effect3>::Fixed1,
            <CausalEffectSystem as Effect3>::Fixed2,
        >>::Type<T>,
        mut f: Func,
    ) -> <<CausalEffectSystem as Effect3>::HktWitness as HKT3<
        <CausalEffectSystem as Effect3>::Fixed1,
        <CausalEffectSystem as Effect3>::Fixed2,
    >>::Type<U>
    where
        Func: FnMut(
            T,
        ) -> <<CausalEffectSystem as Effect3>::HktWitness as HKT3<
            <CausalEffectSystem as Effect3>::Fixed1,
            <CausalEffectSystem as Effect3>::Fixed2,
        >>::Type<U>,
        U: Default,
    {
        if let Some(error) = effect.error {
            return CausalPropagatingEffect {
                value: U::default(),
                error: Some(error),
                logs: effect.logs,
            };
        }

        let mut next_effect = f(effect.value);
        let mut combined_logs = effect.logs;
        combined_logs.append(&mut next_effect.logs);
        next_effect.logs = combined_logs;
        next_effect
    }
}
