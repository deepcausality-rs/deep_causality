/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Witness for `lean/DeepCausalityFormal/Haft/Signatures.lean` — the CyberneticLoop Kleisli
//! factorization. (RiemannMap carries no equational theory; nothing to witness.)

use deep_causality_haft::{CyberneticLoop, HKT5Unbound, NoConstraint, Satisfies};

// Mirror of the crate's canonical System carrier.
#[derive(Debug, PartialEq, Clone)]
struct System<S, C, A, F, M>(S, C, A, F, M);
struct SystemWitness;
impl HKT5Unbound for SystemWitness {
    type Constraint = NoConstraint;
    type Type<S, C, A, F, M> = System<S, C, A, F, M>;
}

impl CyberneticLoop<SystemWitness> for SystemWitness {
    fn control_step<S, B, C, A, E, FObserve, FDecide>(
        agent: System<S, B, C, A, E>,
        sensor_input: S,
        observe_fn: FObserve,
        decide_fn: FDecide,
    ) -> Result<A, E>
    where
        S: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>,
        C: Satisfies<NoConstraint>,
        A: Satisfies<NoConstraint>,
        E: Satisfies<NoConstraint>,
        FObserve: Fn(S, &C) -> B,
        FDecide: Fn(B, &C) -> A,
    {
        let System(_s, _b, context, _a, _e) = agent;
        Ok(decide_fn(observe_fn(sensor_input, &context), &context))
    }
}

/// THEOREM_MAP: haft.cybernetic.kleisli_factorization
#[test]
fn test_cybernetic_kleisli_factorization() {
    // control_step = (pure ∘ observe) >=> (pure ∘ decide): the OODA chaining is Kleisli
    // composition in the error monad, not a new primitive.
    let ctx = 100;
    let observe = |s: i32, c: &i32| s + c;
    let decide = |b: i32, c: &i32| b * c;

    let sys = System(0, 0, ctx, 0, 0i32);
    let via_trait: Result<i32, i32> = SystemWitness::control_step(sys, 7, observe, decide);

    // Kleisli composite, written out
    let kleisli = |s: i32| -> Result<i32, i32> {
        let b = observe(s, &ctx);
        Ok(decide(b, &ctx))
    };
    assert_eq!(via_trait, kleisli(7));
}
