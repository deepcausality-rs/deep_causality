/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{HKT3Unbound, NoConstraint, ParametricMonad, Satisfies};

// Simplified State for testing
#[derive(Debug, Clone)]
pub struct State<SIn, SOut, A> {
    pub value: A,
    pub _phantom_in: std::marker::PhantomData<SIn>,
    pub _phantom_out: std::marker::PhantomData<SOut>,
}

pub struct StateWitness;

impl HKT3Unbound for StateWitness {
    type Constraint = NoConstraint;
    type Type<SIn, SOut, A> = State<SIn, SOut, A>;
}

impl ParametricMonad<StateWitness> for StateWitness {
    fn pure<S, A>(value: A) -> State<S, S, A>
    where
        S: Satisfies<NoConstraint>,
        A: Satisfies<NoConstraint>,
    {
        State {
            value,
            _phantom_in: std::marker::PhantomData,
            _phantom_out: std::marker::PhantomData,
        }
    }

    fn ibind<S1, S2, S3, A, B, F>(m: State<S1, S2, A>, mut f: F) -> State<S1, S3, B>
    where
        S1: Satisfies<NoConstraint>,
        S2: Satisfies<NoConstraint>,
        S3: Satisfies<NoConstraint>,
        A: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>,
        F: FnMut(A) -> State<S2, S3, B>,
    {
        let next = f(m.value);
        State {
            value: next.value,
            _phantom_in: std::marker::PhantomData,
            _phantom_out: std::marker::PhantomData,
        }
    }
}

#[test]
fn test_parametric_monad() {
    // Test pure
    let state: State<i32, i32, i32> = StateWitness::pure(42);
    assert_eq!(state.value, 42);

    // Test ibind with state transition
    let step1: State<i32, String, i32> = State {
        value: 10,
        _phantom_in: std::marker::PhantomData,
        _phantom_out: std::marker::PhantomData,
    };

    let step2 = |val: i32| -> State<String, bool, bool> {
        State {
            value: val > 0,
            _phantom_in: std::marker::PhantomData,
            _phantom_out: std::marker::PhantomData,
        }
    };

    let result = StateWitness::ibind(step1, step2);
    assert!(result.value);
}
