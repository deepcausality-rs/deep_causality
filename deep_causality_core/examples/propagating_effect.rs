/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_core::PropagatingEffect;
use deep_causality_core::PropagatingEffectWitness;
use deep_causality_haft::{Applicative, Functor, Monad};

fn main() {
    println!("--- PropagatingEffect Example ---");

    // 1. Create a pure effect (Success)
    // PropagatingEffect is a type alias for CausalEffectPropagationProcess with unit state/context.
    let effect_a: PropagatingEffect<i32> = PropagatingEffectWitness::pure(10);
    println!("Effect A: {:?}", effect_a.value);

    // 2. Use Functor to map over the value
    let effect_b = PropagatingEffectWitness::fmap(effect_a, |x| x * 2);
    println!("Effect B (A * 2): {:?}", effect_b.value);

    // 3. Use Monad to chain operations (bind)
    // Note: PropagatingEffect is stateless, so we just transform values.
    let effect_c = PropagatingEffectWitness::bind(effect_b, |x| {
        if x > 15 {
            // Return a new effect
            PropagatingEffectWitness::pure(x + 5)
        } else {
            PropagatingEffectWitness::pure(x)
        }
    });
    println!("Effect C (B + 5 if > 15): {:?}", effect_c.value);
}
