/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{CausalEffect, CausalityError, CausalityErrorEnum, EffectLog, PropagatingProcess};
use core::marker::PhantomData;
use deep_causality_haft::{
    Applicative, Functor, HKT, LogAppend, NoConstraint, Placeholder, Pure, Satisfies,
};

pub struct PropagatingProcessWitness<S, C>(Placeholder, PhantomData<S>, PhantomData<C>);

impl<S, C> HKT for PropagatingProcessWitness<S, C> {
    type Constraint = NoConstraint;
    type Type<T> = PropagatingProcess<T, S, C>;
}

impl<S, C> Functor<Self> for PropagatingProcessWitness<S, C>
where
    S: Clone,
    C: Clone,
{
    fn fmap<A, B, Func>(m_a: <Self as HKT>::Type<A>, f: Func) -> <Self as HKT>::Type<B>
    where
        A: Satisfies<<Self as HKT>::Constraint>,
        B: Satisfies<<Self as HKT>::Constraint>,
        Func: FnOnce(A) -> B,
    {
        // Error short-circuits: `f` is not invoked (left zero); state/context/logs preserved.
        let outcome = match m_a.outcome {
            Err(error) => Err(error),
            Ok(value) => match value.into_value() {
                Some(a) => Ok(CausalEffect::value(f(a))),
                None => Err(CausalityError::new(CausalityErrorEnum::InternalLogicError)),
            },
        };

        PropagatingProcess::new(outcome, m_a.state, m_a.context, m_a.logs)
    }
}

impl<S, C> Pure<Self> for PropagatingProcessWitness<S, C>
where
    S: Clone + Default,
    C: Clone,
{
    fn pure<T>(value: T) -> <Self as HKT>::Type<T>
    where
        T: Satisfies<<Self as HKT>::Constraint>,
    {
        PropagatingProcess::new(
            Ok(CausalEffect::value(value)),
            S::default(),
            None,
            EffectLog::default(),
        )
    }
}

impl<S, C> Applicative<Self> for PropagatingProcessWitness<S, C>
where
    S: Clone + Default,
    C: Clone,
{
    fn apply<A, B, Func>(
        f_ab: <Self as HKT>::Type<Func>,
        mut f_a: <Self as HKT>::Type<A>,
    ) -> <Self as HKT>::Type<B>
    where
        A: Satisfies<<Self as HKT>::Constraint> + Clone,
        B: Satisfies<<Self as HKT>::Constraint>,
        Func: Satisfies<<Self as HKT>::Constraint> + FnMut(A) -> B,
    {
        let mut combined_logs = f_ab.logs;
        combined_logs.append(&mut f_a.logs);
        let context = f_ab.context.or(f_a.context);

        // Error short-circuits: the function is not invoked; the first error propagates.
        let outcome = match (f_ab.outcome, f_a.outcome) {
            (Err(error), _) | (_, Err(error)) => Err(error),
            (Ok(func), Ok(arg)) => match (func.into_value(), arg.into_value()) {
                (Some(mut f), Some(a)) => Ok(CausalEffect::value(f(a))),
                _ => Err(CausalityError::new(CausalityErrorEnum::InternalLogicError)),
            },
        };

        PropagatingProcess::new(outcome, f_ab.state, context, combined_logs)
    }
}

// NOTE: `PropagatingProcessWitness` deliberately does NOT implement the value-only `Monad` trait.
// Its `bind` continuation (`FnMut(A) -> M<B>`) cannot thread the Markovian `State` channel, so it
// could only freeze state. The correct, state-threading bind is the `CausalMonad` trait (and the
// inherent `bind` method on `CausalEffectPropagationProcess` / `PropagatingProcess`).
