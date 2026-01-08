/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_core::CausalEffectSystem;
use deep_causality_core::{CausalityError, EffectLog, PropagatingEffectWitness};
use deep_causality_haft::Effect3;
use std::any::TypeId;

#[test]
fn test_causal_effect_system_types() {
    // Verify Fixed1 is CausalityError
    assert_eq!(
        TypeId::of::<<CausalEffectSystem as Effect3>::Fixed1>(),
        TypeId::of::<CausalityError>()
    );

    // Verify Fixed2 is EffectLog
    assert_eq!(
        TypeId::of::<<CausalEffectSystem as Effect3>::Fixed2>(),
        TypeId::of::<EffectLog>()
    );

    // Verify HktWitness is PropagatingEffectWitness<CausalityError, EffectLog>
    assert_eq!(
        TypeId::of::<<CausalEffectSystem as Effect3>::HktWitness>(),
        TypeId::of::<PropagatingEffectWitness<CausalityError, EffectLog>>()
    );
}
