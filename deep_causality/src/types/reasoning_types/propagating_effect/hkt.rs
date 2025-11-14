/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{CausalEffectLog, CausalPropagatingEffect, PropagatingEffectWitness};
use deep_causality_haft::{Applicative, Functor, HKT, HKT3, Monad};

impl<E, L> HKT for PropagatingEffectWitness<E, L> {
    type Type<T> = CausalPropagatingEffect<T, E, L>;
}

impl<E, L> HKT3<E, L> for PropagatingEffectWitness<E, L> {
    type Type<T> = CausalPropagatingEffect<T, E, L>;
}

impl<E, L> Functor<PropagatingEffectWitness<E, L>> for PropagatingEffectWitness<E, L>
where
    E: 'static,
    L: 'static,
{
    fn fmap<A, B, Func>(
        m_a: <PropagatingEffectWitness<E, L> as HKT>::Type<A>,
        f: Func,
    ) -> <PropagatingEffectWitness<E, L> as HKT>::Type<B>
    where
        Func: FnOnce(A) -> B,
    {
        CausalPropagatingEffect {
            value: f(m_a.value),
            error: m_a.error,
            logs: m_a.logs,
        }
    }
}

impl<E> Applicative<PropagatingEffectWitness<E, CausalEffectLog>>
    for PropagatingEffectWitness<E, CausalEffectLog>
where
    E: 'static + Clone,
{
    fn pure<T>(value: T) -> <PropagatingEffectWitness<E, CausalEffectLog> as HKT>::Type<T> {
        CausalPropagatingEffect {
            value,
            error: None,
            logs: CausalEffectLog::new(),
        }
    }

    fn apply<A, B, Func>(
        mut f_ab: <PropagatingEffectWitness<E, CausalEffectLog> as HKT>::Type<Func>,
        mut f_a: <PropagatingEffectWitness<E, CausalEffectLog> as HKT>::Type<A>,
    ) -> <PropagatingEffectWitness<E, CausalEffectLog> as HKT>::Type<B>
    where
        Func: FnMut(A) -> B,
        A: Clone,
    {
        if f_ab.error.is_some() {
            return CausalPropagatingEffect {
                value: (f_ab.value)(f_a.value),
                error: f_ab.error,
                logs: f_ab.logs,
            };
        }
        if f_a.error.is_some() {
            return CausalPropagatingEffect {
                value: (f_ab.value)(f_a.value),
                error: f_a.error,
                logs: f_a.logs,
            };
        }

        let mut combined_logs = f_ab.logs;
        combined_logs.append(&mut f_a.logs);

        CausalPropagatingEffect {
            value: (f_ab.value)(f_a.value),
            error: None,
            logs: combined_logs,
        }
    }
}

impl<E> Monad<PropagatingEffectWitness<E, CausalEffectLog>>
    for PropagatingEffectWitness<E, CausalEffectLog>
where
    E: 'static + Clone,
{
    fn bind<A, B, Func>(
        m_a: <PropagatingEffectWitness<E, CausalEffectLog> as HKT>::Type<A>,
        f: Func,
    ) -> <PropagatingEffectWitness<E, CausalEffectLog> as HKT>::Type<B>
    where
        Func: FnOnce(A) -> <PropagatingEffectWitness<E, CausalEffectLog> as HKT>::Type<B>,
    {
        if m_a.error.is_some() {
            return CausalPropagatingEffect {
                value: f(m_a.value).value,
                error: m_a.error,
                logs: m_a.logs,
            };
        }
        let mut next_effect = f(m_a.value);
        let mut combined_logs = m_a.logs;
        combined_logs.append(&mut next_effect.logs);
        next_effect.logs = combined_logs;
        next_effect
    }
}
