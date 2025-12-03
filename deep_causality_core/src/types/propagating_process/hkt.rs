/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{CausalityError, CausalityErrorEnum, EffectLog, EffectValue, PropagatingProcess};
use core::marker::PhantomData;
use deep_causality_haft::{Applicative, Functor, HKT, LogAppend, Monad, Placeholder};

pub struct PropagatingProcessWitness<S, C>(Placeholder, PhantomData<S>, PhantomData<C>);

impl<S, C> HKT for PropagatingProcessWitness<S, C> {
    type Type<T> = PropagatingProcess<T, S, C>;
}

impl<S, C> Functor<PropagatingProcessWitness<S, C>> for PropagatingProcessWitness<S, C>
where
    S: Clone,
    C: Clone,
{
    fn fmap<A, B, Func>(
        m_a: <PropagatingProcessWitness<S, C> as HKT>::Type<A>,
        f: Func,
    ) -> <PropagatingProcessWitness<S, C> as HKT>::Type<B>
    where
        Func: FnOnce(A) -> B,
    {
        if m_a.error.is_some() {
            return PropagatingProcess {
                value: EffectValue::None,
                state: m_a.state,
                context: m_a.context,
                error: m_a.error,
                logs: m_a.logs,
            };
        }

        match m_a.value.into_value() {
            Some(a) => PropagatingProcess {
                value: EffectValue::Value(f(a)),
                state: m_a.state,
                context: m_a.context,
                error: m_a.error,
                logs: m_a.logs,
            },
            None => PropagatingProcess {
                value: EffectValue::None,
                state: m_a.state,
                context: m_a.context,
                error: Some(CausalityError::new(CausalityErrorEnum::InternalLogicError)),
                logs: m_a.logs,
            },
        }
    }
}

impl<S, C> Applicative<PropagatingProcessWitness<S, C>> for PropagatingProcessWitness<S, C>
where
    S: Clone + Default,
    C: Clone,
{
    fn pure<T>(value: T) -> <Self as HKT>::Type<T> {
        PropagatingProcess {
            value: EffectValue::Value(value),
            state: S::default(),
            context: None,
            error: None,
            logs: EffectLog::default(),
        }
    }

    fn apply<A, B, Func>(
        f_ab: <Self as HKT>::Type<Func>,
        mut f_a: <Self as HKT>::Type<A>,
    ) -> <Self as HKT>::Type<B>
    where
        Func: FnMut(A) -> B,
        A: Clone,
    {
        let mut combined_logs = f_ab.logs;
        combined_logs.append(&mut f_a.logs);

        let error = f_ab.error.or(f_a.error);
        if error.is_some() {
            return PropagatingProcess {
                value: EffectValue::None,
                state: f_ab.state, // State from the function context is carried over
                context: f_ab.context.or(f_a.context),
                error,
                logs: combined_logs,
            };
        }

        match (f_ab.value.into_value(), f_a.value.into_value()) {
            (Some(mut f), Some(a)) => PropagatingProcess {
                value: EffectValue::Value(f(a)),
                state: f_ab.state, // State from the function context is carried over
                context: f_ab.context.or(f_a.context),
                error,
                logs: combined_logs,
            },
            _ => PropagatingProcess {
                value: EffectValue::None,
                state: f_ab.state, // State from the function context is carried over
                context: f_ab.context.or(f_a.context),
                error: Some(CausalityError::new(CausalityErrorEnum::InternalLogicError)),
                logs: combined_logs,
            },
        }
    }
}

impl<S, C> Monad<PropagatingProcessWitness<S, C>> for PropagatingProcessWitness<S, C>
where
    S: Clone + Default,
    C: Clone,
{
    fn bind<A, B, Func>(m_a: <Self as HKT>::Type<A>, mut f: Func) -> <Self as HKT>::Type<B>
    where
        Func: FnMut(A) -> <Self as HKT>::Type<B>,
    {
        if let Some(error) = m_a.error {
            return PropagatingProcess {
                value: EffectValue::None, // No B can be produced
                state: m_a.state,
                context: m_a.context,
                error: Some(error),
                logs: m_a.logs,
            };
        }

        let a = match m_a.value.into_value() {
            Some(val) => val,
            None => {
                return PropagatingProcess {
                    value: EffectValue::None,
                    state: m_a.state,
                    context: m_a.context,
                    error: Some(CausalityError::new(CausalityErrorEnum::InternalLogicError)),
                    logs: m_a.logs,
                };
            }
        };

        let mut next_effect = f(a);

        let mut combined_logs = m_a.logs;
        combined_logs.append(&mut next_effect.logs);

        PropagatingProcess {
            value: next_effect.value,
            state: m_a.state,     // State is passed through, not updated by f
            context: m_a.context, // Context is passed through
            error: next_effect.error,
            logs: combined_logs,
        }
    }
}
