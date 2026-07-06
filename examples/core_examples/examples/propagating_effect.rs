/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_core::PropagatingEffect;

fn main() {
    println!("--- PropagatingEffect Example ---");

    // --------------------------------------------------------------------------------------------
    // ENGINEERING VALUE: Pure Functional Pipelines
    //
    // `PropagatingEffect` allows you to chain pure functions together in a way that automatically:
    // 1. **Short-circuits on Error**: If step A fails, step B acts as a no-op loop-through.
    // 2. **Logs Execution**: Every step is recorded in `EffectLog` (not shown here for brevity
    //    but present in the struct).
    //
    // This replaces "if err != nil { return err }" boilerplate with declarative `bind` or `map`.
    // --------------------------------------------------------------------------------------------

    // 1. Create a pure effect (Success)
    // PropagatingEffect is a type alias for CausalEffectPropagationProcess with unit state/context.
    let effect_a: PropagatingEffect<i32> = PropagatingEffect::pure(10);
    println!("Effect A: {:?}", effect_a.value().unwrap());

    // 2. Map over the value with the fluent `fmap` (the Functor operation).
    let effect_b = effect_a.fmap(|x| x * 2);
    println!("Effect B (A * 2): {:?}", effect_b.value().unwrap());

    // 3. Chain operations with the fluent `bind` (the Monad operation).
    // The closure receives the wrapped value, the threaded state, and the context;
    let effect_c = effect_b.bind(|x, _state, _ctx| {
        let x = x.into_value().unwrap_or_default();
        if x > 15 {
            // Return a new effect
            PropagatingEffect::pure(x + 5)
        } else {
            PropagatingEffect::pure(x)
        }
    });
    println!("Effect C (B + 5 if > 15): {:?}", effect_c.value().unwrap());
}
